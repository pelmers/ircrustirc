#![feature(net)]
#![feature(io)]

mod action;
mod listener;
mod bot;
pub use bot::{CrustyBot, BotInfo};
pub use listener::CrustyListener;
pub use action::{Action};
pub use bot::parse::prefix_to_user;

