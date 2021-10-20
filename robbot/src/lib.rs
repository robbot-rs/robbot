pub mod bot;
pub mod builder;
pub mod context;
pub mod executor;
pub mod hook;
pub mod model;
pub mod task;

pub use robbot_derive::command;
pub use {
    context::Context,
    task::{Task, TaskSchedule},
};

use std::collections::HashSet;

#[derive(Clone, Debug)]
pub struct Command {
    pub name: String,
    pub description: String,
    pub guild_only: bool,
    pub sub_commands: HashSet<Self>,
    // pub executor: Option<CommandExecutor>,
}
