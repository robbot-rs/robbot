use crate as robbot;
use crate::util::color::Color;
use crate::{Decode, Encode};

use serde::{Deserialize, Serialize};

use super::id::UserId;

#[derive(Clone, Debug, Serialize, Deserialize, Encode, Decode)]
pub struct User {
    pub id: UserId,
    pub avatar: Option<String>,
    pub bot: bool,
    pub discriminator: u16,
    pub name: String,
    pub banner: Option<String>,
    pub accent_color: Option<Color>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Encode, Decode)]
pub enum OnlineStatus {
    Online,
    Idle,
    DoNotDisturb,
    Invisible,
    Offline,
}
