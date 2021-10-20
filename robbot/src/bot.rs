use std::{error, result};

pub type Result = result::Result<(), Error>;

#[derive(Debug)]
pub enum Error {
    InvalidCommandUsage,
    Unimplemented,
    NoResponse,
    BoxError(Box<dyn error::Error + Send + Sync + 'static>),
    /// Hook collector timed out.
    HookTimeout,
}

impl<T> From<T> for Error
where
    T: error::Error + Send + Sync + 'static,
{
    fn from(err: T) -> Self {
        Self::BoxError(Box::new(err))
    }
}

pub trait Context {}
