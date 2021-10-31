pub mod arguments;
pub mod bot;
pub mod builder;
pub mod command;
pub mod context;
pub mod executor;
pub mod hook;
pub mod model;
pub mod task;

pub use robbot_derive::command;
pub use {
    arguments::Arguments,
    bot::{Error, Result},
    command::Command,
    context::Context,
    task::{Task, TaskSchedule},
};

pub mod prelude {
    pub use crate::{
        bot::{
            Error::{self, InvalidCommandUsage},
            Result,
        },
        command::Command,
        context::Context,
    };
    pub use robbot_derive::command;
}
