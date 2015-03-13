pub enum Action {
    NoOp,
    Help,
    MOTD,
    Time,
    Topic(String, Option<String>),
    Names(Option<Vec<String>>),
    Nick(String),
    Part(Vec<String>, Option<String>),
    Ping(String),
    Pong(String),
    Join(Vec<String>),
    Away(String),
    Quit(Option<String>),
    Whois(Vec<String>),
    Setname(String),
    Privmsg(String, String),
    ActionVec(Vec<Action>)
}

// Convert optional string to an argument
fn opt_to_arg(arg: &Option<String>) -> String {
    match *arg {
        Some(ref a) => a.clone(),
        _ => String::new()
    }
}

// Convert vector of strings to comma-separated string.
fn optvec_to_args(vec: &Option<Vec<String>>) -> String {
    if let &Some(ref v) = vec {
        v.connect(",")
    } else {
        String::new()
    }
}

impl Action {
    // Serialze an action into a server command.
    pub fn to_command(&self) -> Option<String> {
        use Action::*;
        match *self {
            Help => Some(format!("HELP\r\n")),
            MOTD => Some(format!("MOTD\r\n")),
            Time => Some(format!("TIME\r\n")),
            Topic(ref ch, ref tp) => Some(format!("TOPIC {} :{}\r\n",
                                                  ch, opt_to_arg(tp))),
            Names(ref chans) => Some(format!("NAMES {}\r\n",
                                             optvec_to_args(&chans))),
            Nick(ref nick) => Some(format!("NICK {}\r\n", nick)),
            Part(ref chans, ref msg) => Some(format!("PART {} :{}\r\n",
                                                     chans.connect(","),
                                                     opt_to_arg(msg))),
            Ping(ref srv) => Some(format!("PING :{}\r\n", srv)),
            Pong(ref srv) => Some(format!("PONG :{}\r\n", srv)),
            Join(ref chans) => Some(format!("JOIN {}\r\n",
                                            chans.connect(","))),
            Away(ref msg) => Some(format!("AWAY :{}\r\n",
                                          msg)),
            Quit(ref msg) => Some(format!("QUIT :{}\r\n",
                                          opt_to_arg(msg))),
            Whois(ref nicks) => Some(format!("WHOIS {}\r\n",
                                             nicks.connect(","))),
            Setname(ref name) => Some(format!("SETNAME :{}\r\n", name)),
            Privmsg(ref target, ref msg) => Some(format!("PRIVMSG {} :{}\r\n",
                                                         target, msg)),
            ActionVec(ref acts) => {
                let cmds = acts.iter().map(|x| x.to_command().unwrap_or(String::new()));
                Some(cmds.collect::<Vec<_>>().connect(""))
            },
            _ => None,
        }
    }
}
