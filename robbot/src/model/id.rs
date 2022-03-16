use crate as robbot;
use crate::{Decode, Encode};

use serde::{Deserialize, Serialize};

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
    };
}

impl_id!(AttachmentId);
impl_id!(ChannelId);
impl_id!(EmojiId);
impl_id!(GuildId);
impl_id!(MessageId);
impl_id!(RoleId);
impl_id!(UserId);
