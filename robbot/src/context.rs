use crate::{builder::CreateMessage, model::Message};
use async_trait::async_trait;
use serenity::model::{
    channel::ReactionType,
    id::{ChannelId, MessageId},
};

#[async_trait]
pub trait Context {
    type Error;

    /// Create a new message a text channel.
    async fn send_message<T>(
        &self,
        channel_id: ChannelId,
        message: T,
    ) -> Result<Message, Self::Error>
    where
        T: Into<CreateMessage> + Send + Sync;

    /// Create a new reaction on a message.
    async fn create_reaction<T>(
        &self,
        message_id: MessageId,
        channel_id: ChannelId,
        reaction: T,
    ) -> Result<(), Self::Error>
    where
        T: Into<ReactionType> + Send;

    /// Respond to the message author. Returns the newly created message.
    async fn respond<T>(&self, message: T) -> Result<Message, Self::Error>
    where
        Self: AsRef<MessageId> + AsRef<ChannelId>,
        T: Into<CreateMessage> + Send + Sync,
    {
        let message_id: MessageId = *self.as_ref();
        let channel_id: ChannelId = *self.as_ref();

        let mut message = message.into();
        message.reference_message((channel_id, message_id));

        self.send_message(channel_id, message).await
    }

    /// React to the message the author sent.
    async fn react<T>(&self, reaction: T) -> Result<(), Self::Error>
    where
        Self: AsRef<MessageId> + AsRef<ChannelId>,
        T: Into<ReactionType> + Send,
    {
        let message_id: MessageId = *self.as_ref();
        let channel_id: ChannelId = *self.as_ref();

        self.create_reaction(message_id, channel_id, reaction.into())
            .await
    }
}
