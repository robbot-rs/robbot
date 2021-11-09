use super::{DataQuery, Deserialize, Deserializer, Serialize, Serializer, Store, StoreData};
use async_trait::async_trait;
use futures::TryStreamExt;
use sqlx::{
    mysql::{MySqlPool, MySqlRow},
    Row,
};
use std::{
    error,
    fmt::{self, Display, Formatter},
};

#[derive(Debug)]
pub enum Error {
    SQLx(sqlx::Error),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::SQLx(err) => err,
            }
        )
    }
}

impl error::Error for Error {}

impl From<sqlx::Error> for Error {
    fn from(err: sqlx::Error) -> Self {
        Self::SQLx(err)
    }
}

/// A Store using the MySQL database.
pub struct MysqlStore {
    pool: MySqlPool,
}

#[async_trait]
impl Store for MysqlStore {
    type Serializer = MysqlSerializer;

    async fn connect(uri: &str) -> sqlx::Result<Self> {
        let pool = MySqlPool::connect(uri).await?;

        Ok(Self { pool })
    }

    async fn create<T>(&self) -> sqlx::Result<()>
    where
        T: StoreData<Self> + Default + Send,
    {
        let data = T::default();
        let table_name = T::resource_name();

        let mut serializer = MysqlSerializer::new(table_name, QueryKind::Create);
        data.serialize(&mut serializer).unwrap();

        sqlx::query(&serializer.into_sql())
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn delete<T, Q>(&self, query: Q) -> sqlx::Result<()>
    where
        T: StoreData<Self> + Default + Send,
        Q: DataQuery<T, Self> + Send + Sync,
    {
        let table_name = T::resource_name();

        let mut serializer = MysqlSerializer::new(table_name, QueryKind::Delete);

        serializer.cond = query.into_vals();

        sqlx::query(&serializer.into_sql())
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn get<T, Q>(&self, query: Option<Q>) -> sqlx::Result<Vec<T>>
    where
        T: StoreData<Self> + Default + Send,
        Q: DataQuery<T, Self> + Send,
    {
        let data = T::default();
        let table_name = T::resource_name();

        let mut serializer = MysqlSerializer::new(table_name, QueryKind::Select);

        if let Some(query) = query {
            serializer.cond = query.into_vals();
        }

        data.serialize(&mut serializer).unwrap();

        let sql = serializer.into_sql();
        let mut rows = sqlx::query(&sql).fetch(&self.pool);

        let mut entries = Vec::new();

        while let Some(row) = rows.try_next().await? {
            let mut deserializer = MysqlDeserializer::new(row);
            let data = T::deserialize(&mut deserializer).unwrap();

            entries.push(data);
        }

        Ok(entries)
    }

    async fn get_one<T, Q>(&self, query: Q) -> sqlx::Result<T>
    where
        T: StoreData<Self> + Default + Send,
        Q: DataQuery<T, Self> + Send + Sync,
    {
        let data = T::default();
        let table_name = T::resource_name();

        let mut serializer = MysqlSerializer::new(table_name, QueryKind::Select);

        serializer.cond = query.into_vals();

        data.serialize(&mut serializer).unwrap();

        let sql = serializer.into_sql();
        let row = sqlx::query(&sql).fetch_one(&self.pool).await.unwrap();

        let mut deserializer = MysqlDeserializer::new(row);
        let data = T::deserialize(&mut deserializer).unwrap();

        Ok(data)
    }

    async fn insert<T>(&self, data: T) -> sqlx::Result<()>
    where
        T: StoreData<Self> + Send,
    {
        let table_name = T::resource_name();

        let mut serializer = MysqlSerializer::new(table_name, QueryKind::Insert);
        data.serialize(&mut serializer).unwrap();

        sqlx::query(&serializer.into_sql())
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}

/// Type of the sql query being built.
enum QueryKind {
    Create,
    Delete,
    Insert,
    Select,
}

/// A [`Serializer`] for building SQL queries for use
/// with [`MysqlStore`].
pub struct MysqlSerializer {
    table_name: String,
    kind: QueryKind,
    cols: Vec<String>,
    vals: Vec<String>,
    /// Conditions
    cond: Vec<(String, String)>,
}

impl MysqlSerializer {
    fn new(table_name: String, kind: QueryKind) -> Self {
        Self {
            table_name,
            kind,
            cols: Vec::new(),
            vals: Vec::new(),
            cond: Vec::new(),
        }
    }

    fn into_sql(self) -> String {
        match self.kind {
            QueryKind::Create => {
                if self.cols.len() != self.vals.len() {
                    panic!("Mismatched number of cols and vals");
                }

                let mut values = Vec::new();

                for i in 0..self.cols.len() {
                    values.push(format!("{} {}", self.cols[i], self.vals[i]));
                }

                format!(
                    "CREATE TABLE IF NOT EXISTS {} ({})",
                    self.table_name,
                    values.join(", ")
                )
            }
            QueryKind::Delete => {
                let mut filter = String::new();
                if !self.cond.is_empty() {
                    filter = format!(
                        "WHERE {}",
                        self.cond
                            .iter()
                            .map(|(col, val)| format!("{} = {}", col, val))
                            .collect::<Vec<String>>()
                            .join(" AND ")
                    )
                }

                format!("DELETE FROM {} {}", self.table_name, filter)
            }
            QueryKind::Insert => {
                if self.cols.len() != self.vals.len() {
                    panic!("Mismatched number of cols and vals");
                }

                format!(
                    "INSERT INTO {} ({}) VALUES ({})",
                    self.table_name,
                    self.cols.join(", "),
                    self.vals.join(", ")
                )
            }
            QueryKind::Select => {
                let mut filter = String::new();
                if !self.cond.is_empty() {
                    filter = format!(
                        "WHERE {}",
                        self.cond
                            .iter()
                            .map(|(col, val)| format!("{} = {}", col, val))
                            .collect::<Vec<String>>()
                            .join(" AND ")
                    )
                }

                format!(
                    "SELECT {} FROM {} {}",
                    self.cols.join(", "),
                    self.table_name,
                    filter
                )
            }
        }
    }
}

impl Serializer<MysqlStore> for MysqlSerializer {
    type Ok = ();
    type Err = Error;

    fn serialize_bool(&mut self, v: bool) -> Result<Self::Ok, Self::Err> {
        match self.kind {
            QueryKind::Create => self.vals.push(String::from("BOOLEAN")),
            QueryKind::Delete => {
                let val = match v {
                    false => String::from("FALSE"),
                    true => String::from("TRUE"),
                };

                self.vals.push(val);
            }
            QueryKind::Insert => {
                let val = match v {
                    false => String::from("FALSE"),
                    true => String::from("TRUE"),
                };

                self.vals.push(val);
            }
            _ => (),
        }

        Ok(())
    }

    fn serialize_i8(&mut self, v: i8) -> Result<Self::Ok, Self::Err> {
        match self.kind {
            QueryKind::Create => self.vals.push(String::from("TINYINT")),
            QueryKind::Delete => self.vals.push(v.to_string()),
            QueryKind::Insert => self.vals.push(v.to_string()),
            _ => (),
        }

        Ok(())
    }

    fn serialize_i16(&mut self, v: i16) -> Result<Self::Ok, Self::Err> {
        match self.kind {
            QueryKind::Create => self.vals.push(String::from("SMALLINT")),
            QueryKind::Delete => self.vals.push(v.to_string()),
            QueryKind::Insert => self.vals.push(v.to_string()),
            _ => (),
        }

        Ok(())
    }

    fn serialize_i32(&mut self, v: i32) -> Result<Self::Ok, Self::Err> {
        match self.kind {
            QueryKind::Create => self.vals.push(String::from("INT")),
            QueryKind::Delete => self.vals.push(v.to_string()),
            QueryKind::Insert => self.vals.push(v.to_string()),
            _ => (),
        }

        Ok(())
    }

    fn serialize_i64(&mut self, v: i64) -> Result<Self::Ok, Self::Err> {
        match self.kind {
            QueryKind::Create => self.vals.push(String::from("BIGINT")),
            QueryKind::Delete => self.vals.push(v.to_string()),
            QueryKind::Insert => self.vals.push(v.to_string()),
            _ => (),
        }

        Ok(())
    }

    fn serialize_u8(&mut self, v: u8) -> Result<Self::Ok, Self::Err> {
        match self.kind {
            QueryKind::Create => self.vals.push(String::from("TINYINT UNSIGNED")),
            QueryKind::Delete => self.vals.push(v.to_string()),
            QueryKind::Insert => self.vals.push(v.to_string()),
            _ => (),
        }

        Ok(())
    }

    fn serialize_u16(&mut self, v: u16) -> Result<Self::Ok, Self::Err> {
        match self.kind {
            QueryKind::Create => self.vals.push(String::from("SMALLINT UNSIGNED")),
            QueryKind::Delete => self.vals.push(v.to_string()),
            QueryKind::Insert => self.vals.push(v.to_string()),
            _ => (),
        }

        Ok(())
    }

    fn serialize_u32(&mut self, v: u32) -> Result<Self::Ok, Self::Err> {
        match self.kind {
            QueryKind::Create => self.vals.push(String::from("INT UNSIGNED")),
            QueryKind::Delete => self.vals.push(v.to_string()),
            QueryKind::Insert => self.vals.push(v.to_string()),
            _ => (),
        }

        Ok(())
    }

    fn serialize_u64(&mut self, v: u64) -> Result<Self::Ok, Self::Err> {
        match self.kind {
            QueryKind::Create => self.vals.push(String::from("BIGINT UNSIGNED")),
            QueryKind::Delete => self.vals.push(v.to_string()),
            QueryKind::Insert => self.vals.push(v.to_string()),
            _ => (),
        }

        Ok(())
    }

    fn serialize_f32(&mut self, v: f32) -> Result<Self::Ok, Self::Err> {
        match self.kind {
            QueryKind::Create => self.vals.push(String::from("FLOAT")),
            QueryKind::Delete => self.vals.push(v.to_string()),
            QueryKind::Insert => self.vals.push(v.to_string()),
            _ => (),
        }

        Ok(())
    }

    fn serialize_f64(&mut self, v: f64) -> Result<Self::Ok, Self::Err> {
        match self.kind {
            QueryKind::Create => self.vals.push(String::from("DOUBLE")),
            QueryKind::Delete => self.vals.push(v.to_string()),
            QueryKind::Insert => self.vals.push(v.to_string()),
            _ => (),
        }

        Ok(())
    }

    fn serialize_str(&mut self, v: &str) -> Result<Self::Ok, Self::Err> {
        match self.kind {
            QueryKind::Create => self.vals.push(String::from("TEXT")),
            QueryKind::Delete => self.vals.push(format!("'{}'", v.replace("'", "\\'"))),
            QueryKind::Insert => self.vals.push(format!("'{}'", v.replace("'", "\\'"))),
            _ => (),
        }

        Ok(())
    }

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<Self::Ok, Self::Err>
    where
        T: ?Sized + Serialize<MysqlStore>,
    {
        self.cols.push(key.to_owned());
        value.serialize(self)?;

        Ok(())
    }
}

/// A [`Deserializer`] for reading datatypes from rows. Used
/// by [`MysqlStore`].
pub struct MysqlDeserializer {
    row: MySqlRow,
    column: Option<&'static str>,
}

impl MysqlDeserializer {
    /// Create a new `MysqlDeserializer` from a [`MySqlRow`].
    fn new(row: MySqlRow) -> Self {
        Self { row, column: None }
    }

    /// Return the last requested column name.
    fn column(&self) -> &'static str {
        self.column.unwrap()
    }
}

impl Deserializer<MysqlStore> for MysqlDeserializer {
    type Err = Error;

    fn deserialize_bool(&mut self) -> Result<bool, Self::Err> {
        let v = self.row.try_get(self.column())?;

        Ok(v)
    }

    fn deserialize_i8(&mut self) -> Result<i8, Self::Err> {
        let v = self.row.try_get(self.column())?;

        Ok(v)
    }

    fn deserialize_i16(&mut self) -> Result<i16, Self::Err> {
        let v = self.row.try_get(self.column())?;

        Ok(v)
    }

    fn deserialize_i32(&mut self) -> Result<i32, Self::Err> {
        let v = self.row.try_get(self.column())?;

        Ok(v)
    }

    fn deserialize_i64(&mut self) -> Result<i64, Self::Err> {
        let v = self.row.try_get(self.column())?;

        Ok(v)
    }

    fn deserialize_u8(&mut self) -> Result<u8, Self::Err> {
        let v = self.row.try_get(self.column())?;

        Ok(v)
    }

    fn deserialize_u16(&mut self) -> Result<u16, Self::Err> {
        let v = self.row.try_get(self.column())?;

        Ok(v)
    }

    fn deserialize_u32(&mut self) -> Result<u32, Self::Err> {
        let v = self.row.try_get(self.column())?;

        Ok(v)
    }

    fn deserialize_u64(&mut self) -> Result<u64, Self::Err> {
        let v = self.row.try_get(self.column())?;

        Ok(v)
    }

    fn deserialize_f32(&mut self) -> Result<f32, Self::Err> {
        let v = self.row.try_get(self.column())?;

        Ok(v)
    }

    fn deserialize_f64(&mut self) -> Result<f64, Self::Err> {
        let v = self.row.try_get(self.column())?;

        Ok(v)
    }

    fn deserialize_string(&mut self) -> Result<String, Self::Err> {
        let v = self.row.try_get(self.column())?;

        Ok(v)
    }

    fn deserialize_field<T>(&mut self, key: &'static str) -> Result<T, Self::Err>
    where
        T: Deserialize<MysqlStore>,
    {
        self.column = Some(key);
        T::deserialize(self)
    }
}

// ====================================================
// === Implement [`Serialize`] for supported types. ===
// ====================================================

impl Serialize<MysqlStore> for bool {
    fn serialize<S>(&self, serializer: &mut S) -> Result<S::Ok, S::Err>
    where
        S: Serializer<MysqlStore>,
    {
        serializer.serialize_bool(*self)
    }
}

impl Serialize<MysqlStore> for i8 {
    fn serialize<S>(&self, serializer: &mut S) -> Result<S::Ok, S::Err>
    where
        S: Serializer<MysqlStore>,
    {
        serializer.serialize_i8(*self)
    }
}

impl Serialize<MysqlStore> for i16 {
    fn serialize<S>(&self, serializer: &mut S) -> Result<S::Ok, S::Err>
    where
        S: Serializer<MysqlStore>,
    {
        serializer.serialize_i16(*self)
    }
}

impl Serialize<MysqlStore> for i32 {
    fn serialize<S>(&self, serializer: &mut S) -> Result<S::Ok, S::Err>
    where
        S: Serializer<MysqlStore>,
    {
        serializer.serialize_i32(*self)
    }
}

impl Serialize<MysqlStore> for i64 {
    fn serialize<S>(&self, serializer: &mut S) -> Result<S::Ok, S::Err>
    where
        S: Serializer<MysqlStore>,
    {
        serializer.serialize_i64(*self)
    }
}

impl Serialize<MysqlStore> for u8 {
    fn serialize<S>(&self, serializer: &mut S) -> Result<S::Ok, S::Err>
    where
        S: Serializer<MysqlStore>,
    {
        serializer.serialize_u8(*self)
    }
}

impl Serialize<MysqlStore> for u16 {
    fn serialize<S>(&self, serializer: &mut S) -> Result<S::Ok, S::Err>
    where
        S: Serializer<MysqlStore>,
    {
        serializer.serialize_u16(*self)
    }
}

impl Serialize<MysqlStore> for u32 {
    fn serialize<S>(&self, serializer: &mut S) -> Result<S::Ok, S::Err>
    where
        S: Serializer<MysqlStore>,
    {
        serializer.serialize_u32(*self)
    }
}

impl Serialize<MysqlStore> for u64 {
    fn serialize<S>(&self, serializer: &mut S) -> Result<S::Ok, S::Err>
    where
        S: Serializer<MysqlStore>,
    {
        serializer.serialize_u64(*self)
    }
}

impl Serialize<MysqlStore> for f32 {
    fn serialize<S>(&self, serializer: &mut S) -> Result<S::Ok, S::Err>
    where
        S: Serializer<MysqlStore>,
    {
        serializer.serialize_f32(*self)
    }
}

impl Serialize<MysqlStore> for f64 {
    fn serialize<S>(&self, serializer: &mut S) -> Result<S::Ok, S::Err>
    where
        S: Serializer<MysqlStore>,
    {
        serializer.serialize_f64(*self)
    }
}

impl Serialize<MysqlStore> for str {
    fn serialize<S>(&self, serializer: &mut S) -> Result<S::Ok, S::Err>
    where
        S: Serializer<MysqlStore>,
    {
        serializer.serialize_str(self)
    }
}

impl Serialize<MysqlStore> for String {
    fn serialize<S>(&self, serializer: &mut S) -> Result<S::Ok, S::Err>
    where
        S: Serializer<MysqlStore>,
    {
        serializer.serialize_str(self)
    }
}

// ======================================================
// === Implement [`Deserialize`] for supported types. ===
// ======================================================

impl Deserialize<MysqlStore> for bool {
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Err>
    where
        D: Deserializer<MysqlStore>,
    {
        deserializer.deserialize_bool()
    }
}

impl Deserialize<MysqlStore> for i8 {
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Err>
    where
        D: Deserializer<MysqlStore>,
    {
        deserializer.deserialize_i8()
    }
}

impl Deserialize<MysqlStore> for i16 {
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Err>
    where
        D: Deserializer<MysqlStore>,
    {
        deserializer.deserialize_i16()
    }
}

impl Deserialize<MysqlStore> for i32 {
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Err>
    where
        D: Deserializer<MysqlStore>,
    {
        deserializer.deserialize_i32()
    }
}

impl Deserialize<MysqlStore> for i64 {
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Err>
    where
        D: Deserializer<MysqlStore>,
    {
        deserializer.deserialize_i64()
    }
}

impl Deserialize<MysqlStore> for u8 {
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Err>
    where
        D: Deserializer<MysqlStore>,
    {
        deserializer.deserialize_u8()
    }
}

impl Deserialize<MysqlStore> for u16 {
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Err>
    where
        D: Deserializer<MysqlStore>,
    {
        deserializer.deserialize_u16()
    }
}

impl Deserialize<MysqlStore> for u32 {
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Err>
    where
        D: Deserializer<MysqlStore>,
    {
        deserializer.deserialize_u32()
    }
}

impl Deserialize<MysqlStore> for u64 {
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Err>
    where
        D: Deserializer<MysqlStore>,
    {
        deserializer.deserialize_u64()
    }
}

impl Deserialize<MysqlStore> for f32 {
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Err>
    where
        D: Deserializer<MysqlStore>,
    {
        deserializer.deserialize_f32()
    }
}

impl Deserialize<MysqlStore> for f64 {
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Err>
    where
        D: Deserializer<MysqlStore>,
    {
        deserializer.deserialize_f64()
    }
}

impl Deserialize<MysqlStore> for String {
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Err>
    where
        D: Deserializer<MysqlStore>,
    {
        deserializer.deserialize_string()
    }
}

#[cfg(test)]
mod tests {
    use super::{super::Serializer, MysqlSerializer, QueryKind};

    #[test]
    fn test_serializer() {
        let mut serializer = MysqlSerializer::new(String::from("test"), QueryKind::Create);
        serializer.serialize_field("id", &3).unwrap();

        assert_eq!(serializer.cols, vec![String::from("id")]);
        assert_eq!(serializer.vals, vec![String::from("INT")]);

        let mut serializer = MysqlSerializer::new(String::from("test"), QueryKind::Delete);
        serializer.serialize_field("id", &3).unwrap();

        assert_eq!(serializer.cols, vec![String::from("id")]);
        assert_eq!(serializer.vals, vec![String::from("3")]);

        let mut serializer = MysqlSerializer::new(String::from("test"), QueryKind::Insert);
        serializer.serialize_field("id", &3).unwrap();
        serializer.serialize_field("name", "Hello World").unwrap();
        serializer.serialize_field("test", "panic'; DROP").unwrap();

        assert_eq!(
            serializer.cols,
            vec![
                String::from("id"),
                String::from("name"),
                String::from("test")
            ]
        );
        assert_eq!(
            serializer.vals,
            vec![
                String::from("3"),
                String::from("'Hello World'"),
                String::from("'panic\\'; DROP'")
            ]
        );
    }

    #[test]
    fn test_serializer_sql() {
        let mut serializer = MysqlSerializer::new(String::from("test"), QueryKind::Create);
        serializer.serialize_field("id", &3).unwrap();

        assert_eq!(serializer.into_sql(), "CREATE TABLE test (id INT)");

        let mut serializer = MysqlSerializer::new(String::from("test"), QueryKind::Create);
        serializer.serialize_field("id", &3).unwrap();
        serializer.serialize_field("name", &345i64).unwrap();

        assert_eq!(
            serializer.into_sql(),
            "CREATE TABLE test (id INT, name BIGINT)"
        );

        let mut serializer = MysqlSerializer::new(String::from("test"), QueryKind::Delete);
        serializer.serialize_field("id", &3).unwrap();

        assert_eq!(serializer.into_sql(), "DELETE FROM test WHERE id = 3");

        let mut serializer = MysqlSerializer::new(String::from("test"), QueryKind::Delete);
        serializer.serialize_field("id", &3).unwrap();
        serializer.serialize_field("name", &345i64).unwrap();

        assert_eq!(
            serializer.into_sql(),
            "DELETE FROM test WHERE id = 3 AND name = 345"
        );

        let mut serializer = MysqlSerializer::new(String::from("test"), QueryKind::Insert);
        serializer.serialize_field("id", &3).unwrap();

        assert_eq!(serializer.into_sql(), "INSERT INTO test (id) VALUES (3)");

        let mut serializer = MysqlSerializer::new(String::from("test"), QueryKind::Insert);
        serializer.serialize_field("id", &3).unwrap();
        serializer.serialize_field("name", &345i64).unwrap();

        assert_eq!(
            serializer.into_sql(),
            "INSERT INTO test (id, name) VALUES (3, 345)"
        );
    }
}
