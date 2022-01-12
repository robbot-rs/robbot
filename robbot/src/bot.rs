use std::error::Error as StdError;
use std::result;

/// A type alias for `Result<(), Error>`.
pub type Result = result::Result<(), Error>;

#[derive(Debug)]
pub enum Error {
    InvalidCommandUsage,
    Unimplemented,
    NoResponse,
    /// Hook collector timed out.
    HookTimeout,

    Other(Box<dyn StdError + Send + Sync + 'static>),
}

impl<T> From<T> for Error
where
    T: StdError + Send + Sync + 'static,
{
    fn from(err: T) -> Self {
        Self::Other(Box::new(err))
    }
}
