#[derive(Default)]
pub struct Store {
    pub pool: Option<sqlx::MySqlPool>,
}

// /// A type of data that can be stored in a [`Store`].
// pub trait StoreData: Sized {
//     fn get(query: StoreDataQuery<Self>) -> sqlx::Result<Vec<Self>>;
//     fn delete(query: StoreDataQuery<Self>) -> sqlx::Result<()>;
//     fn insert(&self) -> sqlx::Result<()>;
// }

// // pub struct StoreDataQuery<T> {
// //     x: T,
// // }

// pub trait Serializer {
//     type Ok;
//     type Error;

//     fn serialize_i8(self, value: i8) -> Result<Self::Ok, Self::Error>;
//     fn serialize_i16(self, value: i16) -> Result<Self::Ok, Self::Error>;
//     fn serialize_i32(self, value: i32) -> Result<Self::Ok, Self::Error>;
//     fn serialize_i64(self, value: i64) -> Result<Self::Ok, Self::Error>;
// }

// pub trait Serialize {
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//     where
//         S: Serializer;
// }

// impl Serialize for i8 {
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//     where
//         S: Serializer,
//     {
//         serializer.serialize_i8(*self)
//     }
// }
