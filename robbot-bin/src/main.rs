mod builtin;
mod config;
mod help;
mod logger;
mod macros;
mod model;
mod plugins;

/// Path of the default config.toml file.
const DEFAULT_CONFIG: &str = "./config.toml";

use async_trait::async_trait;
use clap::{App, Arg};
use robbot::{
    arguments::CommandArguments, executor::Executor as _, Command as _, Context as ContextExt,
    Error,
};
use robbot_core::{
    router::{find_command, parse_args},
    state::State,
};
use serenity::{
    client::{bridge::gateway::GatewayIntents, Client, Context, EventHandler},
    model::channel::Message,
};
use std::sync::{Arc, RwLock};

#[tokio::main]
async fn main() {
    let matches = App::new("robbot")
        .version("0.3.1")
        .author("")
        .about("")
        .arg(
            Arg::with_name("config")
                .short("c")
                .long("config")
                .value_name("FILE")
                .help("Provide a path to the config file")
                .takes_value(true),
        )
        .get_matches();

    let config = matches.value_of("config").unwrap_or(DEFAULT_CONFIG);

    // Load the config.toml file.
    let config = config::from_file(config);

    logger::init(&config);

    let gateway_intents = GatewayIntents::GUILDS
        | GatewayIntents::GUILD_MEMBERS
        | GatewayIntents::GUILD_BANS
        | GatewayIntents::GUILD_EMOJIS
        | GatewayIntents::GUILD_INVITES
        | GatewayIntents::GUILD_VOICE_STATES
        | GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::GUILD_MESSAGE_REACTIONS
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::DIRECT_MESSAGE_REACTIONS;

    let mut state = State::new();

    // Create a store
    state
        .store_mut()
        .connect(&config.database.connect_string())
        .await
        .unwrap();

    state.config = Arc::new(RwLock::new(config.clone()));

    let state = Arc::new(state);

    log::info!("Loading builtin commands");

    builtin::init(&state);

    #[cfg(feature = "debug")]
    plugins::debug::init(state.clone());

    log::info!("Connecting");

    let mut client = Client::builder(&config.token)
        .intents(gateway_intents)
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
        let message = robbot::model::Message::from(message);

        let msg = match message.content.strip_prefix('!') {
            Some(msg) => msg,
            None => return,
        };

        #[cfg(feature = "permissions")]
        {
            let config = self.state.config.read().unwrap();
            let admins = &config.admins;

            if !admins.contains(&message.author.id.0) {
                return;
            }
        }

        let mut args = CommandArguments::new(parse_args(msg));

        let cmd = {
            let commands = self.state.commands().get_inner();
            let commands = commands.read().unwrap();

            match find_command(&commands, &mut args) {
                Some(cmd) => cmd.clone(),
                None => return,
            }
        };

        let ctx = robbot_core::context::Context {
            raw_ctx: ctx.clone(),
            state: self.state.clone(),
            args,
            event: message.clone(),
        };

        // Return if the command is guild-only and the message is
        // not send from within a guild.
        if cmd.guild_only() && message.guild_id.is_none() {
            let _ = ctx
                .respond(":x: This command can only be used in guilds.")
                .await;

            return;
        }

        match cmd.executor() {
            Some(executor) => {
                let res = executor.send(ctx.clone()).await;

                if let Err(err) = res {
                    match err {
                        // Display command help message.
                        Error::InvalidCommandUsage => {
                            let _ = message
                                .channel_id
                                .send_message(&ctx.raw_ctx, |m| {
                                    m.embed(|e| {
                                        e.title(format!("Command Help: {}", cmd.name()));
                                        e.description(help::command(&cmd));
                                        e
                                    });
                                    m
                                })
                                .await;
                        }
                        _ => {
                            let _ = ctx.respond(":warning: Internal Server Error").await;
                            log::error!("Command error: {:?}", err);
                        }
                    }
                }
            }
            None => {
                help::command(&cmd);

                // Ignore error
                let _ = message
                    .channel_id
                    .send_message(&ctx.raw_ctx, |m| {
                        m.embed(|e| {
                            e.title(format!("Command Help: {}", cmd.name()));
                            e.description(help::command(&cmd));
                            e
                        });
                        m
                    })
                    .await;
            }
        }
    }

    async fn ready(&self, _ctx: Context, _ready: serenity::model::gateway::Ready) {
        log::info!("[BOT] Bot online");

        {
            let mut connect_time = self.state.connect_time.write().unwrap();
            *connect_time = Some(std::time::Instant::now());
        }
    }
}
