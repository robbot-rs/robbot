use super::{DataDescriptor, DataQuery, Store, StoreData};

use tokio::sync::RwLock;

use std::sync::Arc;

/// A *lazy* wrapper around a store `S`. The connection to the store is only opened
/// when it is first used.
#[derive(Clone, Debug)]
pub struct LazyStore<S>
where
    S: Store + Clone,
{
    inner: Arc<InnerLazyStore<S>>,
}

impl<S> LazyStore<S>
where
    S: Store + Clone,
{
    pub fn new(uri: &str) -> Self {
        let inner = InnerLazyStore::new(uri);

        Self {
            inner: Arc::new(inner),
        }
    }

    pub async fn connect(uri: &str) -> Result<Self, S::Error> {
        let store = S::connect(uri).await?;
        let inner = InnerLazyStore::new_connected(uri, store);

        Ok(Self {
            inner: Arc::new(inner),
        })
    }

    pub async fn create<T, D>(&self, descriptor: D) -> Result<(), S::Error>
    where
        T: StoreData<S> + Send + Sync + 'static,
        D: DataDescriptor<T, S> + Send + Sync,
    {
        let store = self.inner.store().await?;

        store.create(descriptor).await
    }

    pub async fn delete<T, Q>(&self, query: Q) -> Result<(), S::Error>
    where
        T: StoreData<S> + Send + Sync + 'static,
        Q: DataQuery<T, S> + Send,
    {
        let store = self.inner.store().await?;

        store.delete(query).await
    }

    pub async fn get<T, D, Q>(&self, descriptor: D, query: Q) -> Result<Vec<T>, S::Error>
    where
        T: StoreData<S> + Send + Sync + 'static,
        D: DataDescriptor<T, S> + Send + Sync,
        Q: DataQuery<T, S> + Send,
    {
        let store = self.inner.store().await?;

        store.get(descriptor, query).await
    }

    pub async fn get_all<T, D>(&self, descriptor: D) -> Result<Vec<T>, S::Error>
    where
        T: StoreData<S> + Send + Sync + 'static,
        D: DataDescriptor<T, S> + Send + Sync,
    {
        let store = self.inner.store().await?;

        store.get_all(descriptor).await
    }

    pub async fn get_one<T, D, Q>(&self, descriptor: D, query: Q) -> Result<Option<T>, S::Error>
    where
        T: StoreData<S> + Send + Sync + 'static,
        D: DataDescriptor<T, S> + Send + Sync,
        Q: DataQuery<T, S> + Send + Sync,
    {
        let store = self.inner.store().await?;

        store.get_one(descriptor, query).await
    }

    pub async fn insert<T>(&self, data: T) -> Result<(), S::Error>
    where
        T: StoreData<S> + Send + Sync + 'static,
    {
        let store = self.inner.store().await?;

        store.insert(data).await
    }
}

#[derive(Debug)]
struct InnerLazyStore<S>
where
    S: Store + Clone,
{
    uri: String,
    store: RwLock<Option<S>>,
}

impl<S> InnerLazyStore<S>
where
    S: Store + Clone,
{
    fn new<T>(uri: T) -> Self
    where
        T: ToString,
    {
        Self {
            uri: uri.to_string(),
            store: RwLock::new(None),
        }
    }

    // Creates a new `InnerLazyStore` with an already open connection to the store.
    fn new_connected<T>(uri: T, store: S) -> Self
    where
        T: ToString,
    {
        Self {
            uri: uri.to_string(),
            store: RwLock::new(Some(store)),
        }
    }

    async fn store(&self) -> Result<S, S::Error> {
        {
            let inner = self.store.read().await;
            if inner.is_some() {
                return Ok(inner.clone().unwrap());
            }
        }

        let mut inner = self.store.write().await;

        // In case that another thread alredy opened the connection while
        // dropping the read guard.
        if inner.is_some() {
            return Ok(inner.clone().unwrap());
        }

        let store = S::connect(&self.uri).await;
        *inner = Some(store?);

        Ok(inner.clone().unwrap())
    }
}
