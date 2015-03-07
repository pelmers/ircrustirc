use action::Action;
use action::Action::*;
use std::io;

pub trait CrustyListener {
    fn on_connect(&mut self) -> Action {
        NoOp
    }
    fn on_help(&mut self, help_msg: &str) -> Action {
        NoOp
    }
    fn on_msg(&mut self, user: &str, target: &str, msg: &str) -> Action {
        NoOp
    }
    fn on_join(&mut self, user: &str, channel: &str) -> Action {
        NoOp
    }
    fn on_part(&mut self, user: &str, channel: &str) -> Action {
        NoOp
    }
    fn on_topic(&mut self, channel: &str, topic: &str) -> Action {
        NoOp
    }
    fn on_ping(&mut self, server: &str) -> Action {
        Pong(server.to_string())
    }
    fn on_other(&mut self, prefix: Option<&str>, command: &str, params: Option<&str>, trail: Option<&str>) -> Action {
        NoOp
    }
    fn on_reply(&mut self, message: &str, cause: Action) -> Action {
        NoOp
    }
    fn on_ioerror(&mut self, error: io::Error, cause: &Action) -> Action {
        NoOp
    }
}

