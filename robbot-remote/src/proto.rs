//! # robbot-remote Protocol
//!
//! When creating an implementation using the protocol the following conditions must be met:
//! - Requests and responses use the same message format.
//! - A request is always answered by either one or no responses at all, depending on the request.
//! - Messages may or may not be received in transmission order.
//! - Requests and responses must be associable to each other.
//! - Message transmission must be reliable.
//!
use robbot::model::id::ChannelId;
use robbot::{Decode, Encode};

use serde::{Deserialize, Serialize};

use std::sync::atomic::{AtomicU32, Ordering};

pub const REVISION: u8 = 0;

const ID: AtomicU32 = AtomicU32::new(0);

/// A single request/response message. [`Message::revision`] is used
/// to distinguish incompatible api revisions.
#[derive(Clone, Debug, Serialize, Deserialize, Encode, Decode)]
pub struct Message {
    pub id: u32,
    pub revision: u8,
    pub kind: MessageKind,
}

impl Message {
    pub fn new(kind: MessageKind) -> Self {
        Self {
            id: ID.fetch_add(1, Ordering::SeqCst),
            revision: REVISION,
            kind,
        }
    }

    /// Returns `true` if the Message is Ok.
    pub fn is_ok(&self) -> bool {
        match self.kind {
            MessageKind::Meta(msg) => match msg {
                MetaMessage::Ok => true,
                _ => false,
            },
            _ => false,
        }
    }

    pub fn is_proto_error(&self) -> bool {
        match self.kind {
            MessageKind::Meta(msg) => match msg {
                MetaMessage::ProtoError => true,
                _ => false,
            },
            _ => false,
        }
    }

    pub fn is_error(&self) -> bool {
        match self.kind {
            MessageKind::Meta(msg) => match msg {
                MetaMessage::Error => true,
                _ => false,
            },
            _ => false,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Encode, Decode)]
pub enum MessageKind {
    /// Reserved for internal traffic like keepalive.
    Meta(MetaMessage),
    /// Messages that query or mutate the bot state (excl. Store).
    Bot(BotMessage),
    /// Messages that interact with the store.
    Store,
    /// Event dispatcher messages.
    Event(EventMessage),
    /// Message for discord event calls.
    Discord(DiscordMessage),
}

impl MessageKind {
    pub fn is_meta(&self) -> bool {
        match self {
            Self::Meta(_) => true,
            _ => false,
        }
    }

    pub fn is_event(&self) -> bool {
        match self {
            Self::Event(_) => true,
            _ => false,
        }
    }

    pub fn is_response(&self) -> bool {
        match self {
            Self::Meta(msg) => match msg {
                MetaMessage::Ok => true,
                MetaMessage::ProtoError => true,
                MetaMessage::Error => true,
                _ => false,
            },
            Self::Bot(msg) => match msg {
                BotMessage::Command(_) => true,
                BotMessage::CommandList(_) => true,
                BotMessage::CreateExecutor(_) => true,
                _ => false,
            },
            _ => false,
        }
    }
}

// =====================
// === Meta Messages ===
// =====================

#[derive(Copy, Clone, Debug, Serialize, Deserialize, Encode, Decode)]
pub enum MetaMessage {
    /// Indicates that the request was received and processes successfully, but no response
    /// is given.
    Ok,
    Keepalive,
    /// Indicates the client closes the connection. Acknoleged by an [`MetaMessage::Ok`].
    Close,
    /// A protocol error. This may be a malformed request or a client/server-only request
    /// from the wrong source (client-only request from server, server-only request
    /// from client).
    ProtoError,
    Error,
}

// ====================
// === Bot Messages ===
// ====================

#[derive(Clone, Debug, Serialize, Deserialize, Encode, Decode)]
pub enum BotMessage {
    /// A single [`Command`] response without any executor. This is a response-only
    /// message.
    Command(Command),
    /// A list of command names. This is a response-only message.
    CommandList(Vec<String>),
    /// The message for adding a new command. See [`CreateCommand`].
    CreateCommand(CreateCommand),
    /// The message for removing a command. See [`DeleteCommand`].
    DeleteCommand(DeleteCommand),
    /// The message for getting a command. See [`GetCommand`].
    GetCommand(GetCommand),
    /// The message for getting a [`Self::CommandList`] of root command names.
    ListRootCommand,
    // EXECUTOR
    CreateExecutor(CreateExecutor),
}

/// The message for adding a new command. This message is only sent
/// by clients.
#[derive(Clone, Debug, Serialize, Deserialize, Encode, Decode)]
pub struct CreateCommand {
    pub name: String,
    pub description: String,
    pub usage: String,
    pub example: String,
    pub guild_only: bool,
    pub permissions: Vec<String>,

    /// The insertion path for the new command. An
    /// empty string represents the `None` value.
    pub path: String,
}

/// The message for removing a command. This message is only sent
/// by clients.
#[derive(Clone, Debug, Serialize, Deserialize, Encode, Decode)]
pub struct DeleteCommand {
    pub ident: String,
    /// The path to the commands root command.
    pub path: String,
}

/// The message for getting an existing command. This message is only
/// sent by clients.
#[derive(Clone, Debug, Serialize, Deserialize, Encode, Decode)]
pub struct GetCommand {
    pub ident: String,
    pub path: String,
}

/// The message returned by commands that create a new executor.
#[derive(Clone, Debug, Serialize, Deserialize, Encode, Decode)]
pub struct CreateExecutor {
    pub ident: ExecutorIdent,
}

#[derive(Clone, Debug, Serialize, Deserialize, Encode, Decode)]
pub struct Command {
    pub name: String,
    pub description: String,
    pub usage: String,
    pub example: String,
    pub guild_only: bool,
    pub permissions: Vec<String>,
    pub sub_commands: Vec<String>,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, Encode, Decode)]
pub struct ExecutorIdent(pub u64);

// ======================
// === Event Messages ===
// ======================

#[derive(Clone, Debug, Serialize, Deserialize, Encode, Decode)]
pub enum EventMessage {
    Message(MessageEvent),
    Task { ident: ExecutorIdent },
}

#[derive(Clone, Debug, Serialize, Deserialize, Encode, Decode)]
pub struct MessageEvent {
    pub ident: ExecutorIdent,
    pub data: robbot::model::channel::Message,
}

// ========================
// === Discord Messages ===
// ========================

#[derive(Clone, Debug, Serialize, Deserialize, Encode, Decode)]
pub enum DiscordMessage {
    SendMessage(SendMessage),
}

#[derive(Clone, Debug, Serialize, Deserialize, Encode, Decode)]
pub struct SendMessage {
    pub channel_id: ChannelId,
    pub data: robbot::builder::CreateMessage,
}

/// Creates a new [`Message`] with a [`MetaMessage::Ok`].
pub fn ok() -> Message {
    Message::new(MessageKind::Meta(MetaMessage::Ok))
}

/// Creates a new [`Message`] with a [`MetaMessage::ProtoError`].
pub fn proto_error() -> Message {
    Message::new(MessageKind::Meta(MetaMessage::ProtoError))
}

/// Creates a new [`Message`] with a [`MetaMessage::Error`].
pub fn error() -> Message {
    Message::new(MessageKind::Meta(MetaMessage::Error))
}

#[cfg(test)]
mod tests {
    use super::{GetCommand, MessageKind, MetaMessage};
    use bincode::{DefaultOptions, Options};

    #[test]
    fn test_serialize_messsage_kind() {
        let msg = MessageKind::Meta(MetaMessage::Ok);

        let bytes = DefaultOptions::new()
            .with_big_endian()
            .with_fixint_encoding()
            .serialize(&msg)
            .unwrap();

        let output = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];

        assert_eq!(bytes, output);
    }

    // #[test]
    // fn test_deserialize_message_kind() {
    //     let bytes = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01];

    //     let msg: MessageKind = DefaultOptions::new()
    //         .with_big_endian()
    //         .with_fixint_encoding()
    //         .deserialize(&bytes)
    //         .unwrap();
    // }

    #[test]
    fn test_serialize_get_command() {
        let msg = GetCommand {
            ident: String::from("Hello"),
            path: String::from("World!"),
        };

        let bytes = DefaultOptions::new()
            .with_big_endian()
            .with_fixint_encoding()
            .serialize(&msg)
            .unwrap();

        let output = [
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x05, b'H', b'e', b'l', b'l', b'o', 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x06, b'W', b'o', b'r', b'l', b'd', b'!',
        ];

        assert_eq!(bytes, output);
    }

    #[test]
    fn test_deserialize_get_command() {
        let bytes = [
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x05, b'H', b'e', b'l', b'l', b'o', 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x06, b'W', b'o', b'r', b'l', b'd', b'!',
        ];

        let msg: GetCommand = DefaultOptions::new()
            .with_big_endian()
            .with_fixint_encoding()
            .deserialize(&bytes)
            .unwrap();

        assert_eq!(msg.ident, String::from("Hello"));
        assert_eq!(msg.path, String::from("World!"));
    }
}
