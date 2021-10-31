use std::{
    error,
    fmt::{self, Display, Formatter},
    result,
};

pub type Result = result::Result<(), Error>;

#[derive(Debug)]
pub enum Error {
    InvalidCommandUsage,
    Unimplemented,
    NoResponse,
    /// Hook collector timed out.
    HookTimeout,
    BoxError(Box<dyn error::Error + Send + Sync + 'static>),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::InvalidCommandUsage => write!(f, "invalid command usage"),
            Self::Unimplemented => write!(f, "unimplemented"),
            Self::NoResponse => write!(f, "no response"),
            Self::HookTimeout => write!(f, "hook timeout"),
            Self::BoxError(err) => err.fmt(f),
        }
    }
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
