extern crate collections;

use listener::CrustyListener;
use std::io;
use std::io::{Read, Write};
use std::net::{TcpStream, SocketAddr, ToSocketAddrs};

pub enum Action {
    NoOp,
    Help,
    MOTD,
    Time,
    Topic(String, Option<String>),
    Names(Option<Vec<String>>),
    Nick(String),
    Part(String),
    Ping(String),
    Join(Vec<String>),
    Pong(String),
    Away(String),
    Quit(Option<String>),
    Whois(Vec<String>),
    Setname(String),
    Privmsg(String, String),
    ActionChain(Vec<Action>)
}

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

    // Read up to 1024 bytes from server into recvbuf
    fn read(&mut self) -> io::Result<usize> {
        self.stream.read(self.recvbuf.as_mut_slice())
    }

    // Perform the given action.
    fn dispatch_action(&mut self, action: Action) {
        let reply_needed = match action {
            Action::Join(ref channels) => {
                for ch in channels {
                    self.join(ch.as_slice());
                }
                true
            },
            _ => false
        };
        if reply_needed {
            if let Ok(msg) = self.try_receive() {
                let action = self.listener.on_reply(msg, action);
                self.dispatch_action(action);
            }
        }
    }

    // Parse the response string from the server and dispatch to the listener.
    fn dispatch_response(&mut self, resp: String) {
        println!("{}", resp);
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

    fn try_receive(&mut self) -> Result<String, collections::string::FromUtf8Error> {
        String::from_utf8(self.receive().unwrap_or(Vec::new()))
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

    // Try to join the specified channel.
    fn join(&mut self, channel: &str) -> io::Result<bool> {
        let joincmd = format!("JOIN {}\r\n", channel);
        try!(self.send_raw(joincmd.as_slice()));
        Ok(true)
    }

    // Leave the current channel.
    pub fn part(&mut self) -> io::Result<()> {
        Ok(())
    }

    // Listen to the server once and dispatch response.
    fn listen_once(&mut self) {
        if let Ok(msg) = self.try_receive() {
            let lines = msg.as_slice().split_str('\n');
            for cmd in lines {
                self.dispatch_response(cmd.to_string());
            }
        }
    }

    // Listen in this channel forever, process each command received.
    pub fn listen(&mut self) {
        loop {
            self.listen_once();
        }
    }
}

