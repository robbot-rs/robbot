use super::event::{ClientEventHandler, EventHandler, ServerEventHandler};
use super::{Error, Result};
use crate::proto::{
    BotMessage, Command, CreateCommand, CreateExecutor, DeleteCommand, DiscordMessage, GetCommand,
    Message, MessageKind, SendMessage,
};

use super::raw::RawClient;

use tokio::net::{TcpStream, ToSocketAddrs};

#[derive(Clone)]
pub struct Client {
    inner: RawClient<Handler>,
}

impl Client {
    pub async fn connect<A>(addr: A) -> Result<Self>
    where
        A: ToSocketAddrs,
    {
        let socket = TcpStream::connect(addr).await.unwrap();

        let inner = RawClient::new(socket, Handler);

        Ok(Self { inner })
    }

    async fn send(&self, msg: Message) -> Result<Message> {
        self.inner.send(msg).await
    }

    pub fn bot(&self) -> BotClient {
        BotClient { client: &self }
    }

    pub fn discord(&self) -> DiscordClient {
        DiscordClient { client: &self }
    }
}

#[derive(Copy, Clone)]
pub struct BotClient<'a> {
    client: &'a Client,
}

impl<'a> BotClient<'a> {
    pub async fn create_command(&self, msg: CreateCommand) -> Result<CreateExecutor> {
        let msg = Message::new(MessageKind::Bot(BotMessage::CreateCommand(msg)));

        let resp = self.client.send(msg).await?;

        match resp.kind {
            MessageKind::Bot(ref msg) => match msg {
                BotMessage::CreateExecutor(_) => {
                    // SAFETY: Since we are already in this branch, we can safely
                    // go the same path without checking the enums.
                    let msg = match resp.kind {
                        MessageKind::Bot(msg) => match msg {
                            BotMessage::CreateExecutor(msg) => msg,
                            _ => unsafe { std::hint::unreachable_unchecked() },
                        },
                        _ => unsafe { std::hint::unreachable_unchecked() },
                    };

                    Ok(msg)
                }
                _ => Err(Error::UnexpectedMessage(resp)),
            },
            _ => Err(Error::UnexpectedMessage(resp)),
        }
    }

    pub async fn delete_command(&self, msg: DeleteCommand) -> Result<()> {
        let msg = Message::new(MessageKind::Bot(BotMessage::DeleteCommand(msg)));

        let resp = self.client.send(msg).await?;

        if resp.is_ok() {
            Ok(())
        } else {
            Err(Error::UnexpectedMessage(resp))
        }
    }

    pub async fn get_command(&self, msg: GetCommand) -> Result<Command> {
        let msg = Message::new(MessageKind::Bot(BotMessage::GetCommand(msg)));

        let resp = self.client.send(msg).await?;

        match resp.kind {
            MessageKind::Bot(ref msg) => match msg {
                BotMessage::Command(_) => {
                    let msg = match resp.kind {
                        MessageKind::Bot(msg) => match msg {
                            BotMessage::Command(msg) => msg,
                            _ => unsafe { std::hint::unreachable_unchecked() },
                        },
                        _ => unsafe { std::hint::unreachable_unchecked() },
                    };

                    Ok(msg)
                }
                _ => Err(Error::UnexpectedMessage(resp)),
            },
            _ => Err(Error::UnexpectedMessage(resp)),
        }
    }

    pub async fn list_root_commands(&self) -> Result<Vec<String>> {
        let msg = Message::new(MessageKind::Bot(BotMessage::ListRootCommand));

        let resp = self.client.send(msg).await?;

        match resp.kind {
            MessageKind::Bot(ref msg) => match msg {
                BotMessage::CommandList(_) => {
                    let msg = match resp.kind {
                        MessageKind::Bot(msg) => match msg {
                            BotMessage::CommandList(msg) => msg,
                            _ => unsafe { std::hint::unreachable_unchecked() },
                        },
                        _ => unsafe { std::hint::unreachable_unchecked() },
                    };

                    Ok(msg)
                }
                _ => Err(Error::UnexpectedMessage(resp)),
            },
            _ => Err(Error::UnexpectedMessage(resp)),
        }
    }
}

#[derive(Copy, Clone)]
pub struct DiscordClient<'a> {
    client: &'a Client,
}

impl<'a> DiscordClient<'a> {
    pub async fn send_message(&self, msg: SendMessage) -> Result<()> {
        let msg = Message::new(MessageKind::Discord(DiscordMessage::SendMessage(msg)));

        let resp = self.client.send(msg).await?;

        if resp.is_ok() {
            Ok(())
        } else {
            Err(Error::UnexpectedMessage(resp))
        }
    }
}

struct Handler;

impl EventHandler for Handler {}

impl ServerEventHandler for Handler {}

impl ClientEventHandler for Handler {}
