pub mod mysql;

use robbot::store::{DataQuery, Store, StoreData};
use std::{
    error,
    fmt::{self, Display, Formatter},
    result,
};

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub struct Error(Box<dyn error::Error + Send + 'static>);

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<T> From<T> for Error
where
    T: error::Error + Send + 'static,
{
    fn from(err: T) -> Self {
        Self(Box::new(err))
    }
}

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
    S::Error: Send + 'static,
{
    pub async fn new(uri: &str) -> Result<Self> {
        Ok(Self {
            store: S::connect(uri).await?,
        })
    }

    pub async fn create<T>(&self) -> Result<()>
    where
        T: StoreData<S> + Default + Send,
    {
        self.store.create::<T>().await?;
        Ok(())
    }

    /// Delete all items matching the query.
    pub async fn delete<T, Q>(&self, query: Q) -> Result<()>
    where
        T: StoreData<S> + Default + Send,
        Q: DataQuery<T, S> + Send,
    {
        self.store.delete(query).await?;
        Ok(())
    }

    /// Returns all items from the store that match the query. Using
    /// `None` as the query returns all items avaliable.
    ///
    /// If you only need a single item, use [`Self::get_one`].
    pub async fn get<T, Q>(&self, query: Q) -> Result<Vec<T>>
    where
        T: StoreData<S> + Send + Default,
        Q: DataQuery<T, S> + Send,
    {
        let data = self.store.get(query).await?;
        Ok(data)
    }

    pub async fn get_all<T>(&self) -> Result<Vec<T>>
    where
        T: StoreData<S> + Send + Default,
    {
        let data = self.store.get_all().await?;
        Ok(data)
    }

    /// Returns the first item matching the query.
    ///
    /// If you need all items matching the query, use [`Self::get`].
    pub async fn get_one<T, Q>(&self, query: Q) -> Result<T>
    where
        T: StoreData<S> + Send + Default,
        Q: DataQuery<T, S> + Send + Sync,
    {
        let data = self.store.get_one(query).await?;
        Ok(data)
    }

    pub async fn insert<T>(&self, data: T) -> Result<()>
    where
        T: StoreData<S> + Send,
    {
        self.store.insert(data).await?;
        Ok(())
    }
}
