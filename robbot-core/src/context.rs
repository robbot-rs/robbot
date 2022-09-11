use crate::state::State;
use async_trait::async_trait;
use robbot::{
    arguments::{CommandArguments, OwnedArguments},
    builder::CreateMessage,
    context::Context as ContextExt,
};
use serenity::{client::Context as RawContext, model::channel::ReactionType};
use std::sync::Arc;

use robbot::builder::EditMember;
use robbot::model::channel::Message;
use robbot::model::guild::Member;
use robbot::model::id::{ChannelId, GuildId, MessageId, UserId};

use serenity::utils::hashmap_to_json_map;

use robbot::hook::{HookEvent, HookEventWrapper};

use std::fmt::{self, Debug, Formatter};

/// An alias for `Context<Message>`. This context is received by
/// command handlers.
pub type MessageContext = Context<Message>;

/// An alias for `Context<()>`. This context is received by tasks.
pub type TaskContext = Context<()>;

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
            args: CommandArguments::from(OwnedArguments::new()),
            event,
        }
    }
}

impl<T> HookEventWrapper for Context<T>
where
    T: HookEvent,
{
    type HookEvent = T;
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
        let message = serenity::model::id::ChannelId(channel_id.0)
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

    async fn edit_member<S>(
        &self,
        guild_id: GuildId,
        user_id: UserId,
        edit_member: S,
    ) -> Result<Member, Self::Error>
    where
        S: Into<EditMember> + Send,
    {
        let mut builder = serenity::builder::EditMember::default();
        edit_member.into().fill_builder(&mut builder);

        let member = self
            .raw_ctx
            .http
            .edit_member(guild_id.0, user_id.0, &hashmap_to_json_map(builder.0))
            .await;

        member.map(|m| m.into())
    }

    async fn get_member(&self, guild_id: GuildId, user_id: UserId) -> Result<Member, Self::Error> {
        let member = self.raw_ctx.http.get_member(guild_id.0, user_id.0).await;

        member.map(|m| m.into())
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

impl<T> Debug for Context<T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Context {{ raw_ctx: ???, state: {:?}, args: {:?}, event: {:?} }}",
            self.state, self.args, self.event
        )
    }
}
