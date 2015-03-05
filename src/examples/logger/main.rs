extern crate crustybot;

use crustybot::{CrustyBot, Action, CrustyListener, BotInfo};

pub struct EchoListener;

impl CrustyListener for EchoListener {
    fn on_reply(&mut self, msg: String, _: Action) -> Action {
        print!("{}", msg);
        Action::NoOp
    }
    fn on_connect(&mut self) -> Action {
        Action::Join(vec!["#testing".to_string()])
    }
}

fn main() {
    let mut bot = CrustyBot::new(BotInfo{
        nick: "tester",
        hostname: "localhost",
        servername: "crusty.bot",
        realname: "crusty",
    }, "localhost:6667", EchoListener).unwrap();
    let _ = bot.connect(None);
    bot.listen();
}
