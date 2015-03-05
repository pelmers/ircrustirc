use bot::Action;
use bot::Action::*;

pub trait CrustyListener {
    fn on_connect(&mut self) -> Action {
        NoOp
    }
    fn on_help(&mut self, help_msg: String) -> Action {
        NoOp
    }
    fn on_msg(&mut self, user: String, target: String, msg: String) -> Action {
        NoOp
    }
    fn on_join(&mut self, user: String, channel: String) -> Action {
        NoOp
    }
    fn on_part(&mut self, user: String, channel: String) -> Action {
        NoOp
    }
    fn on_topic(&mut self, channel: String, topic: String) -> Action {
        NoOp
    }
    fn on_ping(&mut self, server: String) -> Action {
        Pong(server)
    }
    fn on_reply(&mut self, message: String, cause: Action) -> Action {
        NoOp
    }
}

