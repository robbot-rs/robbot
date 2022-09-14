use crate::context::Context;

use robbot::executor::Executor;
use robbot::hook::{EventData, EventKind, HookEvent};
use robbot::hook::{GuildMemberUpdateData, MessageData};

use tokio::sync::{broadcast, mpsc, oneshot};
use tokio::task;

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

const QUEUE_SIZE: usize = 32;

#[derive(Clone, Debug)]
pub struct Hook {
    pub name: String,
    pub on_event: EventKind,
}

/// An alias for `Context<MessageData>`.
pub type MessageContext = Context<MessageData>;

/// An alias for `Context<GuildMemberUpdateContext>`.
pub type GuildMemberUpdateContext = Context<GuildMemberUpdateData>;

struct InnerHookController {
    hooks: Vec<Hook>,
    channels: HashMap<EventKind, broadcast::Sender<(EventData, Context<()>)>>,
    context: Arc<RwLock<Option<Context<()>>>>,
}

impl InnerHookController {
    pub fn new(ctx: Arc<RwLock<Option<Context<()>>>>) -> Self {
        Self {
            hooks: Vec::new(),
            channels: HashMap::new(),
            context: ctx,
        }
    }

    fn dispatch_event(&self, data: EventData) {
        let event_kind = data.kind();

        if let Some(tx) = self.channels.get(&event_kind) {
            let context = self.context.read().unwrap();
            // Don't dispatch events while context is `None`.
            if context.is_none() {
                return;
            }

            let _ = tx.send((data, context.clone().unwrap()));
        }
    }

    fn get_receiver(&mut self, kind: EventKind) -> broadcast::Receiver<(EventData, Context<()>)> {
        match self.channels.get(&kind) {
            Some(tx) => tx.subscribe(),
            None => {
                let (tx, rx) = broadcast::channel(QUEUE_SIZE);
                self.channels.insert(kind, tx);
                rx
            }
        }
    }

    fn add_hook(&mut self, hook: Hook) {
        self.hooks.push(hook);
    }

    fn list_hooks(&self) -> Vec<Hook> {
        self.hooks.clone()
    }

    fn handle_message(&mut self, msg: Message) {
        match msg {
            Message::AddHook(hook) => self.add_hook(hook),
            Message::ListHooks(tx) => {
                let hooks = self.list_hooks();
                let _ = tx.send(hooks);
            }
            Message::GetReceiver(kind, tx) => {
                let rx = self.get_receiver(kind);
                let _ = tx.send(rx);
            }
            Message::DispatchEvent(event) => self.dispatch_event(event),
        }
    }

    fn start(mut self) -> mpsc::Sender<Message> {
        let (tx, mut rx) = mpsc::channel(32);

        task::spawn(async move {
            while let Some(msg) = rx.recv().await {
                self.handle_message(msg);
            }
        });

        tx
    }
}

enum Message {
    AddHook(Hook),
    ListHooks(oneshot::Sender<Vec<Hook>>),
    GetReceiver(
        EventKind,
        oneshot::Sender<broadcast::Receiver<(EventData, Context<()>)>>,
    ),
    DispatchEvent(EventData),
}

#[derive(Debug)]
pub struct HookController {
    tx: mpsc::Sender<Message>,
}

impl HookController {
    pub fn new(ctx: Arc<RwLock<Option<Context<()>>>>) -> Self {
        Self {
            tx: InnerHookController::new(ctx).start(),
        }
    }

    pub async fn add_hook(&self, hook: Hook) -> broadcast::Receiver<(EventData, Context<()>)> {
        let event = hook.on_event;
        let _ = self.tx.send(Message::AddHook(hook)).await;
        self.get_receiver(event).await
    }

    pub async fn list_hooks(&self) -> Vec<Hook> {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Message::ListHooks(tx)).await;
        rx.await.unwrap()
    }

    pub async fn get_receiver(
        &self,
        kind: EventKind,
    ) -> broadcast::Receiver<(EventData, Context<()>)> {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Message::GetReceiver(kind, tx)).await;
        rx.await.unwrap()
    }

    pub async fn dispatch_event<T>(&self, event: T)
    where
        T: Into<EventData>,
    {
        let _ = self.tx.send(Message::DispatchEvent(event.into())).await;
    }
}

pub struct HookExecutor<T>
where
    T: HookEvent + Send + Sync + 'static,
{
    rx: broadcast::Receiver<(EventData, Context<()>)>,
    executor: Executor<Context<T>>,
}

impl<T> HookExecutor<T>
where
    T: HookEvent + Send + Sync + 'static,
{
    pub fn new(
        rx: broadcast::Receiver<(EventData, Context<()>)>,
        executor: Executor<Context<T>>,
    ) -> Self {
        Self { rx, executor }
    }

    pub fn run(mut self) {
        tokio::task::spawn(async move {
            while let Ok((data, ctx)) = self.rx.recv().await {
                if let Ok(event) = T::try_from(data) {
                    let (ctx, _) = ctx.swap(event);

                    match self.executor.call(ctx).await {
                        Ok(_) => (),
                        Err(err) => {
                            log::error!("Hook failed to execute: {:?}", err);
                        }
                    }
                }
            }
        });
    }
}
