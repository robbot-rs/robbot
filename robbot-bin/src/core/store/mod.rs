pub mod id;
pub mod mysql;

use async_trait::async_trait;
use serenity::model::id::{ChannelId, GuildId, MessageId, RoleId, UserId};

#[derive(Default)]
pub struct MainStore<S>
where
    S: Store,
{
    store: S,
}

impl<S> MainStore<S>
where
    S: Store,
{
    pub async fn new(uri: &str) -> sqlx::Result<Self> {
        Ok(Self {
            store: S::connect(uri).await?,
        })
    }

    pub async fn create<T>(&self) -> sqlx::Result<()>
    where
        T: StoreData<S> + Default + Send,
    {
        self.store.create::<T>().await
    }

    /// Delete all items matching the query.
    pub async fn delete<T, Q>(&self, query: Q) -> sqlx::Result<()>
    where
        T: StoreData<S> + Default + Send,
        Q: DataQuery<T, S> + Send + Sync,
    {
        self.store.delete(query).await
    }

    /// Returns all items from the store that match the query. Using
    /// `None` as the query returns all items avaliable.
    ///
    /// If you only need a single item, use [`Self::get_one`].
    pub async fn get<T, Q>(&self, query: Option<Q>) -> sqlx::Result<Vec<T>>
    where
        T: StoreData<S> + Send + Default,
        Q: DataQuery<T, S> + Send,
    {
        self.store.get(query).await
    }

    pub async fn get_all<T>(&self) -> sqlx::Result<Vec<T>>
    where
        T: StoreData<S> + Send + Default,
        T::DataQuery: Send + Sync,
    {
        self.get::<T, T::DataQuery>(None).await
    }

    /// Returns the first item matching the query.
    ///
    /// If you need all items matching the query, use [`Self::get`].
    pub async fn get_one<T, Q>(&self, query: Q) -> sqlx::Result<T>
    where
        T: StoreData<S> + Send + Default,
        Q: DataQuery<T, S> + Send + Sync,
    {
        self.store.get_one(query).await
    }

    pub async fn insert<T>(&self, data: T) -> sqlx::Result<()>
    where
        T: StoreData<S> + Send,
    {
        self.store.insert(data).await
    }
}

#[async_trait]
pub trait Store: Sized {
    type Serializer: Serializer<Self>;

    /// Create a new store (connection) using a
    /// connection string.
    async fn connect(uri: &str) -> sqlx::Result<Self>;

    /// Initialize the store for storing data of the
    /// type `T`. This might not be required on all
    /// types of stores.
    async fn create<T>(&self) -> sqlx::Result<()>
    where
        T: StoreData<Self> + Default + Send;

    /// Delete all items matching the query.
    async fn delete<T, Q>(&self, query: Q) -> sqlx::Result<()>
    where
        T: StoreData<Self> + Default + Send,
        Q: DataQuery<T, Self> + Send + Sync;

    /// Return all items matching the query. If the query
    /// is `None`, all avaliable items are returned.
    async fn get<T, Q>(&self, query: Option<Q>) -> sqlx::Result<Vec<T>>
    where
        T: StoreData<Self> + Default + Send,
        Q: DataQuery<T, Self> + Send;

    /// Return the first item matching the query.
    async fn get_one<T, Q>(&self, query: Q) -> sqlx::Result<T>
    where
        T: StoreData<Self> + Default + Send,
        Q: DataQuery<T, Self> + Send + Sync;

    /// Insert a new item into the store.
    async fn insert<T>(&self, data: T) -> sqlx::Result<()>
    where
        T: StoreData<Self> + Send;
}

/// Represents a type that can be stored in the store `T`.
/// Requires that all fields implement [`Serialize`] and
/// [`Deserialize`] for the store `T`.
pub trait StoreData<T>: Sized
where
    T: Store,
{
    type DataQuery: DataQuery<Self, T>;

    fn resource_name() -> String;

    fn serialize<S>(&self, serializer: &mut S) -> Result<S::Ok, S::Err>
    where
        S: Serializer<T>;

    fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Err>
    where
        D: Deserializer<T>;

    fn query() -> Self::DataQuery;
}

pub trait DataQuery<T, S>: Sized
where
    T: StoreData<S>,
    S: Store,
{
    fn into_vals(self) -> Vec<(String, String)>;
}

pub trait Serializer<S>
where
    S: Store,
{
    type Ok;
    type Err;

    fn serialize_bool(&mut self, v: bool) -> Result<Self::Ok, Self::Err>;

    fn serialize_i8(&mut self, v: i8) -> Result<Self::Ok, Self::Err>;
    fn serialize_i16(&mut self, v: i16) -> Result<Self::Ok, Self::Err>;
    fn serialize_i32(&mut self, v: i32) -> Result<Self::Ok, Self::Err>;
    fn serialize_i64(&mut self, v: i64) -> Result<Self::Ok, Self::Err>;

    fn serialize_u8(&mut self, v: u8) -> Result<Self::Ok, Self::Err>;
    fn serialize_u16(&mut self, v: u16) -> Result<Self::Ok, Self::Err>;
    fn serialize_u32(&mut self, v: u32) -> Result<Self::Ok, Self::Err>;
    fn serialize_u64(&mut self, v: u64) -> Result<Self::Ok, Self::Err>;

    fn serialize_f32(&mut self, v: f32) -> Result<Self::Ok, Self::Err>;
    fn serialize_f64(&mut self, v: f64) -> Result<Self::Ok, Self::Err>;

    fn serialize_str(&mut self, v: &str) -> Result<Self::Ok, Self::Err>;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<Self::Ok, Self::Err>
    where
        T: ?Sized + Serialize<S>;
}

pub trait Deserializer<S>
where
    S: Store,
{
    type Err;

    fn deserialize_bool(&mut self) -> Result<bool, Self::Err>;

    fn deserialize_i8(&mut self) -> Result<i8, Self::Err>;
    fn deserialize_i16(&mut self) -> Result<i16, Self::Err>;
    fn deserialize_i32(&mut self) -> Result<i32, Self::Err>;
    fn deserialize_i64(&mut self) -> Result<i64, Self::Err>;

    fn deserialize_u8(&mut self) -> Result<u8, Self::Err>;
    fn deserialize_u16(&mut self) -> Result<u16, Self::Err>;
    fn deserialize_u32(&mut self) -> Result<u32, Self::Err>;
    fn deserialize_u64(&mut self) -> Result<u64, Self::Err>;

    fn deserialize_f32(&mut self) -> Result<f32, Self::Err>;
    fn deserialize_f64(&mut self) -> Result<f64, Self::Err>;

    fn deserialize_string(&mut self) -> Result<String, Self::Err>;

    fn deserialize_field<T>(&mut self, key: &'static str) -> Result<T, Self::Err>
    where
        T: ?Sized + Deserialize<S>;
}

pub trait Serialize<T>
where
    T: Store,
{
    fn serialize<S>(&self, serializer: &mut S) -> Result<S::Ok, S::Err>
    where
        S: Serializer<T>;
}

pub trait Deserialize<T>: Sized
where
    T: Store,
{
    fn deserialize<S>(deserializer: &mut S) -> Result<Self, S::Err>
    where
        S: Deserializer<T>;
}

// =================================================================
// === Implementations for types that depend on primitive types. ===
// =================================================================

impl<T> Serialize<T> for ChannelId
where
    T: Store,
    u64: Serialize<T>,
{
    fn serialize<S>(&self, serializer: &mut S) -> Result<S::Ok, S::Err>
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
    fn serialize<S>(&self, serializer: &mut S) -> Result<S::Ok, S::Err>
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
    fn serialize<S>(&self, serializer: &mut S) -> Result<S::Ok, S::Err>
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
    fn serialize<S>(&self, serializer: &mut S) -> Result<S::Ok, S::Err>
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
    fn serialize<S>(&self, serializer: &mut S) -> Result<S::Ok, S::Err>
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
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Err>
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
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Err>
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
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Err>
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
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Err>
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
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Err>
    where
        D: Deserializer<T>,
    {
        let v = u64::deserialize(deserializer)?;

        Ok(Self(v))
    }
}
