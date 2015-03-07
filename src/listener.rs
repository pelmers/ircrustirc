use action::Action;
use action::Action::*;
use std::io;

pub trait CrustyListener {
    fn on_connect(&mut self) -> Action {
        NoOp
    }
    fn on_notice(&mut self, source: &str, target: &str, msg: &str) -> Action {
        NoOp
    }
    fn on_msg(&mut self, source: &str, target: &str, msg: &str) -> Action {
        NoOp
    }
    fn on_join(&mut self, source: &str, channel: &str) -> Action {
        NoOp
    }
    fn on_part(&mut self, source: &str, channel: &str, msg: Option<&str>) -> Action {
        NoOp
    }
    fn on_topic(&mut self, source: &str, channel: &str, topic: &str) -> Action {
        NoOp
    }
    fn on_kick(&mut self, source: &str, kicked: &str, reason: Option<&str>) -> Action {
        NoOp
    }
    fn on_ping(&mut self, token: &str) -> Action {
        Pong(token.to_string())
    }
    fn on_error(&mut self, msg: &str) -> Action {
        Quit(Some(format!("Error: {}", msg)))
    }
    fn on_other(&mut self, prefix: Option<&str>, command: &str, params: Option<&str>, trail: Option<&str>) -> Action {
        NoOp
    }
    fn on_ioerror(&mut self, error: io::Error, cause: &Action) -> Action {
        Quit(Some(format!("Error: {:?}", error)))
    }
}

