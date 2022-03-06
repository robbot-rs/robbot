use super::{Error, Result};
use crate::proto::{
    Command, CreateCommand, CreateExecutor, DeleteCommand, GetCommand, MessageEvent, SendMessage,
};

use async_trait::async_trait;

pub trait EventHandler: ClientEventHandler + ServerEventHandler {}

#[async_trait]
pub trait ClientEventHandler: Sized {
    async fn event_message(&self, _msg: MessageEvent) {}
}

#[async_trait]
pub trait ServerEventHandler: Sized {
    async fn bot_create_command(&self, _msg: CreateCommand) -> Result<CreateExecutor> {
        Err(Error::Unimplemented)
    }

    async fn bot_delete_command(&self, _msg: DeleteCommand) -> Result<()> {
        Err(Error::Unimplemented)
    }

    async fn bot_get_command(&self, _msg: GetCommand) -> Result<Command> {
        Err(Error::Unimplemented)
    }

    async fn bot_list_root_commands(&self) -> Result<Vec<String>> {
        Err(Error::Unimplemented)
    }

    /// The handler for [`SendMessage`] client events.
    async fn discord_send_message(&self, _msg: SendMessage) -> Result<()> {
        Err(Error::Unimplemented)
    }
}

#[derive(Copy, Clone, Debug)]
pub(super) struct DefaultHandler;

impl EventHandler for DefaultHandler {}

impl ClientEventHandler for DefaultHandler {}

impl ServerEventHandler for DefaultHandler {}
