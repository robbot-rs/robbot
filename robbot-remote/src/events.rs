use crate::proto::{self, proto_error};

use async_trait::async_trait;

pub trait EventHandler: ClientEventHandler + ServerEventHandler {}

#[async_trait]
pub trait ClientEventHandler {
    type Error: From<proto::Message>;

    async fn event_message(&self, _msg: proto::MessageEvent) -> Result<(), Self::Error> {
        Err(proto_error().into())
    }

    async fn event_task(&self, _msg: proto::ExecutorIdent) -> Result<(), Self::Error> {
        Err(proto_error().into())
    }
}

#[async_trait]
pub trait ServerEventHandler {
    type Error: From<proto::Message>;

    async fn bot_create_command(
        &self,
        _msg: proto::CreateCommand,
    ) -> Result<proto::CreateExecutor, Self::Error> {
        Err(proto_error().into())
    }

    async fn bot_delete_command(&self, _msg: proto::DeleteCommand) -> Result<(), Self::Error> {
        Err(proto_error().into())
    }

    async fn bot_get_command(
        &self,
        _msg: proto::GetCommand,
    ) -> Result<proto::Command, Self::Error> {
        Err(proto_error().into())
    }

    async fn bot_list_root_commands(&self) -> Result<Vec<String>, Self::Error> {
        Err(proto_error().into())
    }
}
