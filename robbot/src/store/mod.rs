pub mod id;
mod impls;
pub mod lazy;

use async_trait::async_trait;
use std::error::Error;

pub use robbot_derive::{get, get_one};

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
    ///
    /// [`delete`]: Self::delete
    /// [`get`]: Self::get
    /// [`get_all`]: Self::get_all
    /// [`get_one`]: Self::get_one
    /// [`insert`]: Self::insert
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
    /// might return items in different stores depending on the store or even
    /// using the same store.
    async fn get_one<T, D, Q>(&self, descriptor: D, query: Q) -> Result<Option<T>, Self::Error>
    where
        T: StoreData<Self> + Send + Sync + 'static,
        D: DataDescriptor<T, Self> + Send,
        Q: DataQuery<T, Self> + Send;

    /// Inserts a new item into the store.
    async fn insert<T>(&self, data: T) -> Result<(), Self::Error>
    where
        T: StoreData<Self> + Send + Sync + 'static;

    fn make_query<T>(&self) -> T::DataQuery
    where
        T: StoreData<Self>,
        T::DataQuery: Default,
    {
        T::DataQuery::default()
    }

    fn make_descriptor<T>(&self) -> T::DataDescriptor
    where
        T: StoreData<Self>,
        T::DataDescriptor: Default,
    {
        T::DataDescriptor::default()
    }
}

/// A type for serializing some data into a query for store `S`.
pub trait Serializer<S>
where
    S: Store,
{
    type Error;

    /// Serializes a `bool` value.
    fn serialize_bool(&mut self, v: bool) -> Result<(), Self::Error>;

    /// Serializes a `i8` value.
    fn serialize_i8(&mut self, v: i8) -> Result<(), Self::Error>;

    /// Serializes a `i16` value.
    fn serialize_i16(&mut self, v: i16) -> Result<(), Self::Error>;

    /// Serializes a `i32` value.
    fn serialize_i32(&mut self, v: i32) -> Result<(), Self::Error>;

    /// Serializes a `i64` value.
    fn serialize_i64(&mut self, v: i64) -> Result<(), Self::Error>;

    /// Serializes a `u8` value.
    fn serialize_u8(&mut self, v: u8) -> Result<(), Self::Error>;

    /// Serializes a `u16` value.
    fn serialize_u16(&mut self, v: u16) -> Result<(), Self::Error>;

    /// Serializes a `u32` value.
    fn serialize_u32(&mut self, v: u32) -> Result<(), Self::Error>;

    /// Serializes a `u64` value.
    fn serialize_u64(&mut self, v: u64) -> Result<(), Self::Error>;

    /// Serializes a `f32` value.
    fn serialize_f32(&mut self, v: f32) -> Result<(), Self::Error>;

    /// Serializes a `f64` value.
    fn serialize_f64(&mut self, v: f64) -> Result<(), Self::Error>;

    /// Serializes a `&str` value.
    fn serialize_str(&mut self, v: &str) -> Result<(), Self::Error>;

    /// Serialize a single field. A field is a single key-value pair where the key
    /// is a `str` and the value is `T`.
    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize<S>;
}

/// A type for deserializing the response for store `S` into some data.
pub trait Deserializer<S>
where
    S: Store,
{
    type Error;

    /// Deserializes a `bool` value.
    fn deserialize_bool(&mut self) -> Result<bool, Self::Error>;

    /// Deserializes a `i8` value.
    fn deserialize_i8(&mut self) -> Result<i8, Self::Error>;

    /// Deserializes a `i16` value.
    fn deserialize_i16(&mut self) -> Result<i16, Self::Error>;

    /// Deserializes a `i32` value.
    fn deserialize_i32(&mut self) -> Result<i32, Self::Error>;

    /// Deserializes a `i64` value.
    fn deserialize_i64(&mut self) -> Result<i64, Self::Error>;

    /// Deserializes a `u8` value.
    fn deserialize_u8(&mut self) -> Result<u8, Self::Error>;

    /// Deserializes a `u16` value.
    fn deserialize_u16(&mut self) -> Result<u16, Self::Error>;

    /// Deserializes a `u32` value.
    fn deserialize_u32(&mut self) -> Result<u32, Self::Error>;

    /// Deserializes a `u64` value.
    fn deserialize_u64(&mut self) -> Result<u64, Self::Error>;

    /// Deserializes a `f32` value.
    fn deserialize_f32(&mut self) -> Result<f32, Self::Error>;

    /// Deserializes a `f64` value.
    fn deserialize_f64(&mut self) -> Result<f64, Self::Error>;

    /// Deserializes a `String` value.
    fn deserialize_string(&mut self) -> Result<String, Self::Error>;

    /// Deserializes a single field. A field is a single key-value pair where
    /// the key is a `str` and the value is `T`.
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

/// A primitive store type or type that can be serialized as a single key
/// in a store.
pub trait Serialize<T>
where
    T: Store,
{
    /// Serializes this value into the serializer.
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
    where
        S: Serializer<T>;

    /// Serializes the type into the serializer.
    fn serialize_type<S>(serializer: &mut S) -> Result<(), S::Error>
    where
        S: TypeSerializer<T>;
}

/// A primitive store type or type that can be deserialized from a single key
/// in a store.
pub trait Deserialize<T>: Sized
where
    T: Store,
{
    /// Deserializes the value from the deserializer.
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error>
    where
        D: Deserializer<T>;
}

/// Some data that can be stored in a store `T`.
///
/// The [`StoreData`] derive macro automatically implements `StoreData` for all
/// stores that support all the structs contained fields.
///
/// [`StoreData`]: ../derive.StoreData.html
pub trait StoreData<T>: Sized
where
    T: Store,
{
    /// The type describing how to construct `Self`.
    type DataDescriptor: DataDescriptor<Self, T>;

    /// The type for building a query for `Self`.
    type DataQuery: DataQuery<Self, T>;

    /// Returns the unique ressource name.
    fn resource_name() -> String;

    /// Serializes the value into the serializer.
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
    where
        S: Serializer<T>;

    /// Deserializes the value from the deserializer.
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error>
    where
        D: Deserializer<T>;

    /// Returns a new default value of [`Self::DataQuery`].
    fn query() -> Self::DataQuery;
}

/// A descriptor of how to construct some [`StoreData`].
pub trait DataDescriptor<T, U>
where
    T: StoreData<U>,
    U: Store,
{
    /// Serializes the value into the serializer.
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
    /// Serializes the value into the serializer.
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
    where
        S: Serializer<U>;
}
