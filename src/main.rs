pub mod bot;
mod builtin;
mod core;
mod help;
mod macros;
mod plugins;

use crate::core::{router::parse_args, router::route_command, state::State};

use async_trait::async_trait;
use std::sync::Arc;

use serenity::{
    client::{Client, Context, EventHandler},
    model::channel::Message,
};

const TOKEN: &str = "NjAzOTk0NTU0MjE5MTAyMjIw.XTnfww.Kyk3cwqsIc2hUNuzGiAV5SLjnCQ";

#[tokio::main]
async fn main() {
    let mut state = State::new();

    // Load config.json file
    let config = load_config();

    // Create a store
    state.store.pool = Some(
        sqlx::MySqlPool::connect(&format!(
            "mysql://{}:{}@{}/{}?ssl-mode=DISABLED",
            config.user, config.password, config.host, config.database
        ))
        .await
        .unwrap(),
    );

    let state = Arc::new(state);

    #[cfg(debug_assertions)]
    plugins::debug::init(state.clone());

    // builtin::init(state.clone());
    // plugins::guildsync::init(state.clone());
    // plugins::temprole::init(state.clone());
    // plugins::events::init(state.clone());

    let mut client = Client::builder(&TOKEN)
        .event_handler(Handler { state })
        .await
        .unwrap();

    client.start().await.unwrap();
}

pub struct Handler {
    state: Arc<State>,
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, message: Message) {
        let msg = match message.content.strip_prefix("!") {
            Some(msg) => msg,
            None => return,
        };

        // TODO: impl permission system
        let admins = vec![];
        if !admins.contains(&message.author.id.0) {
            return;
        }

        let mut args = parse_args(&msg);

        let cmd = {
            let commands = self.state.commands.read().unwrap();

            match route_command(&commands, &mut args) {
                Some(cmd) => cmd,
                None => return,
            }
        };

        match cmd.executor() {
            Some(executor) => {
                // Convert args to owned strings.
                let args = args.iter().map(|s| s.to_string()).collect();

                let channel_id = message.channel_id;

                let res = executor
                    .send(bot::Context {
                        raw_ctx: ctx.clone(),
                        state: self.state.clone(),
                        args,
                        event: message,
                    })
                    .await;

                if let Err(err) = res {
                    match err {
                        // Display command help message.
                        bot::Error::InvalidCommandUsage => {
                            let _ = channel_id
                                .send_message(&ctx, |m| {
                                    m.embed(|e| {
                                        e.title(format!("Command Help: {}", cmd.name));
                                        e.description(help::command(&cmd));
                                        e
                                    });
                                    m
                                })
                                .await;
                        }
                        _ => {
                            let _ = channel_id
                                .send_message(&ctx, |m| {
                                    m.content(format!(
                                        ":warning: Internal Server Error:\n`{:?}`",
                                        err
                                    ));
                                    m
                                })
                                .await;
                        }
                    }
                }
            }
            None => {
                help::command(&cmd);

                // Ignore error
                let _ = message
                    .channel_id
                    .send_message(&ctx, |m| {
                        m.embed(|e| {
                            e.title(format!("Command Help: {}", cmd.name));
                            e.description(help::command(&cmd));
                            e
                        });
                        m
                    })
                    .await;
            }
        }
    }

    async fn ready(&self, ctx: Context, _: serenity::model::gateway::Ready) {
        self.state
            .task_scheduler
            .update_context(Some(bot::Context {
                raw_ctx: ctx,
                state: self.state.clone(),
                args: Vec::new(),
                event: (),
            }))
            .await;
    }
}

#[derive(serde::Deserialize)]
struct Config {
    host: String,
    port: u16,
    user: String,
    password: String,
    database: String,
}

fn load_config() -> Config {
    let file = std::fs::File::open("config.json").unwrap();
    serde_json::from_reader(&file).unwrap()
}
