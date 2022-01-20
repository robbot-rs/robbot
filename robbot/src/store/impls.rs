//! [`Serialize`] and [`Deserialize`] implementations for common
//! types.

use super::{Deserialize, Deserializer, Serialize, Serializer, Store};

use crate::model::id::{ChannelId, GuildId, MessageId, RoleId, UserId};

impl<T> Serialize<T> for ChannelId
where
    T: Store,
    u64: Serialize<T>,
{
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
    where
        S: Serializer<T>,
    {
        self.0.serialize(serializer)
    }
}

impl<T> Serialize<T> for GuildId
where
    T: Store,
    u64: Serialize<T>,
{
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
    where
        S: Serializer<T>,
    {
        self.0.serialize(serializer)
    }
}

impl<T> Serialize<T> for MessageId
where
    T: Store,
    u64: Serialize<T>,
{
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
    where
        S: Serializer<T>,
    {
        self.0.serialize(serializer)
    }
}

impl<T> Serialize<T> for RoleId
where
    T: Store,
    u64: Serialize<T>,
{
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
    where
        S: Serializer<T>,
    {
        self.0.serialize(serializer)
    }
}

impl<T> Serialize<T> for UserId
where
    T: Store,
    u64: Serialize<T>,
{
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
    where
        S: Serializer<T>,
    {
        self.0.serialize(serializer)
    }
}

impl<T> Deserialize<T> for ChannelId
where
    T: Store,
    u64: Deserialize<T>,
{
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error>
    where
        D: Deserializer<T>,
    {
        let v = u64::deserialize(deserializer)?;

        Ok(Self(v))
    }
}

impl<T> Deserialize<T> for GuildId
where
    T: Store,
    u64: Deserialize<T>,
{
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error>
    where
        D: Deserializer<T>,
    {
        let v = u64::deserialize(deserializer)?;

        Ok(Self(v))
    }
}

impl<T> Deserialize<T> for MessageId
where
    T: Store,
    u64: Deserialize<T>,
{
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error>
    where
        D: Deserializer<T>,
    {
        let v = u64::deserialize(deserializer)?;

        Ok(Self(v))
    }
}

impl<T> Deserialize<T> for RoleId
where
    T: Store,
    u64: Deserialize<T>,
{
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error>
    where
        D: Deserializer<T>,
    {
        let v = u64::deserialize(deserializer)?;

        Ok(Self(v))
    }
}

impl<T> Deserialize<T> for UserId
where
    T: Store,
    u64: Deserialize<T>,
{
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error>
    where
        D: Deserializer<T>,
    {
        let v = u64::deserialize(deserializer)?;

        Ok(Self(v))
    }
}
