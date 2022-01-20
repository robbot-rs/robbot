pub mod arguments;
pub mod bot;
pub mod builder;
pub mod command;
pub mod context;
pub mod executor;
pub mod hook;
pub mod model;
pub mod remote;
pub mod store;
pub mod task;

pub use robbot_derive::{command, Decode, Encode, StoreData};
pub use {
    arguments::Arguments,
    bot::{Error, Result},
    command::Command,
    context::Context,
    store::StoreData,
    task::{Task, TaskSchedule},
};

pub mod prelude {
    pub use crate::{
        bot::{Error, Error::InvalidCommandUsage, Result},
        command::Command,
        context::Context,
        store::StoreData,
    };
    pub use robbot_derive::{command, StoreData};
}
