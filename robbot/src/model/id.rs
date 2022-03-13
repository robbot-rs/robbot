use crate as robbot;
use crate::{Decode, Encode};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use std::num::ParseIntError;
use std::str::FromStr;

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum ParseError {
    #[error("invalid format")]
    InvalidFormat,
    #[error(transparent)]
    ParseIntError(#[from] ParseIntError),
}

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

impl FromStr for ChannelId {
    type Err = ParseError;

    fn from_str(mut s: &str) -> Result<Self, Self::Err> {
        if !s.starts_with("<#") || !s.ends_with('>') {
            return Err(ParseError::InvalidFormat);
        }
        s = s.strip_prefix("<#").unwrap().strip_suffix('>').unwrap();

        let id = s.parse()?;
        Ok(Self(id))
    }
}

impl From<u64> for ChannelId {
    fn from(id: u64) -> ChannelId {
        Self(id)
    }
}

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

impl FromStr for RoleId {
    type Err = ParseError;

    fn from_str(mut s: &str) -> Result<Self, Self::Err> {
        if !s.starts_with("<&") || !s.ends_with('>') {
            return Err(ParseError::InvalidFormat);
        }
        s = s.strip_prefix("<&").unwrap().strip_suffix('>').unwrap();

        let id = s.parse()?;
        Ok(Self(id))
    }
}

impl From<u64> for RoleId {
    fn from(id: u64) -> Self {
        Self(id)
    }
}

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

impl FromStr for UserId {
    type Err = ParseError;

    fn from_str(mut s: &str) -> Result<Self, Self::Err> {
        if !s.starts_with("<@") || !s.ends_with('>') {
            return Err(ParseError::InvalidFormat);
        }
        s = s.strip_prefix("<@").unwrap().strip_suffix('>').unwrap();
        s = match s.strip_prefix('!') {
            Some(s) => s,
            None => s,
        };

        let id = s.parse()?;
        Ok(Self(id))
    }
}

impl From<u64> for UserId {
    fn from(id: u64) -> Self {
        Self(id)
    }
}

impl From<serenity::model::id::ChannelId> for ChannelId {
    fn from(src: serenity::model::id::ChannelId) -> Self {
        Self(src.0)
    }
}

impl From<serenity::model::id::EmojiId> for EmojiId {
    fn from(src: serenity::model::id::EmojiId) -> Self {
        Self(src.0)
    }
}

impl From<serenity::model::id::GuildId> for GuildId {
    fn from(src: serenity::model::id::GuildId) -> Self {
        Self(src.0)
    }
}

impl From<serenity::model::id::MessageId> for MessageId {
    fn from(src: serenity::model::id::MessageId) -> Self {
        Self(src.0)
    }
}

impl From<serenity::model::id::RoleId> for ChannelId {
    fn from(src: serenity::model::id::RoleId) -> Self {
        Self(src.0)
    }
}

impl From<serenity::model::id::UserId> for UserId {
    fn from(src: serenity::model::id::UserId) -> Self {
        Self(src.0)
    }
}

#[cfg(test)]
mod tests {
    use super::{ChannelId, ParseError, RoleId, UserId};

    #[test]
    fn test_parse_channel_id() {
        let channel = "<#904569845>";
        let channel_id: ChannelId = channel.parse().unwrap();
        assert_eq!(channel_id, ChannelId(904569845));

        let channel = "<1231235234534>";
        let err = channel.parse::<ChannelId>().unwrap_err();
        assert_eq!(err, ParseError::InvalidFormat);

        let channel = "<#1ad32424>";
        let left_err = channel.parse::<ChannelId>().unwrap_err();
        let right_err = "1ad32424".parse::<u64>().unwrap_err();
        assert_eq!(left_err, ParseError::ParseIntError(right_err));
    }

    #[test]
    fn test_parse_role_id() {
        let role = "<&904569845>";
        let role_id: RoleId = role.parse().unwrap();
        assert_eq!(role_id, RoleId(904569845));

        let role = "<1231235234534>";
        let err = role.parse::<RoleId>().unwrap_err();
        assert_eq!(err, ParseError::InvalidFormat);

        let role = "<&1ad32424>";
        let left_err = role.parse::<RoleId>().unwrap_err();
        let right_err = "1ad32424".parse::<u64>().unwrap_err();
        assert_eq!(left_err, ParseError::ParseIntError(right_err));
    }

    #[test]
    fn test_parse_user_id() {
        let user = "<@904569845>";
        let user_id: UserId = user.parse().unwrap();
        assert_eq!(user_id, UserId(904569845));

        let user = "<1231235234534>";
        let err = user.parse::<UserId>().unwrap_err();
        assert_eq!(err, ParseError::InvalidFormat);

        let user = "<@1ad32424>";
        let left_err = user.parse::<UserId>().unwrap_err();
        let right_err = "1ad32424".parse::<u64>().unwrap_err();
        assert_eq!(left_err, ParseError::ParseIntError(right_err));

        let user = "<@!904569845>";
        let user_id: UserId = user.parse().unwrap();
        assert_eq!(user_id, UserId(904569845));

        let user = "<1231235234534>";
        let err = user.parse::<UserId>().unwrap_err();
        assert_eq!(err, ParseError::InvalidFormat);

        let user = "<@1ad32424>";
        let left_err = user.parse::<UserId>().unwrap_err();
        let right_err = "1ad32424".parse::<u64>().unwrap_err();
        assert_eq!(left_err, ParseError::ParseIntError(right_err));
    }
}
