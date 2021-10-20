use crate::model::Message;
use serenity::model::{
    channel::{GuildChannel, Reaction},
    guild::Member,
    id::{ChannelId, GuildId, MessageId},
    user::User,
};
use std::{
    borrow::Borrow,
    convert::TryFrom,
    hash::{Hash, Hasher},
};

#[derive(Clone, Debug)]
pub struct Hook {
    pub name: String,
    pub on_event: EventKind,
}

impl Hash for Hook {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        self.name.hash(state);
    }
}

impl PartialEq for Hook {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for Hook {}

impl Borrow<str> for Hook {
    fn borrow(&self) -> &str {
        &self.name
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum EventKind {
    ChannelCreate,
    ChannelDelete,
    GuildMemberAddition,
    GuildMemberRemoval,
    GuildMemberUpdate,
    Message,
    ReactionAdd,
    ReactionRemove,
    ReactionRemoveAll,
}

pub struct InvalidEventKindError;

impl TryFrom<&str> for EventKind {
    type Error = InvalidEventKindError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s {
            "ChannelCreate" => Ok(Self::ChannelCreate),
            "ChannelDelete" => Ok(Self::ChannelDelete),
            "GuildMemberAddition" => Ok(Self::GuildMemberAddition),
            "GuildMemberRemoval" => Ok(Self::GuildMemberRemoval),
            "GuildMemberUpdate" => Ok(Self::GuildMemberUpdate),
            "Message" => Ok(Self::Message),
            "ReactionAdd" => Ok(Self::ReactionAdd),
            "ReactionRemove" => Ok(Self::ReactionRemove),
            "ReactionRemoveAll" => Ok(Self::ReactionRemoveAll),
            _ => Err(InvalidEventKindError),
        }
    }
}

pub enum EventData {
    ChannelCreate(Box<ChannelCreateData>),
    ChannelDelete(Box<ChannelDeleteData>),
    GuildMemberAddition(Box<GuildMemberAdditionData>),
    GuildMemberRemoval(Box<GuildMemberRemovalData>),
    GuildMemberUpdate(Box<GuildMemberUpdateData>),
    Message(Box<MessageData>),
    ReactionAdd(Box<ReactionAddData>),
    ReactionRemove(Box<ReactionRemoveData>),
    ReactionRemoveAll(Box<ReactionRemoveAllData>),
}

pub struct ChannelCreateData(pub GuildChannel);

pub struct ChannelDeleteData(pub GuildChannel);

pub struct GuildMemberAdditionData {
    pub guild_id: GuildId,
    pub member: Member,
}

pub struct GuildMemberRemovalData {
    pub guild_id: GuildId,
    pub user: User,
    pub member: Option<Member>,
}

pub struct GuildMemberUpdateData {
    pub old_member: Option<Member>,
    pub member: Member,
}

pub struct MessageData(pub Message);

pub struct ReactionAddData(pub Reaction);

pub struct ReactionRemoveData(pub Reaction);

pub struct ReactionRemoveAllData {
    pub channel_id: ChannelId,
    pub message_id: MessageId,
}
