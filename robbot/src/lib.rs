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

pub use crate::arguments::Arguments;
pub use crate::bot::{Error, Result};
pub use crate::command::Command;
pub use crate::context::Context;
pub use crate::store::StoreData;
pub use crate::task::{Task, TaskSchedule};

pub use robbot_derive::{command, hook, module, Decode, Encode, StoreData};

pub mod prelude {
    pub use crate::arguments::ArgumentsExt;
    pub use crate::bot::Error::InvalidCommandUsage;
    pub use crate::bot::{Error, Result};
    pub use crate::command::Command;
    pub use crate::context::Context;
    pub use crate::store::StoreData;

    pub use robbot_derive::{command, hook, module, StoreData};
}
