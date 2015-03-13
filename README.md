ircrustirc: simple IRC client library
=====================================

ircrustirc provides a simple event-listener model for writing IRC client code.
You write a client listener and the library handles parsing and sending IRC
commands and calling back to your handlers appropriately. The listener then
responds to events with an Action.

Example below lives in examples/debug. Setup a local IRC server and run it with
`cargo run`.

```rust
extern crate ircrustirc;

use ircrustirc::{CrustyBot, Action, CrustyListener, BotInfo};

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
        servername: "testarena",
        realname: "crusty",
    }, "localhost:6667", DebugListener, true).unwrap();
    let _ = bot.connect(None);
    bot.listen();
}
```
