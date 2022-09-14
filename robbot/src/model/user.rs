use crate as robbot;
use crate::arguments::UserMention;
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

impl User {
    /// Creates a new [`UserMention`] of the user.
    ///
    /// # Examples
    ///
    /// ```
    /// # use robbot::model::id::UserId;
    /// # use robbot::model::user::User;
    ///
    /// let user = User {
    ///     id: UserId(12345),
    ///     avatar: None,
    ///     bot: false,
    ///     discriminator: 0,
    ///     name: String::from(""),
    ///     banner: None,
    ///     accent_color: None,
    /// };
    ///
    /// assert_eq!(user.mention().to_string(), "<@12345>");
    /// ```
    #[inline]
    pub fn mention(&self) -> UserMention {
        UserMention::new(self.id)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Encode, Decode)]
pub enum OnlineStatus {
    Online,
    Idle,
    DoNotDisturb,
    Invisible,
    Offline,
}
