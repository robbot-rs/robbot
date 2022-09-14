use crate::{bot::Result, Error};
use std::future::Future;
use tokio::sync::{mpsc, oneshot};
use tokio::task;

#[derive(Clone, Debug)]
pub struct Executor<C> {
    tx: mpsc::Sender<(C, oneshot::Sender<Result>)>,
}

impl<C> Executor<C> {
    pub fn from_fn<F>(f: fn(C) -> F) -> Self
    where
        F: Future<Output = Result> + Send + 'static,
        C: Send + 'static,
    {
        let (tx, mut rx) = mpsc::channel::<(C, oneshot::Sender<Result>)>(32);

        task::spawn(async move {
            while let Some((ctx, tx)) = rx.recv().await {
                task::spawn(async move {
                    let res = f(ctx).await;
                    let _ = tx.send(res);
                });
            }
        });

        Self { tx }
    }

    pub async fn call(&self, ctx: C) -> Result {
        let (tx, rx) = oneshot::channel();

        let _ = self.tx.send((ctx, tx)).await;

        match rx.await {
            Ok(val) => val,
            // Sender dropped.
            Err(_) => Err(Error::NoResponse),
        }
    }
}
