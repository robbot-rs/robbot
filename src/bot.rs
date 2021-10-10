use crate::core::state::State;
use serenity::model::{
    channel::{GuildChannel, Message, Reaction},
    guild::Member,
    id::{ChannelId, GuildId, MessageId},
    user::User,
};
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
pub type ChannelCreateContext = Context<GuildChannel>;
pub type ChannelDeleteContext = Context<GuildChannel>;
pub type GuildMemberAdditionContext = Context<(GuildId, Member)>;
pub type GuildmemberRemovalContext = Context<(GuildId, User, Option<Member>)>;
pub type GuildMemberUpdateContext = Context<(Option<Member>, Member)>;
pub type MessageContext = Context<Message>;
pub type ReactionAddContext = Context<Reaction>;
pub type ReactionRemoveContext = Context<Reaction>;
pub type ReactionRemoveAllContext = Context<(ChannelId, MessageId)>;

pub type TaskContext = Context<()>;

#[derive(Clone)]
pub struct Context<T> {
    pub raw_ctx: serenity::client::Context,
    pub state: Arc<State>,
    pub args: Vec<String>,
    pub event: T,
}

impl<T> Context<T> {
    pub fn new(raw_ctx: serenity::client::Context, state: Arc<State>, event: T) -> Self {
        Self {
            raw_ctx,
            state,
            args: Vec::new(),
            event,
        }
    }
}

// impl Context<Message> {
//     fn respond<T>(&self, s: &str) -> serenity::Result<()> {}
// }

pub mod prelude {
    pub use crate::bot::{self, Error::InvalidCommandUsage, MessageContext, TaskContext};
}
