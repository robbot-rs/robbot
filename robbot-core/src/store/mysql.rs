use async_trait::async_trait;
use futures::TryStreamExt;
use robbot::store::{
    DataQuery, Deserialize, Deserializer, Serialize, Serializer, Store, StoreData,
};
use sqlx::{
    mysql::{MySqlPool, MySqlRow},
    Row,
};

use std::fmt::{self, Display, Formatter};

pub type Error = sqlx::Error;

/// A Store using the MySQL database.
#[derive(Clone)]
pub struct MysqlStore {
    pool: MySqlPool,
}

#[async_trait]
impl Store for MysqlStore {
    type Error = Error;
    type Serializer = MysqlSerializer;

    async fn connect(uri: &str) -> Result<Self, Error> {
        let pool = MySqlPool::connect(uri).await?;

        Ok(Self { pool })
    }

    async fn create<T>(&self) -> Result<(), Error>
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

    async fn delete<T, Q>(&self, query: Q) -> Result<(), Error>
    where
        T: StoreData<Self> + Default + Send,
        Q: DataQuery<T, Self> + Send,
    {
        let table_name = T::resource_name();

        let mut serializer = MysqlSerializer::new(table_name, QueryKind::Delete);
        serializer.enable_condition();

        query.serialize(&mut serializer).unwrap();

        let sql = serializer.into_sql();
        log::debug!("[MySQL] Executing SQL delete query: \"{}\"", sql);

        sqlx::query(&sql).execute(&self.pool).await?;

        Ok(())
    }

    async fn get<T, Q>(&self, query: Q) -> Result<Vec<T>, Error>
    where
        T: StoreData<Self> + Default + Send,
        Q: DataQuery<T, Self> + Send,
    {
        let data = T::default();
        let table_name = T::resource_name();

        let mut serializer = MysqlSerializer::new(table_name, QueryKind::Select);

        data.serialize(&mut serializer).unwrap();

        serializer.enable_condition();
        query.serialize(&mut serializer).unwrap();

        let sql = serializer.into_sql();
        log::debug!("[MySQL] Executing SQL select query: \"{}\"", sql);

        let mut rows = sqlx::query(&sql).fetch(&self.pool);

        let mut entries = Vec::new();

        while let Some(row) = rows.try_next().await? {
            let mut deserializer = MysqlDeserializer::new(row);
            let data = T::deserialize(&mut deserializer).unwrap();

            entries.push(data);
        }

        Ok(entries)
    }

    async fn get_all<T>(&self) -> Result<Vec<T>, Error>
    where
        T: StoreData<Self> + Default + Send,
    {
        let table_name = T::resource_name();
        let data = T::default();

        let mut serializer = MysqlSerializer::new(table_name, QueryKind::Select);
        data.serialize(&mut serializer).unwrap();

        let sql = serializer.into_sql();
        log::debug!("[MySQL] Executing SQL select query: \"{}\"", sql);

        let mut rows = sqlx::query(&sql).fetch(&self.pool);

        let mut entries = Vec::new();

        while let Some(row) = rows.try_next().await? {
            let mut deserializer = MysqlDeserializer::new(row);
            let data = T::deserialize(&mut deserializer).unwrap();

            entries.push(data);
        }

        Ok(entries)
    }

    async fn get_one<T, Q>(&self, query: Q) -> Result<Option<T>, Error>
    where
        T: StoreData<Self> + Default + Send,
        Q: DataQuery<T, Self> + Send,
    {
        let data = T::default();
        let table_name = T::resource_name();

        let mut serializer = MysqlSerializer::new(table_name, QueryKind::Select);

        data.serialize(&mut serializer).unwrap();

        serializer.enable_condition();
        query.serialize(&mut serializer).unwrap();

        let sql = serializer.into_sql();
        log::debug!("[MySQL] Executing SQL select query: \"{}\"", sql);

        let row = sqlx::query(&sql).fetch_one(&self.pool).await.unwrap();

        let mut deserializer = MysqlDeserializer::new(row);
        let data = T::deserialize(&mut deserializer).unwrap();

        Ok(Some(data))
    }

    async fn insert<T>(&self, data: T) -> Result<(), Error>
    where
        T: StoreData<Self> + Send,
    {
        let table_name = T::resource_name();

        let mut serializer = MysqlSerializer::new(table_name, QueryKind::Insert);
        data.serialize(&mut serializer).unwrap();

        let sql = serializer.into_sql();
        log::debug!("[MySQL] Executing SQL insert query: \"{}\"", sql);

        sqlx::query(&sql).execute(&self.pool).await?;

        Ok(())
    }
}

/// Type of the sql query being built.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum QueryKind {
    Create,
    Delete,
    Insert,
    Select,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Query {
    Create {
        table_name: String,
        columns: Vec<String>,
        /// Type and attributes
        values: Vec<String>,
    },
    Delete {
        table_name: String,
        conditions: ConditionsExpr,
    },
    Insert {
        table_name: String,
        columns: Vec<String>,
        values: Vec<String>,
    },
    Select {
        table_name: String,
        columns: Vec<String>,
        conditions: ConditionsExpr,
    },
}

impl Display for Query {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::Create {
                table_name,
                columns,
                values,
            } => write!(
                f,
                "CREATE TABLE IF NOT EXISTS {} ({})",
                table_name,
                columns
                    .iter()
                    .zip(values)
                    .map(|(column, value)| format!("{} {}", column, value))
                    .collect::<Vec<String>>()
                    .join(",")
            ),
            Self::Delete {
                table_name,
                conditions,
            } => write!(f, "DELETE FROM {}{}", table_name, conditions),
            Self::Insert {
                table_name,
                columns,
                values,
            } => write!(
                f,
                "INSERT INTO {} ({}) VALUES ({})",
                table_name,
                columns.join(","),
                values.join(",")
            ),
            Self::Select {
                table_name,
                columns,
                conditions,
            } => write!(
                f,
                "SELECT {} FROM {}{}",
                columns.join(","),
                table_name,
                conditions
            ),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Comparator {
    /// The equality comparator `=`.
    Eq,
    // /// The not equal comparator `!=`.
    // Ne,
    // /// The greater than comparator `>`.
    // Gt,
    // /// The greater than or equal comparator `>=`.
    // Ge,
    // /// The less than comparator `<`.
    // Lt,
    // /// The less than or equal comparator `<=`.
    // Le,
}

impl Display for Comparator {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let string = match self {
            Self::Eq => "=",
            // Self::Ne => "!=",
            // Self::Gt => ">",
            // Self::Ge => ">=",
            // Self::Lt => "<",
            // Self::Le => "<=",
        };

        write!(f, "{}", string)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ConditionsExpr {
    conditions: Vec<Condition>,
}

impl ConditionsExpr {
    fn new() -> Self {
        Self {
            conditions: Vec::new(),
        }
    }

    fn push(&mut self, condition: Condition) {
        self.conditions.push(condition);
    }
}

impl Display for ConditionsExpr {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        if self.conditions.is_empty() {
            return Ok(());
        }

        write!(f, " WHERE {}", self.conditions[0])?;

        let iter = self.conditions.iter().skip(1);

        for condition in iter {
            write!(f, " AND {}", condition)?;
        }

        Ok(())
    }
}

/// A single SQL filtering condition.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Condition {
    column: String,
    value: String,
    comparator: Comparator,
}

impl Condition {
    fn new() -> Self {
        Self {
            column: String::new(),
            value: String::new(),
            comparator: Comparator::Eq,
        }
    }
}

impl Display for Condition {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{} {} {}", self.column, self.comparator, self.value)
    }
}

/// A [`Serializer`] for building SQL queries for use
/// with [`MysqlStore`].
pub struct MysqlSerializer {
    query: Query,
    condition: Option<Condition>,
    // kind: QueryKind,
    // cols: Vec<String>,
    // vals: Vec<String>,
    // /// Conditions
    // where_cols: Vec<String>,
    // where_vals: Vec<String>,
    // /// Whether new serialized values should be targeted
    // /// as the where statement.
    // target_where: bool
}

impl MysqlSerializer {
    fn new(table_name: String, kind: QueryKind) -> Self {
        Self {
            query: match kind {
                QueryKind::Create => Query::Create {
                    table_name,
                    columns: Vec::new(),
                    values: Vec::new(),
                },
                QueryKind::Delete => Query::Delete {
                    table_name,
                    conditions: ConditionsExpr::new(),
                },
                QueryKind::Insert => Query::Insert {
                    table_name,
                    columns: Vec::new(),
                    values: Vec::new(),
                },
                QueryKind::Select => Query::Select {
                    table_name,
                    columns: Vec::new(),
                    conditions: ConditionsExpr::new(),
                },
            },
            condition: None,
        }
    }

    /// Writes a column name into the query. The usage depends on the
    /// type of the query. If the condition section of the query is reached,
    /// the column is instead used in the conditional expression.
    fn write_column<T>(&mut self, column: T)
    where
        T: ToString,
    {
        let val = column.to_string();

        match &mut self.condition {
            Some(ref mut condition) => condition.column = val,
            None => match &mut self.query {
                Query::Create {
                    ref mut columns, ..
                } => columns.push(val),
                Query::Delete { .. } => unreachable!(),
                Query::Insert {
                    ref mut columns, ..
                } => columns.push(val),
                Query::Select {
                    ref mut columns, ..
                } => columns.push(val),
            },
        }
    }

    /// Writes a value into the query. The usage depends on the type of the
    /// query. If the condition section of the query is reached, the value
    /// is instead used in the conditional expression, finalizing the conditional
    /// expression and opening a new one.
    fn write_value<T>(&mut self, value: T)
    where
        T: ToString,
    {
        let val = value.to_string();

        match &mut self.condition {
            Some(ref mut condition) => {
                condition.value = val;

                let condition = self.condition.take().unwrap();

                // Push the complete condition to the query.
                match &mut self.query {
                    Query::Create { .. } => unreachable!(),
                    Query::Delete {
                        ref mut conditions, ..
                    } => conditions.push(condition),
                    Query::Insert { .. } => unreachable!(),
                    Query::Select {
                        ref mut conditions, ..
                    } => conditions.push(condition),
                }

                // Create a new empty condition.
                self.condition = Some(Condition::new());
            }
            None => match &mut self.query {
                Query::Create { ref mut values, .. } => values.push(val),
                Query::Delete { .. } => unreachable!(),
                Query::Insert { ref mut values, .. } => values.push(val),
                Query::Select { .. } => (),
            },
        }
    }

    fn into_sql(self) -> String {
        format!("{}", self.query)
    }

    /// Mark the [`MysqlSerializer`] to interpret any following
    /// serialized values to be used in the conditional section
    /// of the SQL statement.
    fn enable_condition(&mut self) {
        self.condition = Some(Condition::new());
    }
}

impl Serializer<MysqlStore> for MysqlSerializer {
    type Error = Error;

    fn serialize_bool(&mut self, v: bool) -> Result<(), Self::Error> {
        let val = match self.query {
            Query::Create { .. } => "BOOLEAN",
            _ => match v {
                false => "FALSE",
                true => "TRUE",
            },
        };

        self.write_value(val);
        Ok(())
    }

    fn serialize_i8(&mut self, v: i8) -> Result<(), Self::Error> {
        match self.query {
            Query::Create { .. } => self.write_value("TINYINT"),
            _ => self.write_value(v),
        }

        Ok(())
    }

    fn serialize_i16(&mut self, v: i16) -> Result<(), Self::Error> {
        match self.query {
            Query::Create { .. } => self.write_value("SMALLINT"),
            _ => self.write_value(v),
        }

        Ok(())
    }

    fn serialize_i32(&mut self, v: i32) -> Result<(), Self::Error> {
        match self.query {
            Query::Create { .. } => self.write_value("INT"),
            _ => self.write_value(v),
        }

        Ok(())
    }

    fn serialize_i64(&mut self, v: i64) -> Result<(), Self::Error> {
        match self.query {
            Query::Create { .. } => self.write_value("BIGINT"),
            _ => self.write_value(v),
        }

        Ok(())
    }

    fn serialize_u8(&mut self, v: u8) -> Result<(), Self::Error> {
        match self.query {
            Query::Create { .. } => self.write_value("TINYINT UNSIGNED"),
            _ => self.write_value(v),
        }

        Ok(())
    }

    fn serialize_u16(&mut self, v: u16) -> Result<(), Self::Error> {
        match self.query {
            Query::Create { .. } => self.write_value("SMALLINT UNSIGNED"),
            _ => self.write_value(v),
        }

        Ok(())
    }

    fn serialize_u32(&mut self, v: u32) -> Result<(), Self::Error> {
        match self.query {
            Query::Create { .. } => self.write_value("INT UNSIGNED"),
            _ => self.write_value(v),
        }

        Ok(())
    }

    fn serialize_u64(&mut self, v: u64) -> Result<(), Self::Error> {
        match self.query {
            Query::Create { .. } => self.write_value("BIGINT UNSIGNED"),
            _ => self.write_value(v),
        }

        Ok(())
    }

    fn serialize_f32(&mut self, v: f32) -> Result<(), Self::Error> {
        match self.query {
            Query::Create { .. } => self.write_value("FLOAT"),
            _ => self.write_value(v),
        }

        Ok(())
    }

    fn serialize_f64(&mut self, v: f64) -> Result<(), Self::Error> {
        match self.query {
            Query::Create { .. } => self.write_value("DOUBLE"),
            _ => self.write_value(v),
        }

        Ok(())
    }

    fn serialize_str(&mut self, v: &str) -> Result<(), Self::Error> {
        match self.query {
            Query::Create { .. } => self.write_value("TEXT"),
            _ => self.write_value(format!("'{}'", v.replace("'", "\\'"))),
        }

        Ok(())
    }

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize<MysqlStore>,
    {
        self.write_column(key);
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
    type Error = Error;

    fn deserialize_bool(&mut self) -> Result<bool, Self::Error> {
        let v = self.row.try_get(self.column())?;

        Ok(v)
    }

    fn deserialize_i8(&mut self) -> Result<i8, Self::Error> {
        let v = self.row.try_get(self.column())?;

        Ok(v)
    }

    fn deserialize_i16(&mut self) -> Result<i16, Self::Error> {
        let v = self.row.try_get(self.column())?;

        Ok(v)
    }

    fn deserialize_i32(&mut self) -> Result<i32, Self::Error> {
        let v = self.row.try_get(self.column())?;

        Ok(v)
    }

    fn deserialize_i64(&mut self) -> Result<i64, Self::Error> {
        let v = self.row.try_get(self.column())?;

        Ok(v)
    }

    fn deserialize_u8(&mut self) -> Result<u8, Self::Error> {
        let v = self.row.try_get(self.column())?;

        Ok(v)
    }

    fn deserialize_u16(&mut self) -> Result<u16, Self::Error> {
        let v = self.row.try_get(self.column())?;

        Ok(v)
    }

    fn deserialize_u32(&mut self) -> Result<u32, Self::Error> {
        let v = self.row.try_get(self.column())?;

        Ok(v)
    }

    fn deserialize_u64(&mut self) -> Result<u64, Self::Error> {
        let v = self.row.try_get(self.column())?;

        Ok(v)
    }

    fn deserialize_f32(&mut self) -> Result<f32, Self::Error> {
        let v = self.row.try_get(self.column())?;

        Ok(v)
    }

    fn deserialize_f64(&mut self) -> Result<f64, Self::Error> {
        let v = self.row.try_get(self.column())?;

        Ok(v)
    }

    fn deserialize_string(&mut self) -> Result<String, Self::Error> {
        let v = self.row.try_get(self.column())?;

        Ok(v)
    }

    fn deserialize_field<T>(&mut self, key: &'static str) -> Result<T, Self::Error>
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
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
    where
        S: Serializer<MysqlStore>,
    {
        serializer.serialize_bool(*self)
    }
}

impl Serialize<MysqlStore> for i8 {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
    where
        S: Serializer<MysqlStore>,
    {
        serializer.serialize_i8(*self)
    }
}

impl Serialize<MysqlStore> for i16 {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
    where
        S: Serializer<MysqlStore>,
    {
        serializer.serialize_i16(*self)
    }
}

impl Serialize<MysqlStore> for i32 {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
    where
        S: Serializer<MysqlStore>,
    {
        serializer.serialize_i32(*self)
    }
}

impl Serialize<MysqlStore> for i64 {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
    where
        S: Serializer<MysqlStore>,
    {
        serializer.serialize_i64(*self)
    }
}

impl Serialize<MysqlStore> for u8 {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
    where
        S: Serializer<MysqlStore>,
    {
        serializer.serialize_u8(*self)
    }
}

impl Serialize<MysqlStore> for u16 {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
    where
        S: Serializer<MysqlStore>,
    {
        serializer.serialize_u16(*self)
    }
}

impl Serialize<MysqlStore> for u32 {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
    where
        S: Serializer<MysqlStore>,
    {
        serializer.serialize_u32(*self)
    }
}

impl Serialize<MysqlStore> for u64 {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
    where
        S: Serializer<MysqlStore>,
    {
        serializer.serialize_u64(*self)
    }
}

impl Serialize<MysqlStore> for f32 {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
    where
        S: Serializer<MysqlStore>,
    {
        serializer.serialize_f32(*self)
    }
}

impl Serialize<MysqlStore> for f64 {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
    where
        S: Serializer<MysqlStore>,
    {
        serializer.serialize_f64(*self)
    }
}

impl Serialize<MysqlStore> for str {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
    where
        S: Serializer<MysqlStore>,
    {
        serializer.serialize_str(self)
    }
}

impl Serialize<MysqlStore> for String {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
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
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error>
    where
        D: Deserializer<MysqlStore>,
    {
        deserializer.deserialize_bool()
    }
}

impl Deserialize<MysqlStore> for i8 {
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error>
    where
        D: Deserializer<MysqlStore>,
    {
        deserializer.deserialize_i8()
    }
}

impl Deserialize<MysqlStore> for i16 {
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error>
    where
        D: Deserializer<MysqlStore>,
    {
        deserializer.deserialize_i16()
    }
}

impl Deserialize<MysqlStore> for i32 {
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error>
    where
        D: Deserializer<MysqlStore>,
    {
        deserializer.deserialize_i32()
    }
}

impl Deserialize<MysqlStore> for i64 {
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error>
    where
        D: Deserializer<MysqlStore>,
    {
        deserializer.deserialize_i64()
    }
}

impl Deserialize<MysqlStore> for u8 {
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error>
    where
        D: Deserializer<MysqlStore>,
    {
        deserializer.deserialize_u8()
    }
}

impl Deserialize<MysqlStore> for u16 {
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error>
    where
        D: Deserializer<MysqlStore>,
    {
        deserializer.deserialize_u16()
    }
}

impl Deserialize<MysqlStore> for u32 {
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error>
    where
        D: Deserializer<MysqlStore>,
    {
        deserializer.deserialize_u32()
    }
}

impl Deserialize<MysqlStore> for u64 {
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error>
    where
        D: Deserializer<MysqlStore>,
    {
        deserializer.deserialize_u64()
    }
}

impl Deserialize<MysqlStore> for f32 {
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error>
    where
        D: Deserializer<MysqlStore>,
    {
        deserializer.deserialize_f32()
    }
}

impl Deserialize<MysqlStore> for f64 {
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error>
    where
        D: Deserializer<MysqlStore>,
    {
        deserializer.deserialize_f64()
    }
}

impl Deserialize<MysqlStore> for String {
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error>
    where
        D: Deserializer<MysqlStore>,
    {
        deserializer.deserialize_string()
    }
}

#[cfg(test)]
mod tests {
    use super::{Comparator, Condition, ConditionsExpr, MysqlSerializer, Query, QueryKind};
    use robbot::store::Serializer;

    #[test]
    fn test_serializer() {
        let mut serializer = MysqlSerializer::new(String::from("test"), QueryKind::Create);
        serializer.serialize_field("id", &3).unwrap();

        assert_eq!(
            serializer.query,
            Query::Create {
                table_name: String::from("test"),
                columns: vec![String::from("id")],
                values: vec![String::from("INT")],
            }
        );

        let mut serializer = MysqlSerializer::new(String::from("test"), QueryKind::Delete);
        serializer.enable_condition();
        serializer.serialize_field("id", &3).unwrap();

        assert_eq!(
            serializer.query,
            Query::Delete {
                table_name: String::from("test"),
                conditions: ConditionsExpr {
                    conditions: vec![Condition {
                        column: String::from("id"),
                        value: String::from("3"),
                        comparator: Comparator::Eq,
                    }]
                }
            }
        );

        let mut serializer = MysqlSerializer::new(String::from("test"), QueryKind::Insert);
        serializer.serialize_field("id", &3).unwrap();
        serializer.serialize_field("name", "Hello World").unwrap();
        serializer.serialize_field("test", "panic'; DROP").unwrap();

        assert_eq!(
            serializer.query,
            Query::Insert {
                table_name: String::from("test"),
                columns: vec![
                    String::from("id"),
                    String::from("name"),
                    String::from("test")
                ],
                values: vec![
                    String::from("3"),
                    String::from("'Hello World'"),
                    String::from("'panic\\'; DROP'")
                ]
            }
        );

        let mut serializer = MysqlSerializer::new(String::from("test"), QueryKind::Select);
        serializer.serialize_field("id", &0).unwrap();
        serializer.serialize_field("name", &0).unwrap();

        assert_eq!(
            serializer.query,
            Query::Select {
                table_name: String::from("test"),
                columns: vec![String::from("id"), String::from("name")],
                conditions: ConditionsExpr {
                    conditions: Vec::new()
                }
            }
        );

        let mut serializer = MysqlSerializer::new(String::from("test"), QueryKind::Select);
        serializer.serialize_field("id", &0).unwrap();
        serializer.serialize_field("name", &0).unwrap();
        serializer.enable_condition();
        serializer.serialize_field("id", &3).unwrap();
        serializer.serialize_field("name", "abc").unwrap();

        assert_eq!(
            serializer.query,
            Query::Select {
                table_name: String::from("test"),
                columns: vec![String::from("id"), String::from("name")],
                conditions: ConditionsExpr {
                    conditions: vec![
                        Condition {
                            column: String::from("id"),
                            value: String::from("3"),
                            comparator: Comparator::Eq,
                        },
                        Condition {
                            column: String::from("name"),
                            value: String::from("'abc'"),
                            comparator: Comparator::Eq,
                        },
                    ],
                },
            }
        );
    }

    #[test]
    fn test_serializer_sql() {
        let mut serializer = MysqlSerializer::new(String::from("test"), QueryKind::Create);
        serializer.serialize_field("id", &3).unwrap();

        assert_eq!(
            serializer.into_sql(),
            "CREATE TABLE IF NOT EXISTS test (id INT)"
        );

        let mut serializer = MysqlSerializer::new(String::from("test"), QueryKind::Create);
        serializer.serialize_field("id", &3).unwrap();
        serializer.serialize_field("name", &345i64).unwrap();

        assert_eq!(
            serializer.into_sql(),
            "CREATE TABLE IF NOT EXISTS test (id INT,name BIGINT)"
        );

        let mut serializer = MysqlSerializer::new(String::from("test"), QueryKind::Delete);
        serializer.enable_condition();
        serializer.serialize_field("id", &3).unwrap();

        assert_eq!(serializer.into_sql(), "DELETE FROM test WHERE id = 3");

        let mut serializer = MysqlSerializer::new(String::from("test"), QueryKind::Delete);
        serializer.enable_condition();
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
            "INSERT INTO test (id,name) VALUES (3,345)"
        );

        let mut serializer = MysqlSerializer::new(String::from("test"), QueryKind::Select);
        serializer.serialize_field("id", &0).unwrap();
        serializer.serialize_field("name", &0).unwrap();

        assert_eq!(serializer.into_sql(), "SELECT id,name FROM test");

        let mut serializer = MysqlSerializer::new(String::from("test"), QueryKind::Select);
        serializer.serialize_field("id", &0).unwrap();
        serializer.serialize_field("name", &0).unwrap();
        serializer.enable_condition();
        serializer.serialize_field("id", &3).unwrap();
        serializer.serialize_field("name", "abc").unwrap();

        assert_eq!(
            serializer.into_sql(),
            "SELECT id,name FROM test WHERE id = 3 AND name = 'abc'"
        )
    }
}
