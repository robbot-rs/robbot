use crate as robbot;
use crate::{Decode, Encode};

use serde::{Deserialize, Serialize};

use serenity::model::user::User as SUser;

use super::id::UserId;

#[derive(Clone, Debug, Serialize, Deserialize, Encode, Decode)]
pub struct User {
    pub id: UserId,
    pub avatar: Option<String>,
    pub bot: bool,
    pub discriminator: u16,
    #[serde(rename = "username")]
    pub name: String,
    // pub public_flags: Option<UserPublicFlags>,
}

impl From<SUser> for User {
    fn from(src: SUser) -> Self {
        Self {
            id: src.id.into(),
            avatar: src.avatar,
            bot: src.bot,
            discriminator: src.discriminator,
            name: src.name,
        }
    }
}
