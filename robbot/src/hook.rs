use crate::model::channel::Message;

use serenity::model::{
    channel::{GuildChannel, Reaction},
    guild::Member,
    id::{ChannelId, GuildId, MessageId},
    user::User,
};

use std::borrow::Borrow;
use std::convert::TryFrom;
use std::hash::{Hash, Hasher};
use std::str::FromStr;

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

impl FromStr for EventKind {
    type Err = InvalidEventKindError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
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

impl std::fmt::Display for EventKind {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::ChannelCreate => "ChannelCreate",
                Self::ChannelDelete => "ChannelDelete",
                Self::GuildMemberAddition => "GuildMemberAddition",
                Self::GuildMemberRemoval => "GuildMemberRemoval",
                Self::GuildMemberUpdate => "GuildMemberUpdate",
                Self::Message => "Message",
                Self::ReactionAdd => "ReactionAdd",
                Self::ReactionRemove => "ReactionRemove",
                Self::ReactionRemoveAll => "ReactionRemoveAll",
            }
        )
    }
}

pub trait HookEvent: TryFrom<EventData, Error = InvalidEventKindError> + Into<EventData> {
    /// Returns the appropriate [`EventKind`] of the `HookEvent`.
    fn kind() -> EventKind;
}

pub trait HookEventWrapper {
    type HookEvent: HookEvent;
}

#[derive(Clone, Debug)]
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

impl EventData {
    pub fn kind(&self) -> EventKind {
        match self {
            Self::ChannelCreate(_) => EventKind::ChannelCreate,
            Self::ChannelDelete(_) => EventKind::ChannelDelete,
            Self::GuildMemberAddition(_) => EventKind::GuildMemberAddition,
            Self::GuildMemberRemoval(_) => EventKind::GuildMemberRemoval,
            Self::GuildMemberUpdate(_) => EventKind::GuildMemberUpdate,
            Self::Message(_) => EventKind::Message,
            Self::ReactionAdd(_) => EventKind::ReactionAdd,
            Self::ReactionRemove(_) => EventKind::ReactionRemove,
            Self::ReactionRemoveAll(_) => EventKind::ReactionRemoveAll,
        }
    }
}

#[derive(Clone, Debug)]
pub struct ChannelCreateData(pub GuildChannel);

#[derive(Clone, Debug)]
pub struct ChannelDeleteData(pub GuildChannel);

#[derive(Clone, Debug)]
pub struct GuildMemberAdditionData {
    pub guild_id: GuildId,
    pub member: Member,
}

#[derive(Clone, Debug)]
pub struct GuildMemberRemovalData {
    pub guild_id: GuildId,
    pub user: User,
    pub member: Option<Member>,
}

#[derive(Clone, Debug)]
pub struct GuildMemberUpdateData {
    pub old_member: Option<Member>,
    pub member: Member,
}

#[derive(Clone, Debug)]
pub struct MessageData(pub Message);

#[derive(Clone, Debug)]
pub struct ReactionAddData(pub Reaction);

#[derive(Clone, Debug)]
pub struct ReactionRemoveData(pub Reaction);

#[derive(Clone, Debug)]
pub struct ReactionRemoveAllData {
    pub channel_id: ChannelId,
    pub message_id: MessageId,
}

macro_rules! impl_hookevent {
    ($struct_name:ty, $event_name:tt) => {
        impl TryFrom<EventData> for $struct_name {
            type Error = InvalidEventKindError;

            fn try_from(event: EventData) -> Result<Self, Self::Error> {
                if event.kind() == Self::kind() {
                    match event {
                        EventData::$event_name(event) => Ok(*event),
                        _ => unreachable!(),
                    }
                } else {
                    Err(InvalidEventKindError)
                }
            }
        }

        impl From<$struct_name> for EventData {
            fn from(event: $struct_name) -> Self {
                Self::$event_name(Box::new(event))
            }
        }

        impl HookEvent for $struct_name {
            fn kind() -> EventKind {
                EventKind::$event_name
            }
        }
    };
}

impl_hookevent!(GuildMemberAdditionData, GuildMemberAddition);
impl_hookevent!(GuildMemberRemovalData, GuildMemberRemoval);
impl_hookevent!(GuildMemberUpdateData, GuildMemberUpdate);
impl_hookevent!(MessageData, Message);
