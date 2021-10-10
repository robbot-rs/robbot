//! Hooks
//!
use std::{
    borrow::Borrow,
    collections::{HashMap, HashSet},
    convert::TryFrom,
    hash::{Hash, Hasher},
};
use tokio::{
    sync::{broadcast, mpsc, oneshot},
    task,
};

/// Queue size for each event channel. If the queue size
/// is too small, some events may be dropped before received.
const QUEUE_SIZE: usize = 32;

#[derive(Clone, Debug)]
pub struct Hook {
    name: String,
    on_event: EventKind,
}

impl Hash for Hook {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        self.name.hash(state);
    }
}

impl PartialEq for Hook {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for Hook {}

impl Borrow<str> for Hook {
    fn borrow(&self) -> &str {
        &self.name
    }
}

pub struct InvalidEventKindError;

#[allow(clippy::all)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum EventKind {
    // ApplicationCommandCreate,
    // ApplicationCommandDelete,
    // ApplicationCommandUpdate,
    // CacheReady,
    // CategoryCreate,
    // CategoryDelete,
    ChannelCreate,
    ChannelDelete,
    // ChannelPinsUpdate,
    // ChannelUpdate,
    // GuildBanAddition,
    // GuildBanRemoval,
    // GuildCreate,
    // GuildDelete,
    // GuildEmojisUpdate,
    // GuildIntegrationsUpdate,
    GuildMemberAddition,
    GuildMemberRemoval,
    GuildMemberUpdate,
    // GuildMembersChunk,
    // GuildRoleCreate,
    // GuildRoleDelete,
    // GuildRoleUpdate,
    // GuildUnavaliable,
    // GuildUpdate,
    // IntegrationCreate,
    // IntegrationDelete,
    // InteractionCreate,
    // InviteCreate,
    // InviteDelete,
    Message,
    // MessageDelete,
    // MessageDeleteBulk,
    // MessageUpdate,
    // PresenceReplace,
    // PresenceUpdate,
    ReactionAdd,
    ReactionRemove,
    ReactionRemoveAll,
    // Ready,
    // Resume,
    // SharedStageUpdate,
    // StageInstanceCreate,
    // StageInstanceDelete,
    // StageInstanceUpdate,
    // ThreadCreate,
    // ThreadDelete,
    // ThreadListSync,
    // ThreadMemberUpdate,
    // ThreadMembersUpdate,
    // ThreadUpdate,
    // TypingStart,
    // Unknown,
    // UserUpdate,
    // VoiceServerUpdate,
    // VoiceStateUpdate,
    // WebhookUpdate,
}

impl TryFrom<&str> for EventKind {
    type Error = InvalidEventKindError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s {
            "ChannelCreate" => Ok(Self::ChannelCreate),
            "ChannelDelete" => Ok(Self::ChannelDelete),
            "GuildMemberAddition" => Ok(Self::GuildMemberAddition),
            "GuildMemberRemoval" => Ok(Self::GuildMemberRemoval),
            "GuildMemberUpdate" => Ok(Self::GuildMemberUpdate),
            "Message" => Ok(Self::Message),
            "ReactionAdd" => Ok(Self::ReactionAdd),
            "ReactionRemove" => Ok(Self::ReactionRemove),
            "ReactionRemoveAll" => Ok(Self::ReactionRemoveAll),
            _ => Err(InvalidEventKindError),
        }
    }
}

#[derive(Clone)]
pub(crate) enum Event {
    ChannelCreate(Box<crate::bot::ChannelCreateContext>),
    ChannelDelete(Box<crate::bot::ChannelDeleteContext>),
    GuildMemberAddition(Box<crate::bot::GuildMemberAdditionContext>),
    GuildMemberRemoval(Box<crate::bot::GuildmemberRemovalContext>),
    GuildMemberUpdate(Box<crate::bot::GuildMemberUpdateContext>),
    Message(Box<crate::bot::MessageContext>),
    ReactionAdd(Box<crate::bot::ReactionAddContext>),
    ReactionRemove(Box<crate::bot::ReactionRemoveContext>),
    ReactionRemoveAll(Box<crate::bot::ReactionRemoveAllContext>),
}

#[derive(Clone, Debug, Default)]
struct InnerHookController {
    hooks: HashSet<Hook>,
    channels: HashMap<EventKind, broadcast::Sender<Event>>,
}

impl InnerHookController {
    fn send_event(&self, data: Event) {
        let event = match data {
            Event::ChannelCreate(_) => EventKind::ChannelCreate,
            Event::ChannelDelete(_) => EventKind::ChannelDelete,
            Event::GuildMemberAddition(_) => EventKind::GuildMemberAddition,
            Event::GuildMemberRemoval(_) => EventKind::GuildMemberRemoval,
            Event::GuildMemberUpdate(_) => EventKind::GuildMemberUpdate,
            Event::Message(_) => EventKind::Message,
            Event::ReactionAdd(_) => EventKind::ReactionAdd,
            Event::ReactionRemove(_) => EventKind::ReactionRemove,
            Event::ReactionRemoveAll(_) => EventKind::ReactionRemoveAll,
        };

        if let Some(tx) = self.channels.get(&event) {
            let _ = tx.send(data);
        }
    }

    fn add_hook(&mut self, hook: Hook) {
        self.hooks.insert(hook);
    }

    /// Get a new [`broadcast::Receiver`] for receiving evnets
    /// of the type `event`.
    fn get_receiver(&mut self, event: EventKind) -> broadcast::Receiver<Event> {
        match self.channels.get(&event) {
            Some(tx) => tx.subscribe(),
            None => {
                let (tx, rx) = broadcast::channel(QUEUE_SIZE);

                self.channels.insert(event, tx);

                rx
            }
        }
    }

    fn handle_message(&mut self, msg: Message) {
        match msg {
            Message::AddHook(hook) => self.add_hook(hook),
            Message::SendEvent(data) => self.send_event(*data),
            Message::GetReceiver(event, tx) => {
                let rx = self.get_receiver(event);
                let _ = tx.send(rx);
            }
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
    SendEvent(Box<Event>),
    GetReceiver(EventKind, oneshot::Sender<broadcast::Receiver<Event>>),
}

#[derive(Clone, Debug)]
pub(crate) struct HookController {
    tx: mpsc::Sender<Message>,
}

impl HookController {
    pub fn new() -> Self {
        let inner = InnerHookController::default();

        Self { tx: inner.start() }
    }

    pub async fn add_hook(&self, hook: Hook) -> broadcast::Receiver<Event> {
        let event = hook.on_event;

        let _ = self.tx.send(Message::AddHook(hook)).await;

        self.get_receiver(event).await
    }

    pub async fn send_event(&self, data: Event) {
        let _ = self.tx.send(Message::SendEvent(Box::new(data))).await;
    }

    pub async fn get_receiver(&self, event: EventKind) -> broadcast::Receiver<Event> {
        let (tx, rx) = oneshot::channel();

        let _ = self.tx.send(Message::GetReceiver(event, tx)).await;

        rx.await.unwrap()
    }
}

impl Default for HookController {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::{EventKind, Hook, InnerHookController};

    #[test]
    fn test_hook_controller() {
        fn hooks_as_vec<'life0>(hook_controller: &InnerHookController) -> Vec<Hook> {
            hook_controller
                .hooks
                .iter()
                .map(|hook| hook.clone())
                .collect()
        }

        let mut hook_controller = InnerHookController::default();

        hook_controller.add_hook(Hook {
            name: String::from("test1"),
            on_event: EventKind::Message,
        });

        assert_eq!(
            hooks_as_vec(&hook_controller),
            vec![Hook {
                name: String::from("test1"),
                on_event: EventKind::Message,
            }]
        );
    }
}
