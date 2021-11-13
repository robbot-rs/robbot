use crate::{bot::Result, context::Context};
use async_trait::async_trait;
use std::future::Future;

#[async_trait]
pub trait Executor<T>
where
    T: Context + Send,
{
    /// Create a new `Executor` from a future or static async
    /// function.
    fn from_fn<F>(f: fn(T) -> F) -> Self
    where
        F: Future<Output = Result> + Send + 'static,
        T: 'static;

    /// Call the executor with the context.
    async fn send(&self, ctx: T) -> Result;
}
