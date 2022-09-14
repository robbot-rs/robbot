mod message;

use crate::command::SubCommand;

use robbot::arguments::OwnedArguments;

use std::error::Error as StdError;

#[derive(Debug)]
pub enum Error {
    UnknownCommand(OwnedArguments),
    InvalidCommandUsage(SubCommand, OwnedArguments),
    GuildOnly,
    #[cfg(feature = "permissions")]
    NoPermission,
    Unknown,
    Other(Box<dyn StdError + Sync + Send>),
}

// impl<T> From<T> for Error
// where
//     T: StdError + Sync + Send + 'static,
// {
//     fn from(err: T) -> Self {
//         Self::Other(Box::new(err))
//     }
// }

// impl<T> From<T> for Error
// where
//     T: AsRef<dyn StdError + Sync + Send + 'static> + 'static,
// {
//     fn from(err: T) -> Self {
//         let boxed = err.as_ref();

//         Self::Other(boxed)
//     }
// }
