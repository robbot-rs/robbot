use crate::bot::{self, Error};
use std::future::Future;
use tokio::{
    sync::{mpsc, oneshot},
    task,
};

#[derive(Clone, Debug)]
pub struct Executor<T> {
    tx: mpsc::Sender<(T, oneshot::Sender<bot::Result>)>,
}

impl<T> Executor<T> {
    pub fn new(tx: mpsc::Sender<(T, oneshot::Sender<bot::Result>)>) -> Self {
        Self { tx }
    }

    pub async fn send(&self, ctx: T) -> bot::Result {
        let (tx, rx) = oneshot::channel();

        let _ = self.tx.send((ctx, tx)).await;

        match rx.await {
            Ok(val) => val,
            Err(_) => Err(Error::NoResponse),
        }
    }
}

impl<T> Executor<T>
where
    T: Send + Sync + 'static,
{
    pub fn from_fn<F>(f: fn(T) -> F) -> Self
    where
        F: Future<Output = bot::Result> + Send + Sync + 'static,
    {
        let (tx, mut rx) = mpsc::channel::<(T, oneshot::Sender<bot::Result>)>(32);

        task::spawn(async move {
            while let Some((data, tx)) = rx.recv().await {
                task::spawn(async move {
                    let res = f(data).await;
                    let _ = tx.send(res);
                });
            }
        });

        Self { tx }
    }
}
