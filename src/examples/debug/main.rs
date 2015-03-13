extern crate crustirc;

use crustirc::{CrustyBot, Action, CrustyListener, BotInfo};

pub struct DebugListener;

impl CrustyListener for DebugListener {
    fn on_connect(&mut self) -> Action {
        Action::Join(vec![format!("#testing")])
    }
}

fn main() {
    let mut bot = CrustyBot::new(BotInfo{
        nick: "tester",
        hostname: "localhost",
        servername: "crusty.bot",
        realname: "crusty",
    }, "localhost:6667", DebugListener, true).unwrap();
    let _ = bot.connect(None);
    bot.listen();
}
