use crate::state::State;
use robbot::arguments::{CommandArguments, OwnedArguments};
use serenity::client::Context as RawContext;
use std::{ops::Deref, sync::Arc};

use robbot::model::channel::Message;

use robbot::hook::{HookEvent, HookEventWrapper};

/// An alias for `Context<Message>`. This context is received by
/// command handlers.
pub type MessageContext = Context<Message>;

/// An alias for `Context<()>`. This context is received by tasks.
pub type TaskContext = Context<()>;

#[derive(Clone, Debug)]
pub struct Context<T>
where
    T: Send + Sync,
{
    inner: robbot::Context<T, Arc<State>>,
    pub args: CommandArguments,
}

impl<T> Context<T>
where
    T: Send + Sync,
{
    pub fn new(raw_ctx: RawContext, state: Arc<State>, event: T) -> Self {
        Self {
            inner: robbot::Context {
                raw_ctx,
                event,
                state,
            },
            args: CommandArguments::from(OwnedArguments::new()),
        }
    }

    pub fn new_with_args(
        raw_ctx: RawContext,
        state: Arc<State>,
        event: T,
        args: CommandArguments,
    ) -> Self {
        Self {
            inner: robbot::Context {
                raw_ctx,
                event,
                state,
            },
            args,
        }
    }

    pub fn swap<U>(self, event: U) -> (Context<U>, T)
    where
        U: Send + Sync,
    {
        let Self { inner, args } = self;

        let (inner, old_event) = inner.swap(event);

        (Context { inner, args }, old_event)
    }
}

impl<T> Deref for Context<T>
where
    T: Send + Sync,
{
    type Target = robbot::Context<T, Arc<State>>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> HookEventWrapper for Context<T>
where
    T: Send + Sync + HookEvent,
{
    type HookEvent = T;
}
