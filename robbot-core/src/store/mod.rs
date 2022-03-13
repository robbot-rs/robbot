pub mod mem;
pub mod mysql;

use std::error::Error as StdError;
use std::fmt::{self, Display, Formatter};
use std::result;

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub struct Error(Box<dyn StdError + Send + Sync + 'static>);

impl AsRef<dyn StdError + Send + 'static> for Error {
    fn as_ref(&self) -> &(dyn StdError + Send + 'static) {
        &*self.0
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<T> From<T> for Error
where
    T: StdError + Send + Sync + 'static,
{
    fn from(err: T) -> Self {
        Self(Box::new(err))
    }
}

impl From<Error> for robbot::Error {
    fn from(err: Error) -> Self {
        Self::Other(err.0)
    }
}
