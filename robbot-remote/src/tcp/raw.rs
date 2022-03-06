use super::event::EventHandler;
use super::{Error, Result};

use crate::proto::{
    error, ok, proto_error, BotMessage, CreateCommand, DeleteCommand, DiscordMessage, EventMessage,
    GetCommand, Message, MessageKind, MetaMessage, SendMessage,
};

use robbot::remote::{Decode, Decoder, Encode, Encoder};

use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::sync::{mpsc, oneshot};
use tokio::{select, task};

use std::collections::HashMap;
use std::marker::PhantomData;
use std::sync::Arc;

#[derive(Debug)]
pub(super) struct RawClient<H>
where
    H: EventHandler + Send + Sync,
{
    pub tx: mpsc::Sender<(Message, Option<oneshot::Sender<Message>>)>,
    _marker: PhantomData<Arc<H>>,
}

impl<H> RawClient<H>
where
    H: EventHandler + Send + Sync + 'static,
{
    pub fn new(socket: TcpStream, event_handler: H) -> Self {
        let (tx, rx) = mpsc::channel(32);

        Self::new_with_channel(socket, event_handler, tx, rx)
    }

    pub fn new_with_channel(
        socket: TcpStream,
        event_handler: H,
        tx: mpsc::Sender<(Message, Option<oneshot::Sender<Message>>)>,
        mut rx: mpsc::Receiver<(Message, Option<oneshot::Sender<Message>>)>,
    ) -> Self {
        let (read_tx, mut read_rx) = mpsc::channel::<Message>(32);

        let (reader, mut writer) = socket.into_split();
        let mut reader = BufReader::new(reader);

        let event_handler = Arc::new(event_handler);

        // Reader task
        task::spawn(async move {
            loop {
                let len = match reader.read_u64().await {
                    Ok(len) => len as usize,
                    Err(err) => {
                        log::error!("Failed to read message length: {:?}", err);
                        return;
                    }
                };

                let mut buf = vec![0; len];
                if let Err(err) = reader.read_exact(&mut buf).await {
                    log::error!("Failed to read message body: {:?}", err);
                    return;
                }

                log::trace!("Read: {:?}", buf);

                let mut decoder = Decoder::new(&*buf);
                let msg = match Message::decode(&mut decoder) {
                    Ok(msg) => msg,
                    Err(err) => {
                        log::error!("Failed to decode message: {:?}", err);
                        return;
                    }
                };

                if let Err(_) = read_tx.send(msg).await {
                    return;
                }
            }
        });

        // Writer task
        task::spawn(async move {
            let mut txs = HashMap::new();

            loop {
                // Wait for an message, either from the other side of the connection or from
                // the local sender.
                select! {
                    // Wait for requests from the sender.
                    res = rx.recv() => {
                        // No more requests to received, close the connection.
                        if res.is_none() {
                            // TODO: Close the connection properly
                            break;
                        }

                        let (msg, tx) = res.unwrap();

                        let mut buf = Vec::new();
                        let mut encoder = Encoder::new(&mut buf);
                        msg.encode(&mut encoder).unwrap();

                        log::trace!("Sending message: {:?}", msg);
                        log::trace!("Encoded {:?}", buf);

                        writer.write_u64(buf.len() as u64).await.unwrap();
                        writer.write_all(&buf).await.unwrap();

                        if let Some(tx) = tx {
                            txs.insert(msg.id, tx);
                        }
                    }

                    // Respond to requests from the other side.
                    res = read_rx.recv() => {
                        // read_tx read a `None` value, meaning the reader has dropped.
                        if res.is_none() {
                            break;
                        }

                        let msg = res.unwrap();

                        log::trace!("Received message: {:?}", msg);

                        if txs.contains_key(&msg.id) {
                            let tx = txs.remove(&msg.id).unwrap();

                            let _ = tx.send(msg);
                            continue;
                        }

                        let resp = match handle_request(event_handler.as_ref(), msg).await {
                            Ok(resp) => match resp {
                                Some(resp) => resp,
                                None => continue,
                            },
                            Err(err) => {
                                log::error!("{:?}", err);
                                error()
                            }
                        };

                        let mut buf = Vec::new();
                        let mut encoder = Encoder::new(&mut buf);
                        resp.encode(&mut encoder).unwrap();

                        log::trace!("Sending message: {:?}", resp);
                        log::trace!("Encoded {:?}", buf);

                        writer.write_u64(buf.len() as u64).await.unwrap();
                        writer.write_all(&buf).await.unwrap();
                    }
                }
            }
        });

        Self {
            tx,
            _marker: PhantomData,
        }
    }

    pub async fn send(&self, msg: Message) -> Result<Message> {
        // Do not send meta messages or responses.
        #[cfg(debug_assertions)]
        {
            assert!(!msg.kind.is_meta());
            assert!(!msg.kind.is_response());
        }

        log::trace!("Sending message: {:?}", msg);

        let (tx, rx) = oneshot::channel();

        let _ = self.tx.send((msg, Some(tx))).await;

        match rx.await {
            Ok(resp) => Ok(resp),
            Err(_) => Err(Error::NoResponse),
        }
    }

    /// Sends a [`Message`] without awaiting a response for it.
    pub async fn dispatch(&self, msg: Message) {
        #[cfg(debug_assertions)]
        {
            assert!(!msg.kind.is_meta());
            assert!(!msg.kind.is_response());
        }

        let _ = self.tx.send((msg, None)).await;
    }
}

impl<H> Clone for RawClient<H>
where
    H: EventHandler + Send + Sync,
{
    fn clone(&self) -> Self {
        Self {
            tx: self.tx.clone(),
            _marker: PhantomData,
        }
    }
}

/// Handles a single request from a client using the handler `H`. Returns `Ok(None)` when
/// a request should not be responded to.
async fn handle_request<H>(handler: &H, request: Message) -> Result<Option<Message>>
where
    H: EventHandler + Send + Sync,
{
    Ok(match request.kind {
        MessageKind::Meta(msg) => match msg {
            // [`MetaMessage::Ok`] is not a request.
            MetaMessage::Ok => Some(proto_error()),
            MetaMessage::Keepalive => Some(ok()),
            MetaMessage::Close => unimplemented!(),
            // [`MetaMessage::ProtoError`] is not a request.
            MetaMessage::ProtoError => Some(proto_error()),
            // [`MetaMessage::Error`] is not a request.
            MetaMessage::Error => Some(proto_error()),
        },
        MessageKind::Bot(msg) => match msg {
            // [`BotMessage::Command`] is not a request.
            BotMessage::Command(_) => Some(proto_error()),
            // [`BotMessage::CommandList`] is not a request.
            BotMessage::CommandList(_) => Some(proto_error()),
            BotMessage::CreateCommand(msg) => Some(bot_create_command(handler, msg).await?),
            BotMessage::DeleteCommand(msg) => Some(bot_delete_command(handler, msg).await?),
            BotMessage::GetCommand(msg) => Some(bot_get_command(handler, msg).await?),
            BotMessage::ListRootCommand => Some(bot_list_root_commands(handler).await?),
            BotMessage::CreateExecutor(_) => unimplemented!(),
        },
        MessageKind::Store => unimplemented!(),
        // Events are never responded to
        MessageKind::Event(msg) => match msg {
            EventMessage::Message(msg) => {
                handler.event_message(msg).await;
                None
            }
            EventMessage::Task { ident: _ } => unimplemented!(),
        },
        MessageKind::Discord(msg) => match msg {
            DiscordMessage::SendMessage(msg) => Some(discord_send_message(handler, msg).await?),
        },
    })
}

async fn bot_create_command<H>(handler: &H, req: CreateCommand) -> Result<Message>
where
    H: EventHandler + Send + Sync,
{
    let resp = handler.bot_create_command(req).await?;

    Ok(Message::new(MessageKind::Bot(BotMessage::CreateExecutor(
        resp,
    ))))
}

async fn bot_delete_command<H>(handler: &H, req: DeleteCommand) -> Result<Message>
where
    H: EventHandler + Send + Sync,
{
    handler.bot_delete_command(req).await?;
    Ok(ok())
}

async fn bot_get_command<H>(handler: &H, req: GetCommand) -> Result<Message>
where
    H: EventHandler + Send + Sync,
{
    let resp = handler.bot_get_command(req).await?;

    Ok(Message::new(MessageKind::Bot(BotMessage::Command(resp))))
}

async fn bot_list_root_commands<H>(handler: &H) -> Result<Message>
where
    H: EventHandler + Send + Sync,
{
    let resp = handler.bot_list_root_commands().await?;

    Ok(Message::new(MessageKind::Bot(BotMessage::CommandList(
        resp,
    ))))
}

async fn discord_send_message<H>(handler: &H, req: SendMessage) -> Result<Message>
where
    H: EventHandler + Send + Sync,
{
    handler.discord_send_message(req).await?;
    Ok(ok())
}
