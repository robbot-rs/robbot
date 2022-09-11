use crate::builder::{CreateMessage, EditMember};
use crate::model::channel::Message;

use crate::model::guild::Member;
use crate::model::id::{ChannelId, GuildId, MessageId, UserId};

use serenity::model::channel::ReactionType;

use async_trait::async_trait;

#[async_trait]
pub trait Context {
    type Error;

    // CHANNEL

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

    // GUILD

    /// Modifies properties of a guild member. Only modified
    /// properties are changed.
    async fn edit_member<T>(
        &self,
        guild_id: GuildId,
        user_id: UserId,
        edit_member: T,
    ) -> Result<Member, Self::Error>
    where
        T: Into<EditMember> + Send;

    // CHANNEL

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

    async fn get_member(&self, guild_id: GuildId, user_id: UserId) -> Result<Member, Self::Error>;
}
