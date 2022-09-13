use crate::builder::{CreateMessage, EditMember, EditMessage};
use crate::model::channel::Message;

use crate::model::guild::Member;
use crate::model::id::{ChannelId, GuildId, MessageId, UserId};

use serenity::model::channel::ReactionType;

use async_trait::async_trait;
use thiserror::Error;

#[async_trait]
pub trait ContextOld {
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

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Raw(#[from] serenity::Error),
}

pub struct Context<T>
where
    T: Send + Sync,
{
    raw_ctx: serenity::client::Context,
    event: T,
}

impl<T> Context<T>
where
    T: Send + Sync,
{
    /// Sends a new message in the channel with the given id.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use std::mem::MaybeUninit;
    /// # use robbot::context::Context;
    /// #
    /// use robbot::builder::CreateMessage;
    /// use robbot::model::id::ChannelId;
    ///
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// # let ctx: Context<()> = unsafe { MaybeUninit::uninit().assume_init() };
    /// #
    /// ctx.send_message(ChannelId(1234), CreateMessage::new(|m| {
    ///     m.content("Hello World!");
    /// })).await?;
    /// #
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// ```no_run
    /// # use std::mem::MaybeUninit;
    /// # use robbot::context::Context;
    /// #
    /// use robbot::model::id::ChannelId;
    ///
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// # let ctx: Context<()> = unsafe { MaybeUninit::uninit().assume_init() };
    /// #
    /// ctx.send_message(ChannelId(1234), "Hello World!").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn send_message<M>(&self, channel_id: ChannelId, message: M) -> Result<Message, Error>
    where
        M: Into<CreateMessage>,
    {
        let builder = message.into();

        let msg = serenity::model::id::ChannelId(channel_id.0)
            .send_message(&self.raw_ctx, |m| {
                builder.fill_builder(m);
                m
            })
            .await?;

        Ok(msg.into())
    }

    pub async fn delete_message(
        &self,
        channel_id: ChannelId,
        message_id: MessageId,
    ) -> Result<(), Error> {
        serenity::model::id::ChannelId(channel_id.0)
            .delete_message(&self.raw_ctx, message_id)
            .await?;

        Ok(())
    }

    pub async fn pin_message(
        &self,
        channel_id: ChannelId,
        message_id: MessageId,
    ) -> Result<(), Error> {
        serenity::model::id::ChannelId(channel_id.0)
            .pin(&self.raw_ctx, message_id)
            .await?;

        Ok(())
    }

    pub async fn unpin_message(
        &self,
        channel_id: ChannelId,
        message_id: MessageId,
    ) -> Result<(), Error> {
        serenity::model::id::ChannelId(channel_id.0)
            .unpin(&self.raw_ctx, message_id)
            .await?;

        Ok(())
    }

    pub async fn edit_message<B>(
        &self,
        channel_id: ChannelId,
        message_id: MessageId,
        builder: B,
    ) -> Result<Message, Error>
    where
        B: Into<EditMessage>,
    {
        let builder = builder.into();

        let msg = serenity::model::id::ChannelId(channel_id.0)
            .edit_message(&self.raw_ctx, message_id, |m| {
                builder.fill_builder(m);
                m
            })
            .await?;

        Ok(msg.into())
    }

    pub async fn create_reaction<R>(
        &self,
        message_id: MessageId,
        channel_id: ChannelId,
        reaction: R,
    ) -> Result<(), Error>
    where
        R: Into<ReactionType>,
    {
        let reaction = reaction.into();

        self.raw_ctx
            .http
            .create_reaction(channel_id.0, message_id.0, &reaction)
            .await?;

        Ok(())
    }

    pub async fn member(&self, guild_id: GuildId, user_id: UserId) -> Result<Member, Error> {
        let member = serenity::model::id::GuildId(guild_id.0)
            .member(&self.raw_ctx, user_id)
            .await?;

        Ok(member.into())
    }
}

impl<T> Context<T>
where
    T: Send + Sync + AsRef<ChannelId> + AsRef<MessageId>,
{
    pub async fn respond<M>(&self, message: M) -> Result<Message, Error>
    where
        M: Into<CreateMessage>,
    {
        let channel_id = *self.event.as_ref();
        let message_id = *self.event.as_ref();

        let mut builder = message.into();
        builder.reference_message((channel_id, message_id));

        self.send_message(channel_id, builder).await
    }

    pub async fn react<R>(&self, reaction: R) -> Result<(), Error>
    where
        R: Into<ReactionType>,
    {
        let channel_id = *self.event.as_ref();
        let message_id = *self.event.as_ref();

        self.create_reaction(message_id, channel_id, reaction).await
    }
}

pub struct GuildContext<'a, T>
where
    T: Send + Sync,
{
    ctx: &'a Context<T>,
    guild_id: GuildId,
}

impl<'a, T> GuildContext<'a, T>
where
    T: Send + Sync,
{
    pub fn new(ctx: &'a Context<T>, guild_id: GuildId) -> Self {
        Self { ctx, guild_id }
    }

    pub async fn member(&self, user_id: UserId) -> Result<Member, Error> {
        let member = serenity::model::id::GuildId(self.guild_id.0)
            .member(&self.ctx.raw_ctx, user_id)
            .await?;

        Ok(member.into())
    }

    pub async fn members(
        &self,
        limit: Option<u64>,
        after: Option<UserId>,
    ) -> Result<Vec<Member>, Error> {
        let members = serenity::model::id::GuildId(self.guild_id.0)
            .members(&self.ctx.raw_ctx, limit, after.map(|id| id.into()))
            .await?;

        Ok(members.into_iter().map(|m| m.into()).collect())
    }

    // pub fn members_iter(&self) -> impl Stream<Item = Result<Member, Error>> {}
}
