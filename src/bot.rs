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

pub mod parse {
    use ::std::str;
    use ::std::str::{Pattern, Searcher};

    #[derive(Debug)]
    pub struct Response<'a> {
        pub prefix: Option<&'a str>,
        pub command: &'a str,
        pub params: Option<&'a str>,
        pub trail: Option<&'a str>
    }

    pub struct User<'a> {
        pub nick: Option<&'a str>,
        pub host: &'a str
    }

    // parse resposne line of format: [:PREFIX] COMMAND [PARAMS] [:TRAIL]
    pub fn parse_response(resp: &str) -> Response {
        // start of trail
        let trail_i = resp.find(" :").unwrap_or(resp.len());
        // end of prefix
        let prefix_i = if &resp[0..1] == ":" {
            resp.find(" ").unwrap() + 1
        } else {
            0
        };
        // end of command
        let cmd_i = match resp[prefix_i..trail_i].find(" ") {
            Some(i) => prefix_i + i,
            None => trail_i
        };
        let prefix = if prefix_i != 0 {
            Some(&resp[1..prefix_i-1])
        } else {
            None
        };
        let cmd = &resp[prefix_i..cmd_i];
        let params = if cmd_i < trail_i {
            Some(&resp[cmd_i+1..trail_i])
        } else {
            None
        };
        let trail = if trail_i < resp.len() {
            Some(&resp[trail_i+2..])
        } else {
            None
        };
        Response{prefix: prefix, command: cmd, params: params, trail: trail}
    }

    // Return User from a prefix like me!www.domain.example
    pub fn prefix_to_user(prefix: &str) -> User {
        let ex = prefix.find("!");
        User{
            nick: if let Some(i) = ex {
                Some(&prefix[0..i])
            } else {
                None
            },
            host: if let Some(i) = ex {
                &prefix[i+1..]
            } else {
                prefix
            }
        }
    }
    // Split user_chan into first (word, rest).
    pub fn split_user_chan(user_chan: &str) -> (&str, &str) {
        let x = user_chan.find(" ").unwrap_or(user_chan.len());
        (&user_chan[..x], &user_chan[x+1..])
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

    // Send string on TcpStream to server.
    fn send_raw(&mut self, raw: &str) -> io::Result<()> {
        print!("-> {}", raw);
        self.stream.write_all(raw.as_bytes())
    }

    // Handle the given action.
    fn dispatch_action(&mut self, action: Action) {
        if let Some(cmd) = action.to_command() {
            if let Err(e) = self.send_raw(cmd.as_slice()) {
                let action = self.listener.on_ioerror(e, &action);
                self.dispatch_action(action);
            }
        }
    }

    // Parse the response string from the server and dispatch to the listener.
    fn dispatch_response(&mut self, resp: &str) {
        println!("<- {}", resp);
        let r = parse::parse_response(resp);
        let action = match r.command {
            "PRIVMSG" => self.listener.on_msg(r.prefix.unwrap_or(""),
                                              r.params.unwrap_or(""),
                                              r.trail.unwrap_or("")),
            "NOTICE" => self.listener.on_notice(r.prefix.unwrap_or(""),
                                                r.params.unwrap_or(""),
                                                r.trail.unwrap_or("")),
            "PING" => self.listener.on_ping(r.trail.unwrap_or("")),
            "JOIN" => self.listener.on_join(r.prefix.unwrap_or(""),
                                            r.params.unwrap_or("")),
            "PART" => self.listener.on_part(r.prefix.unwrap_or(""),
                                            r.params.unwrap_or(""),
                                            r.trail),
            "KICK" => self.listener.on_kick(r.prefix.unwrap_or(""),
                                            r.params.unwrap_or(""),
                                            r.trail),
            "INVITE" => self.listener.on_invite(r.prefix.unwrap_or(""),
                                                r.params.unwrap_or("")),
            "TOPIC" => self.listener.on_topic(r.prefix.unwrap_or(""),
                                              r.params.unwrap_or(""),
                                              r.trail.unwrap_or("")),
            "ERROR" => self.listener.on_error(r.trail.unwrap_or("")),
            _ => self.listener.on_other(r.prefix, r.command, r.params, r.trail),
        };
        self.dispatch_action(action);
    }

    // Read up to 1024 bytes from server into recvbuf
    fn read(&mut self) -> io::Result<usize> {
        self.stream.read(&mut self.recvbuf)
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
            Some(p) => try!(self.send_raw(
                            format!("PASS {} \r\n", p).as_slice())),
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
                for cmd in msg.as_slice().lines_any().filter(|c| !c.is_empty()) {
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

