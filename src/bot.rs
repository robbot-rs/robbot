use crate::{
    core::{
        hook::{Event, EventKind, Hook},
        state::State,
    },
    model::{GuildMessage, Message},
};
use serenity::model::{
    channel::{GuildChannel, Reaction},
    guild::Member,
    id::{ChannelId, GuildId, MessageId},
    user::User,
};
use std::borrow::Borrow;
use std::convert::From;
use std::error;
use std::sync::Arc;
use std::time::Duration;
use tokio::{select, time};

#[derive(Debug)]
pub enum Error {
    InvalidCommandUsage,
    Unimplemented,
    /// Indicates that the executor dropped before
    /// sending a response. This likely means that
    /// the executing thread panicked.
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

pub type Result = std::result::Result<(), Error>;

// Context aliases.
pub type ChannelCreateContext = Context<GuildChannel>;
pub type ChannelDeleteContext = Context<GuildChannel>;
pub type GuildMemberAdditionContext = Context<(GuildId, Member)>;
pub type GuildmemberRemovalContext = Context<(GuildId, User, Option<Member>)>;
pub type GuildMemberUpdateContext = Context<(Option<Member>, Member)>;
pub type MessageContext = Context<Message>;
pub type GuildMessageContext = Context<GuildMessage>;
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

    /// Create a new hook.
    pub async fn create_hook(
        &self,
        event_kind: EventKind,
    ) -> tokio::sync::broadcast::Receiver<Event> {
        let hook_id = self
            .state
            .hook_id
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);

        let hook = Hook {
            name: hook_id.to_string(),
            on_event: event_kind,
        };

        self.state.add_hook(hook).await
    }
}

impl<T> Context<T>
where
    T: Borrow<MessageId> + Borrow<ChannelId>,
{
    pub async fn respond(&self, content: impl std::fmt::Display) -> std::result::Result<(), Error> {
        let message_id: MessageId = *self.event.borrow();
        let channel_id: ChannelId = *self.event.borrow();

        channel_id
            .send_message(&self.raw_ctx, |m| {
                m.reference_message((channel_id, message_id));
                m.content(content);
                m
            })
            .await?;
        Ok(())
    }
}

impl Context<Message> {
    /// Wait for the same author to send another message in the same
    /// channel. Returns the new message. Returns [`Error::HookTimeout`]
    /// if the author doesn't respond in time.
    pub async fn await_message(&self, timeout: Duration) -> std::result::Result<Message, Error> {
        let mut rx = self.create_hook(EventKind::Message).await;

        loop {
            select! {
                event = rx.recv() => {
                    let event = match event.unwrap() {
                        Event::Message(ctx) => ctx.event,
                        _ => unreachable!(),
                    };

                    if self.event.channel_id == event.channel_id && self.event.author.id == event.author.id {
                        return Ok(event)
                    }
                }
                _ = time::sleep(timeout) => return Err(Error::HookTimeout),
            }
        }
    }

    /// Wait for the same author to react to the message. Returns the
    /// new reaction. Returns [`Error::HookTimeout`] if the author doesn't
    /// respond in time.
    pub async fn await_reaction(&self, timeout: Duration) -> std::result::Result<Reaction, Error> {
        let mut rx = self.create_hook(EventKind::ReactionAdd).await;

        loop {
            select! {
                event = rx.recv() => {
                    // Unwrap Event enum
                    let event = match event.unwrap() {
                        Event::ReactionAdd(ctx) => ctx.event,
                        _ => unreachable!(),
                    };

                    if let Some(user_id) = event.user_id {
                        if self.event.id == event.message_id && self.event.author.id == user_id {
                            return Ok(event);
                        }
                    }
                }
                _ = time::sleep(timeout) => return Err(Error::HookTimeout),
            }
        }
    }
}

pub mod prelude {
    pub use crate::bot::{self, Error::InvalidCommandUsage, MessageContext, TaskContext};
}
