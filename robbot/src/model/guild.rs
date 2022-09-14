use crate as robbot;
use crate::{Decode, Encode};

use super::id::{GuildId, RoleId};
use super::user::User;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, Encode, Decode)]
pub struct Member {
    pub deaf: bool,
    pub guild_id: GuildId,
    pub joined_at: Option<DateTime<Utc>>,
    pub mute: bool,
    pub nick: Option<String>,
    pub roles: Vec<RoleId>,
    pub user: User,
    pub pending: bool,
    pub avatar: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Encode, Decode)]
pub struct PartialMember {
    pub deaf: bool,
    pub joined_at: Option<DateTime<Utc>>,
    pub mute: bool,
    pub nick: Option<String>,
    pub roles: Vec<RoleId>,
    pub pending: bool,
    pub guild_id: Option<GuildId>,
    pub user: Option<User>,
}

impl From<serenity::model::guild::Member> for Member {
    fn from(src: serenity::model::guild::Member) -> Self {
        Self {
            deaf: src.deaf,
            guild_id: src.guild_id.into(),
            joined_at: src.joined_at,
            mute: src.mute,
            nick: src.nick,
            roles: src.roles.into_iter().map(|r| r.into()).collect(),
            user: src.user.into(),
            pending: src.pending,
            avatar: src.avatar,
        }
    }
}
