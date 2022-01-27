pub mod id;
mod impls;

use async_trait::async_trait;
use std::error::Error;

#[async_trait]
pub trait Store: Sized {
    type Serializer: Serializer<Self>;
    type Error: Error;

    async fn connect(uri: &str) -> Result<Self, Self::Error>;

    /// Initializes the store for storing data of the type `T`.
    /// If `create` was not called before calling [`delete`],
    /// [`get`], [`get_all`], [`get_one`] or [`insert`] on the
    /// store, the operation might fail.
    ///
    /// Note: Calling `create` might not be required for all
    /// store types.
    async fn create<T, D>(&self, descriptor: D) -> Result<(), Self::Error>
    where
        T: StoreData<Self> + Send + Sync + 'static,
        D: DataDescriptor<T, Self> + Send + Sync;

    /// Deletes all items of type `T` matching the query `Q`
    /// from the store.
    async fn delete<T, Q>(&self, query: Q) -> Result<(), Self::Error>
    where
        T: StoreData<Self> + Send + Sync + 'static,
        Q: DataQuery<T, Self> + Send;

    /// Returns all items of type `T` matching the query `Q`
    /// from the store. If no items are stored, an empty [`Vec`]
    /// is returned.
    async fn get<T, D, Q>(&self, descriptor: D, query: Q) -> Result<Vec<T>, Self::Error>
    where
        T: StoreData<Self> + Send + Sync + 'static,
        D: DataDescriptor<T, Self> + Send + Sync,
        Q: DataQuery<T, Self> + Send;

    /// Returns all items of type `T` from the store. If no items
    /// are stored, an empty [`Vec`] is returned.
    async fn get_all<T, D>(&self, descriptor: D) -> Result<Vec<T>, Self::Error>
    where
        T: StoreData<Self> + Send + Sync + 'static,
        D: DataDescriptor<T, Self> + Send + Sync;

    /// Returns the an item of type `T` matching the query `Q`
    /// from the store. If no items of type `T` are stored, `None`
    /// is returned.
    ///
    /// Note: There is no guarantee of how items are ordered. `get_one`
    /// might return different items depending on the store.
    async fn get_one<T, D, Q>(&self, descriptor: D, query: Q) -> Result<Option<T>, Self::Error>
    where
        T: StoreData<Self> + Send + Sync + 'static,
        D: DataDescriptor<T, Self> + Send,
        Q: DataQuery<T, Self> + Send;

    /// Inserts a new item into the store.
    async fn insert<T>(&self, data: T) -> Result<(), Self::Error>
    where
        T: StoreData<Self> + Send + Sync + 'static;
}

pub trait Serializer<S>
where
    S: Store,
{
    type Error;

    fn serialize_bool(&mut self, v: bool) -> Result<(), Self::Error>;

    fn serialize_i8(&mut self, v: i8) -> Result<(), Self::Error>;
    fn serialize_i16(&mut self, v: i16) -> Result<(), Self::Error>;
    fn serialize_i32(&mut self, v: i32) -> Result<(), Self::Error>;
    fn serialize_i64(&mut self, v: i64) -> Result<(), Self::Error>;

    fn serialize_u8(&mut self, v: u8) -> Result<(), Self::Error>;
    fn serialize_u16(&mut self, v: u16) -> Result<(), Self::Error>;
    fn serialize_u32(&mut self, v: u32) -> Result<(), Self::Error>;
    fn serialize_u64(&mut self, v: u64) -> Result<(), Self::Error>;

    fn serialize_f32(&mut self, v: f32) -> Result<(), Self::Error>;
    fn serialize_f64(&mut self, v: f64) -> Result<(), Self::Error>;

    fn serialize_str(&mut self, v: &str) -> Result<(), Self::Error>;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize<S>;
}

pub trait Deserializer<S>
where
    S: Store,
{
    type Error;

    fn deserialize_bool(&mut self) -> Result<bool, Self::Error>;

    fn deserialize_i8(&mut self) -> Result<i8, Self::Error>;
    fn deserialize_i16(&mut self) -> Result<i16, Self::Error>;
    fn deserialize_i32(&mut self) -> Result<i32, Self::Error>;
    fn deserialize_i64(&mut self) -> Result<i64, Self::Error>;

    fn deserialize_u8(&mut self) -> Result<u8, Self::Error>;
    fn deserialize_u16(&mut self) -> Result<u16, Self::Error>;
    fn deserialize_u32(&mut self) -> Result<u32, Self::Error>;
    fn deserialize_u64(&mut self) -> Result<u64, Self::Error>;

    fn deserialize_f32(&mut self) -> Result<f32, Self::Error>;
    fn deserialize_f64(&mut self) -> Result<f64, Self::Error>;

    fn deserialize_string(&mut self) -> Result<String, Self::Error>;

    fn deserialize_field<T>(&mut self, key: &'static str) -> Result<T, Self::Error>
    where
        T: Sized + Deserialize<S>;
}

pub trait TypeSerializer<S>
where
    S: Store,
{
    type Error;

    fn serialize_bool(&mut self) -> Result<(), Self::Error>;

    fn serialize_i8(&mut self) -> Result<(), Self::Error>;
    fn serialize_i16(&mut self) -> Result<(), Self::Error>;
    fn serialize_i32(&mut self) -> Result<(), Self::Error>;
    fn serialize_i64(&mut self) -> Result<(), Self::Error>;

    fn serialize_u8(&mut self) -> Result<(), Self::Error>;
    fn serialize_u16(&mut self) -> Result<(), Self::Error>;
    fn serialize_u32(&mut self) -> Result<(), Self::Error>;
    fn serialize_u64(&mut self) -> Result<(), Self::Error>;

    fn serialize_f32(&mut self) -> Result<(), Self::Error>;
    fn serialize_f64(&mut self) -> Result<(), Self::Error>;

    fn serialize_str(&mut self) -> Result<(), Self::Error>;

    fn serialize_field<T>(&mut self, key: &'static str) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize<S>;
}

pub trait Serialize<T>
where
    T: Store,
{
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
    where
        S: Serializer<T>;

    fn serialize_type<S>(serializer: &mut S) -> Result<(), S::Error>
    where
        S: TypeSerializer<T>;
}

pub trait Deserialize<T>: Sized
where
    T: Store,
{
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error>
    where
        D: Deserializer<T>;
}

pub trait StoreData<T>: Sized + Clone
where
    T: Store,
{
    type DataQuery: DataQuery<Self, T>;
    type DataDescriptor: DataDescriptor<Self, T>;

    fn resource_name() -> String;

    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
    where
        S: Serializer<T>;

    fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error>
    where
        D: Deserializer<T>;

    fn query() -> Self::DataQuery;
}

pub trait DataDescriptor<T, U>
where
    T: StoreData<U>,
    U: Store,
{
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
    where
        S: TypeSerializer<U>;
}

/// A `DataQuery<T, U>` is used to build a query for the [`StoreData`] data `T`
/// for the [`Store`] `U`.
pub trait DataQuery<T, U>
where
    T: StoreData<U>,
    U: Store,
{
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
    where
        S: Serializer<U>;
}
