use std::{error, result};

/// A type alias for `Result<(), Error>`.
pub type Result = result::Result<(), Error>;

#[derive(Debug)]
pub enum Error {
    InvalidCommandUsage,
    Unimplemented,
    NoResponse,
    /// Hook collector timed out.
    HookTimeout,

    Other(Box<dyn error::Error + Send + Sync + 'static>),
}

impl<T> From<T> for Error
where
    T: error::Error + Send + Sync + 'static,
{
    fn from(err: T) -> Self {
        Self::Other(Box::new(err))
    }
}
