use crate::core::state::State;
use serenity::model::channel::Message;
use std::convert::From;
use std::error;
use std::sync::Arc;

#[derive(Debug)]
pub enum Error {
    InvalidCommandUsage,
    Unimplemented,
    /// Indicates that the executor dropped before
    /// sending a response. This likely means that
    /// the executing thread panicked.
    NoResponse,
    BoxError(Box<dyn error::Error + Send + Sync + 'static>),
}

impl<T> From<T> for Error
where
    T: error::Error + Send + Sync + 'static,
{
    fn from(err: T) -> Self {
        Self::BoxError(Box::new(err))
    }
}

pub type Result = std::result::Result<(), Error>;

// Context aliases.
pub type MessageContext = Context<Message>;
pub type TaskContext = Context<()>;

#[derive(Clone)]
pub struct Context<T> {
    pub raw_ctx: serenity::client::Context,
    pub state: Arc<State>,
    pub args: Vec<String>,
    pub event: T,
}

// impl Context<Message> {
//     fn respond<T>(&self, s: &str) -> serenity::Result<()> {}
// }

pub mod prelude {
    pub use crate::bot::{self, Error::InvalidCommandUsage, MessageContext, TaskContext};
}
