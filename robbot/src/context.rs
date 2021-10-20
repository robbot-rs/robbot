use crate::{builder::CreateMessage, model::Message};
use async_trait::async_trait;
use serenity::model::id::{ChannelId, MessageId};

#[async_trait]
pub trait Context: Sync {
    type Error;

    async fn send_message<T>(
        &self,
        channel_id: ChannelId,
        message: T,
    ) -> Result<Message, Self::Error>
    where
        T: Into<CreateMessage> + Send + Sync;

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
}
