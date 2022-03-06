use crate::proto::{
    Command, CreateCommand, CreateExecutor, DeleteCommand, EventMessage, ExecutorIdent, GetCommand,
    Message, MessageEvent, MessageKind, SendMessage,
};

use super::event::{ClientEventHandler, EventHandler, ServerEventHandler};
use super::raw::RawClient;
use super::{Error, Result};

// use super::client::Client;

use robbot::command::Command as _;
use robbot::context::Context;
use robbot_core::command::AddOptions;
use robbot_core::context::MessageContext;
use robbot_core::executor::Executor;
use robbot_core::router::parse_args;
use robbot_core::state::State;

use async_trait::async_trait;
use tokio::net::{TcpListener, ToSocketAddrs};
use tokio::sync::{mpsc, oneshot};
use tokio::task;

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

pub struct Server {
    listener: TcpListener,
    state: Arc<State>,
}

impl Server {
    pub async fn bind<A>(addr: A, state: Arc<State>) -> Result<Self>
    where
        A: ToSocketAddrs,
    {
        let listener = TcpListener::bind(addr).await?;

        log::info!("Listening for connections on {}", listener.local_addr()?);

        Ok(Self { listener, state })
    }

    pub async fn start(&self) {
        loop {
            match self.listener.accept().await {
                Ok((socket, addr)) => {
                    log::debug!("Accepting new connection from {}", addr);

                    let state = self.state.clone();
                    task::spawn(async move {
                        let (tx, rx) = mpsc::channel(32);

                        let handler = Handler::new(state, tx.clone());

                        let client = RawClient::new_with_channel(socket, handler, tx, rx);

                        client.tx.closed().await;
                    });
                }
                Err(err) => {
                    log::debug!("Failed to accpet new connection: {:?}", err)
                }
            }
        }
    }
}

// impl Server {
//     pub async fn bind<A>(addr: A, state: Arc<State>) -> io::Result<Self>
//     where
//         A: ToSocketAddrs,
//     {
//         let listener = TcpListener::bind(addr).await?;

//         task::spawn(async move {
//             loop {
//                 select! {
//                     res = listener.accept() => match res {
//                         Ok((socket, addr)) => {
//                             let state = state.clone();

//                             log::debug!("[TCP] Accepting new connection from {}", addr);

//                             task::spawn(async move {
//                                 let client = Client::from_stream(socket).unwrap();

//                                 let handler = Handler::new(state, client.tx.clone());

//                                 client.set_event_handler(handler);
//                                 client.tx.closed().await;
//                             });
//                         }
//                         Err(err) => {
//                             log::error!("[TCP] Failed to accept new connection: {:?}", err);
//                         }
//                     }
//                 }
//             }
//         });

//         Ok(Self {})
//     }
// }

struct Handler {
    state: Arc<State>,
    executor_id: Arc<AtomicU64>,
    tx: mpsc::Sender<(Message, Option<oneshot::Sender<Message>>)>,
}

// pub struct Handler {
//     state: Arc<State>,
//     // Counter for the current executor id.

//     writer: mpsc::Sender<Message>,
// }

impl Handler {
    pub fn new(
        state: Arc<State>,
        tx: mpsc::Sender<(Message, Option<oneshot::Sender<Message>>)>,
    ) -> Self {
        Self {
            state,
            executor_id: Arc::new(AtomicU64::new(0)),
            tx,
        }
    }
}

impl EventHandler for Handler {}

impl ClientEventHandler for Handler {}

#[async_trait]
impl ServerEventHandler for Handler {
    async fn bot_create_command(&self, msg: CreateCommand) -> Result<CreateExecutor> {
        let options = AddOptions::new().path(&msg.path);

        let (tx, mut rx) = mpsc::channel::<(MessageContext, oneshot::Sender<robbot::Result>)>(32);
        let executor = Executor::new(tx);

        let executor_id = self.executor_id.fetch_add(1, Ordering::SeqCst);

        let client = self.tx.clone();
        task::spawn(async move {
            while let Some((data, tx)) = rx.recv().await {
                let msg = Message::new(MessageKind::Event(EventMessage::Message(MessageEvent {
                    ident: ExecutorIdent(executor_id),
                    data: data.event,
                })));

                let _ = client.send((msg, None)).await;

                let _ = tx.send(Ok(()));
            }
        });

        let mut command = robbot_core::command::Command::new(msg.name);
        command.set_description(msg.description);
        command.set_usage(msg.usage);
        command.set_example(msg.example);
        command.set_guild_only(msg.guild_only);
        command.set_permissions(msg.permissions);
        command.executor = Some(executor);

        if let Err(err) = self.state.commands().add_commands([command], options) {
            log::debug!("Failed to add command: {:?}", err);
            return Err(Error::Unknown);
        }

        Ok(CreateExecutor {
            ident: ExecutorIdent(executor_id),
        })
    }

    async fn bot_delete_command(&self, msg: DeleteCommand) -> Result<()> {
        let path = match msg.path.as_str() {
            "" => None,
            path => Some(path),
        };

        if let Err(err) = self.state.commands().remove_command(&msg.ident, path) {
            log::debug!(
                "[TCP] Failed to remove command {} {}: {:?}",
                msg.path,
                msg.ident,
                err
            );
            return Err(Error::Unknown);
        }

        Ok(())
    }

    async fn bot_get_command(&self, msg: GetCommand) -> Result<Command> {
        let args = parse_args(msg.path + &msg.ident);

        let cmd = self
            .state
            .commands()
            .get_command(&mut args.as_args())
            .ok_or(Error::Unknown)?;

        Ok(Command {
            name: cmd.name().to_owned(),
            description: cmd.description().to_owned(),
            usage: cmd.usage().to_owned(),
            example: cmd.example().to_owned(),
            guild_only: cmd.guild_only(),
            permissions: cmd.permissions().to_vec(),
            sub_commands: cmd
                .sub_commands()
                .into_iter()
                .map(|c| c.name().to_owned())
                .collect(),
        })
    }

    async fn bot_list_root_commands(&self) -> Result<Vec<String>> {
        let commands = self.state.commands().list_root_commands();

        Ok(commands)
    }

    async fn discord_send_message(&self, msg: SendMessage) -> Result<()> {
        let ctx = {
            let ctx = self.state.context.read().unwrap();

            match &*ctx {
                Some(ctx) => ctx.clone(),
                None => return Err(Error::Unknown),
            }
        };

        if let Err(err) = ctx.send_message(msg.channel_id, msg.data).await {
            log::debug!("[TCP] SendMessage failed: {:?}", err);
            return Err(Error::Unknown);
        }

        Ok(())
    }
}
