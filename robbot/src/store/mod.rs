pub mod id;
mod impls;

use async_trait::async_trait;
use std::error::Error;

#[async_trait]
pub trait Store: Sized {
    type Serializer: Serializer<Self>;
    type Error: Error;

    async fn connect(uri: &str) -> Result<Self, Self::Error>;

    async fn create<T>(&self) -> Result<(), Self::Error>
    where
        T: StoreData<Self> + Default + Send;

    async fn delete<T, Q>(&self, query: Q) -> Result<(), Self::Error>
    where
        T: StoreData<Self> + Default + Send,
        Q: DataQuery<T, Self> + Send;

    async fn get<T, Q>(&self, query: Q) -> Result<Vec<T>, Self::Error>
    where
        T: StoreData<Self> + Default + Send,
        Q: DataQuery<T, Self> + Send;

    async fn get_all<T>(&self) -> Result<Vec<T>, Self::Error>
    where
        T: StoreData<Self> + Default + Send;

    async fn get_one<T, Q>(&self, query: Q) -> Result<T, Self::Error>
    where
        T: StoreData<Self> + Default + Send,
        Q: DataQuery<T, Self> + Send;

    async fn insert<T>(&self, data: T) -> Result<(), Self::Error>
    where
        T: StoreData<Self> + Send;
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

pub trait Serialize<T>
where
    T: Store,
{
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
    where
        S: Serializer<T>;
}

pub trait Deserialize<T>: Sized
where
    T: Store,
{
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error>
    where
        D: Deserializer<T>;
}

pub trait StoreData<T>: Sized
where
    T: Store,
{
    type DataQuery: DataQuery<Self, T>;

    fn resource_name() -> String;

    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
    where
        S: Serializer<T>;

    fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error>
    where
        D: Deserializer<T>;

    fn query() -> Self::DataQuery;
}

pub trait DataQuery<T, U>
where
    T: StoreData<U>,
    U: Store,
{
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
    where
        S: Serializer<U>;
}
