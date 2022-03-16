use crate as robbot;
use crate::{Decode, Encode};

use super::id::{GuildId, RoleId};
use super::user::User;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

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
