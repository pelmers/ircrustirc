use listener::CrustyListener;
use action::Action;
use std::io;
use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};

pub struct BotInfo<'a> {
    pub nick: &'a str,
    pub realname: &'a str,
    pub hostname: &'a str,
    pub servername: &'a str,
}

pub struct CrustyBot<'a, L: CrustyListener> {
    info: BotInfo<'a>,
    stream: TcpStream,
    listener: L,
    recvbuf: [u8; 1024]
}

mod parse {
    use ::std::str;
    use ::std::str::{Pattern, Searcher};
    // format: [:PREFIX] [TARGET] COMMAND [:TRAIL]
    // prefix and message can have : in front
    pub fn parse_response(resp: &str) ->
            (Option<&str>, &str, Option<&str>, Option<&str>) {
        let bytes = resp.as_bytes();
        let prefix_end = if bytes[0] == b':' {
            (0..bytes.len()).find(|i| bytes[*i] == b' ').unwrap_or(1)
        } else {
            0
        };
        let trail_start = {
            let mut search = " :".into_searcher(resp);
            // If we find " :" then the response has a trail
            if let Some((_, end))  = search.next_match() {
                end
            } else {
                bytes.len()
            }
        };
        let cmd_end = (prefix_end+1..trail_start).find(
                                |i| bytes[*i] == b' ').unwrap_or(trail_start);
        // now turn the bounds into slices
        let prefix = if prefix_end > 0 {
            Some(slice_to_str(bytes, 1, prefix_end))
        } else {
            None
        };
        let trail = if trail_start < bytes.len() {
            Some(slice_to_str(bytes, trail_start, bytes.len()-1))
        } else {
            None
        };
        let cmd = slice_to_str(bytes,
                               if prefix_end > 0 { prefix_end + 1} else { 0 },
                               cmd_end);
        let params = if cmd_end + 1 < trail_start - 2 {
            Some(slice_to_str(bytes, cmd_end + 1, trail_start - 2))
        } else {
            None
        };
        // so prefix is everything from cmd_end to trail_start
        (prefix, cmd, params, trail)
    }
    fn slice_to_str(bytes: &[u8], lo: usize, hi: usize) -> &str {
        str::from_utf8(&bytes[lo..hi]).unwrap_or("")
    }
}

impl<'a, L: CrustyListener> CrustyBot<'a, L> {
    // Create bot struct, establish connection to addr
    pub fn new<A: ToSocketAddrs + ?Sized>(info: BotInfo<'a>, addr: &A, listener: L) -> io::Result<CrustyBot<'a, L>> {
        match TcpStream::connect(addr) {
            Ok(stream) => Ok(CrustyBot{
                info: info,
                stream: stream,
                listener: listener,
                recvbuf: [0; 1024]
            }),
            Err(e) => Err(e)
        }
    }

    // Send raw data to server.
    fn send_raw(&mut self, raw: &str) -> io::Result<()> {
        self.stream.write_all(raw.as_bytes())
    }

    // Handle the given action.
    fn dispatch_action(&mut self, action: Action) {
        // branches that return do not expect immediate reply from the server
        if let Some(cmd) = action.to_command() {
            print!("Sending: {}", cmd);
            if let Err(e) = self.send_raw(cmd.as_slice()) {
                let action = self.listener.on_ioerror(e, &action);
                self.dispatch_action(action);
            }
        }
        if action.expects_reply() {
            if let Ok(msg) = self.try_receive() {
                let action = self.listener.on_reply(&msg, action);
                self.dispatch_action(action);
            }
        }
    }

    // Parse the response string from the server and dispatch to the listener.
    fn dispatch_response(&mut self, resp: &str) {
        println!("{}", resp);
        let (prefix, cmd, params, trail) = parse::parse_response(resp);
        println!("PRE: {:?}, CMD: {:?}, PAR: {:?}, TRL: {:?}", prefix, cmd, params, trail);
        let action = match cmd {
            "PRIVMSG" => self.listener.on_msg(prefix.unwrap_or(""),
                                              params.unwrap_or(""),
                                              trail.unwrap_or("")),
            "PING" => self.listener.on_ping(trail.unwrap_or("")),
            _ => self.listener.on_other(prefix, cmd, params, trail),
        };
        self.dispatch_action(action);
    }

    // Read up to 1024 bytes from server into recvbuf
    fn read(&mut self) -> io::Result<usize> {
        self.stream.read(self.recvbuf.as_mut_slice())
    }

    // Continuously read until end, return byte vec
    fn receive(&mut self) -> io::Result<Vec<u8>> {
        let mut recv_vec = Vec::with_capacity(self.recvbuf.len());
        loop {
            let s = try!(self.read());
            recv_vec.push_all(self.recvbuf[0..s].as_slice());
            if s < self.recvbuf.len() {
                break;
            }
        }
        Ok(recv_vec)
    }

    // Receive into a utf8-encoded string.
    // Empty string if decode error occurs.
    fn try_receive(&mut self) -> io::Result<String> {
        match self.receive() {
            Ok(vec) => Ok(String::from_utf8(vec).unwrap_or(String::new())),
            Err(e) => Err(e)
        }
    }

    // Connect to the server with bot's credentials
    pub fn connect(&mut self, password: Option<&str>) -> io::Result<()> {
        // Send the password if we need it
        match password {
            Some(p) => try!(self.send_raw(format!("PASS {} \r\n", p).as_slice())),
            _ => ()
        }
        let usercmd = format!("USER {} {} {} :{}\r\n",
                              self.info.nick,
                              self.info.hostname,
                              self.info.servername,
                              self.info.realname);
        let nickcmd = format!("NICK {} \r\n", self.info.nick);
        try!(self.send_raw(usercmd.as_slice()));
        try!(self.send_raw(nickcmd.as_slice()));
        // do a listen here in case server asks for PING or something
        self.listen_once();
        let action = self.listener.on_connect();
        self.dispatch_action(action);
        Ok(())
    }

    // Listen to the server once and dispatch response.
    // Return whether to continue listening.
    fn listen_once(&mut self) -> bool {
        match self.try_receive() {
            Ok(msg) => {
                let lines = msg.as_slice().split("\n");
                for cmd in lines.filter(|c| !c.is_empty()) {
                    self.dispatch_response(cmd);
                }
                !msg.is_empty()
            },
            Err(e) => {
                let action = self.listener.on_ioerror(e, &Action::NoOp);
                self.dispatch_action(action);
                println!("Error occurred.");
                false
            }
        }
    }

    // Listen in this channel forever, process each command received.
    pub fn listen(&mut self) {
        while self.listen_once() {}
    }
}

