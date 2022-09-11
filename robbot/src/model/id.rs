use crate as robbot;
use crate::arguments::{ChannelMention, RoleMention, UserMention};
use crate::{Decode, Encode};

use serde::{Deserialize, Serialize};

use std::fmt::{self, Display, Formatter};

/// A unique identifier for an Attachment.
#[derive(
    Copy,
    Clone,
    Debug,
    Default,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Serialize,
    Deserialize,
    Encode,
    Decode,
)]
pub struct AttachmentId(pub u64);

/// A unique identifier for a Channel.
#[derive(
    Copy,
    Clone,
    Debug,
    Default,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Serialize,
    Deserialize,
    Encode,
    Decode,
)]
pub struct ChannelId(pub u64);

/// A unique identifier for an Emoji.
#[derive(
    Copy,
    Clone,
    Debug,
    Default,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Serialize,
    Deserialize,
    Encode,
    Decode,
)]
pub struct EmojiId(pub u64);

/// A unique identifier for a Guild.
#[derive(
    Copy,
    Clone,
    Debug,
    Default,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Serialize,
    Deserialize,
    Encode,
    Decode,
)]
pub struct GuildId(pub u64);

/// A unique identifier for a Message.
#[derive(
    Copy,
    Clone,
    Debug,
    Default,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Serialize,
    Deserialize,
    Encode,
    Decode,
)]
pub struct MessageId(pub u64);

/// A unique identifier for a Role.
#[derive(
    Copy,
    Clone,
    Debug,
    Default,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Serialize,
    Deserialize,
    Encode,
    Decode,
)]
pub struct RoleId(pub u64);

/// A unique identifier for an User.
#[derive(
    Copy,
    Clone,
    Debug,
    Default,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Serialize,
    Deserialize,
    Encode,
    Decode,
)]
pub struct UserId(pub u64);

macro_rules! impl_id {
    ($t:tt) => {
        impl Display for $t {
            fn fmt(&self, f: &mut Formatter) -> fmt::Result {
                self.0.fmt(f)
            }
        }

        impl From<u64> for $t {
            fn from(id: u64) -> Self {
                Self(id)
            }
        }

        impl From<serenity::model::id::$t> for $t {
            fn from(src: serenity::model::id::$t) -> Self {
                Self(src.0)
            }
        }

        impl From<$t> for serenity::model::id::$t {
            fn from(src: $t) -> serenity::model::id::$t {
                serenity::model::id::$t(src.0)
            }
        }

        impl AsRef<$t> for $t {
            fn as_ref(&self) -> &$t {
                self
            }
        }
    };
}

impl_id!(AttachmentId);
impl_id!(ChannelId);
impl_id!(EmojiId);
impl_id!(GuildId);
impl_id!(MessageId);
impl_id!(RoleId);
impl_id!(UserId);

impl ChannelId {
    /// Creates a [`ChannelMention`] from an [`ChannelId`].
    ///
    /// # Examples
    ///
    /// ```
    /// # fn main() {
    /// # use robbot::model::id::ChannelId;
    /// let channel_id = ChannelId(583806438531661826);
    /// let msg = format!("Mentioning a channel: {}", channel_id.mention());
    /// assert_eq!(msg, "Mentioning a channel: <#583806438531661826>");
    /// # }
    /// ```
    pub fn mention(&self) -> ChannelMention {
        ChannelMention::new(self.0)
    }
}

impl RoleId {
    /// Creates a [`RoleMention`] from an [`RoleId`].
    ///
    /// # Examples
    ///
    /// ```
    /// # fn main() {
    /// # use robbot::model::id::RoleId;
    /// let role_id = RoleId(583816507839217667);
    /// let msg = format!("Mentioning a role: {}", role_id.mention());
    /// assert_eq!(msg, "Mentioning a role: <@&583816507839217667>")
    /// # }
    /// ```
    pub fn mention(&self) -> RoleMention {
        RoleMention::new(self.0)
    }
}

impl UserId {
    /// Creates a [`UserMention`] from an [`UserId`].
    ///
    /// # Examples
    ///
    /// ```
    /// # fn main() {
    /// # use robbot::model::id::UserId;
    /// let user_id = UserId(583818005197357076);
    /// let msg = format!("Mentioning a user: {}", user_id.mention());
    /// assert_eq!(msg, "Mentioning a user: <@583818005197357076>")
    /// # }
    /// ```
    pub fn mention(&self) -> UserMention {
        UserMention::new(self.0)
    }
}
