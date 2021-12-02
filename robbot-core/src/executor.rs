use async_trait::async_trait;
use robbot::{Context, Error, Result};
use std::future::Future;
use tokio::{
    sync::{mpsc, oneshot},
    task,
};

#[derive(Clone, Debug)]
pub struct Executor<T>
where
    T: Context + Send,
{
    tx: mpsc::Sender<(T, oneshot::Sender<Result>)>,
}

impl<T> Executor<T>
where
    T: Context + Send,
{
    pub fn new(tx: mpsc::Sender<(T, oneshot::Sender<Result>)>) -> Self {
        Self { tx }
    }
}

#[async_trait]
impl<T> robbot::executor::Executor<T> for Executor<T>
where
    T: Context + Send,
{
    fn from_fn<F>(f: fn(T) -> F) -> Self
    where
        F: Future<Output = Result> + Send + 'static,
        T: 'static,
    {
        let (tx, mut rx) = mpsc::channel::<(T, oneshot::Sender<Result>)>(32);

        task::spawn(async move {
            while let Some((data, tx)) = rx.recv().await {
                task::spawn(async move {
                    let res = f(data).await;
                    let _ = tx.send(res);
                });
            }
        });

        Self::new(tx)
    }

    async fn send(&self, ctx: T) -> Result {
        let (tx, rx) = oneshot::channel();

        let _ = self.tx.send((ctx, tx)).await;

        match rx.await {
            Ok(val) => val,
            // The sender was dropped. This likely
            // happened because the command panicked.
            Err(_) => Err(Error::NoResponse),
        }
    }
}
