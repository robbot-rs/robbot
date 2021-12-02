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

#[derive(Clone, Debug)]
pub struct MainStore<S>
where
    S: Store,
{
    main: Option<S>,
}

impl<S> MainStore<S>
where
    S: Store,
    S::Error: Send + 'static,
{
    pub async fn new(uri: &str) -> Result<Self> {
        Ok(Self {
            main: Some(S::connect(uri).await?),
        })
    }

    pub async fn connect(&mut self, uri: &str) -> Result<()> {
        self.main = Some(S::connect(uri).await?);
        Ok(())
    }

    pub fn close(&mut self) {
        self.main = None;
    }

    pub async fn create<T>(&self) -> Result<()>
    where
        T: StoreData<S> + Default + Send + Sync + 'static,
    {
        self.main_store().create::<T>().await?;
        Ok(())
    }

    pub async fn delete<T, Q>(&self, query: Q) -> Result<()>
    where
        T: StoreData<S> + Default + Send + Sync + 'static,
        Q: DataQuery<T, S> + Send,
    {
        self.main_store().delete(query).await?;
        Ok(())
    }

    pub async fn get<T, Q>(&self, query: Q) -> Result<Vec<T>>
    where
        T: StoreData<S> + Send + Sync + Default + 'static,
        Q: DataQuery<T, S> + Send,
    {
        let data = self.main_store().get(query).await?;
        Ok(data)
    }

    pub async fn get_all<T>(&self) -> Result<Vec<T>>
    where
        T: StoreData<S> + Send + Sync + Default + 'static,
    {
        let data = self.main_store().get_all().await?;
        Ok(data)
    }

    pub async fn get_one<T, Q>(&self, query: Q) -> Result<Option<T>>
    where
        T: StoreData<S> + Send + Sync + Default + 'static,
        Q: DataQuery<T, S> + Send + Sync,
    {
        let data = self.main_store().get_one(query).await?;
        Ok(data)
    }

    pub async fn insert<T>(&self, data: T) -> Result<()>
    where
        T: StoreData<S> + Send + Sync + 'static,
    {
        self.main_store().insert(data).await?;
        Ok(())
    }

    fn main_store(&self) -> &S {
        self.main.as_ref().unwrap()
    }

    pub fn is_connected(&self) -> bool {
        self.main.is_some()
    }
}

impl<S> Default for MainStore<S>
where
    S: Store,
{
    fn default() -> Self {
        Self { main: None }
    }
}
