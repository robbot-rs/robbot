use crate::state::State;
use async_trait::async_trait;
use robbot::{
    arguments::{CommandArguments, OwnedArguments},
    builder::CreateMessage,
    context::Context as ContextExt,
    model::Message,
};
use serenity::{
    client::Context as RawContext,
    model::{
        channel::ReactionType,
        id::{ChannelId, MessageId},
    },
};
use std::sync::Arc;

pub type MessageContext = Context<Message>;

#[derive(Clone)]
pub struct Context<T> {
    pub raw_ctx: RawContext,
    pub state: Arc<State>,
    pub args: CommandArguments,
    pub event: T,
}

impl<T> Context<T> {
    pub fn new(raw_ctx: RawContext, state: Arc<State>, event: T) -> Self {
        Self {
            raw_ctx,
            state,
            args: CommandArguments::new(OwnedArguments::new()),
            event,
        }
    }
}

#[async_trait]
impl<T> ContextExt for Context<T>
where
    T: Send + Sync,
{
    type Error = serenity::Error;

    async fn send_message<S>(
        &self,
        channel_id: ChannelId,
        message: S,
    ) -> Result<Message, Self::Error>
    where
        S: Into<CreateMessage> + Send,
    {
        let message = channel_id
            .send_message(&self.raw_ctx, |m| {
                message.into().fill_builder(m);
                m
            })
            .await?;
        Ok(message.into())
    }

    async fn create_reaction<S>(
        &self,
        message_id: MessageId,
        channel_id: ChannelId,
        reaction: S,
    ) -> Result<(), Self::Error>
    where
        S: Into<ReactionType> + Send,
    {
        self.raw_ctx
            .http
            .create_reaction(channel_id.0, message_id.0, &reaction.into())
            .await?;
        Ok(())
    }
}

impl<T> AsRef<ChannelId> for Context<T>
where
    T: AsRef<ChannelId>,
{
    fn as_ref(&self) -> &ChannelId {
        self.event.as_ref()
    }
}

impl<T> AsRef<MessageId> for Context<T>
where
    T: AsRef<MessageId>,
{
    fn as_ref(&self) -> &MessageId {
        self.event.as_ref()
    }
}
