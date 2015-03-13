use action::Action;
use action::Action::*;
use std::io;
use bot::parse::split_user_chan;

pub trait CrustyListener {
    fn on_connect(&mut self) -> Action {
        NoOp
    }
    fn on_notice(&mut self, _source: &str, _target: &str, _msg: &str) -> Action {
        NoOp
    }
    fn on_msg(&mut self, _source: &str, _target: &str, _msg: &str) -> Action {
        NoOp
    }
    fn on_join(&mut self, _source: &str, _channel: &str) -> Action {
        NoOp
    }
    fn on_part(&mut self, _source: &str, _channel: &str, _msg: Option<&str>) -> Action {
        NoOp
    }
    fn on_topic(&mut self, _source: &str, _channel: &str, _topic: &str) -> Action {
        NoOp
    }
    fn on_kick(&mut self, _source: &str, _kicked: &str, _reason: Option<&str>) -> Action {
        NoOp
    }
    fn on_invite(&mut self, _source: &str, _target: &str) -> Action {
        let (_, chan) = split_user_chan(_target);
        Join(vec![chan.to_string()])
    }
    fn on_ping(&mut self, token: &str) -> Action {
        Pong(token.to_string())
    }
    fn on_error(&mut self, msg: &str) -> Action {
        Quit(Some(format!("Error: {}", msg)))
    }
    fn on_other(&mut self, _prefix: Option<&str>, _command: &str, _params: Option<&str>, _trail: Option<&str>) -> Action {
        NoOp
    }
    fn on_ioerror(&mut self, error: io::Error, _cause: &Action) -> Action {
        Quit(Some(format!("Error: {:?}", error)))
    }
}

