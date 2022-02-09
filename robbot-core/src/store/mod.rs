pub mod mysql;

use robbot::store::lazy::LazyStore;
use robbot::store::{DataQuery, Store, StoreData};

use std::error::Error as StdError;
use std::fmt::{self, Display, Formatter};
use std::result;

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub struct Error(Box<dyn StdError + Send + Sync + 'static>);

impl AsRef<dyn StdError + Send + 'static> for Error {
    fn as_ref(&self) -> &(dyn StdError + Send + 'static) {
        &*self.0
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<T> From<T> for Error
where
    T: StdError + Send + Sync + 'static,
{
    fn from(err: T) -> Self {
        Self(Box::new(err))
    }
}

impl From<Error> for robbot::Error {
    fn from(err: Error) -> Self {
        Self::Other(err.0)
    }
}

#[derive(Clone, Debug)]
pub struct MainStore<S>
where
    S: Store + Clone,
{
    store: LazyStore<S>,
}

impl<S> MainStore<S>
where
    S: Store + Clone,
    S::Error: Send + Sync + 'static,
{
    pub fn new(uri: &str) -> Self {
        Self {
            store: LazyStore::new(uri),
        }
    }

    pub async fn create<T>(&self) -> Result<()>
    where
        T: StoreData<S> + Send + Sync + 'static,
        T::DataDescriptor: Default + Send + Sync,
    {
        let descriptor = T::DataDescriptor::default();

        self.store.create::<T, _>(descriptor).await?;
        Ok(())
    }

    pub async fn delete<T, Q>(&self, query: Q) -> Result<()>
    where
        T: StoreData<S> + Send + Sync + 'static,
        Q: DataQuery<T, S> + Send,
    {
        self.store.delete(query).await?;
        Ok(())
    }

    pub async fn get<T, Q>(&self, query: Q) -> Result<Vec<T>>
    where
        T: StoreData<S> + Send + Sync + 'static,
        T::DataDescriptor: Default + Send + Sync,
        Q: DataQuery<T, S> + Send,
    {
        let descriptor = T::DataDescriptor::default();

        let data = self.store.get(descriptor, query).await?;
        Ok(data)
    }

    pub async fn get_all<T>(&self) -> Result<Vec<T>>
    where
        T: StoreData<S> + Send + Sync + 'static,
        T::DataDescriptor: Default + Send + Sync,
    {
        let descriptor = T::DataDescriptor::default();

        let data = self.store.get_all(descriptor).await?;
        Ok(data)
    }

    pub async fn get_one<T, Q>(&self, query: Q) -> Result<Option<T>>
    where
        T: StoreData<S> + Send + Sync + 'static,
        T::DataDescriptor: Default + Send + Sync,
        Q: DataQuery<T, S> + Send + Sync,
    {
        let descriptor = T::DataDescriptor::default();

        let data = self.store.get_one(descriptor, query).await?;
        Ok(data)
    }

    pub async fn insert<T>(&self, data: T) -> Result<()>
    where
        T: StoreData<S> + Send + Sync + 'static,
    {
        self.store.insert(data).await?;
        Ok(())
    }
}
