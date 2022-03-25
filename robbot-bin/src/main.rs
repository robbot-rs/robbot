mod builtin;
mod config;
mod help;
mod logger;
mod macros;
mod model;
mod permissions;
mod plugins;
mod signal;

/// Path of the default config.toml file.
const DEFAULT_CONFIG: &str = "./config.toml";

use async_trait::async_trait;
use clap::Parser;
use robbot::builder::CreateMessage;
use robbot::{
    arguments::CommandArguments, executor::Executor as _, Command as _, Context as ContextExt,
    Error,
};
use robbot_core::{router::parse_args, state::State};
use serenity::{
    client::{bridge::gateway::GatewayIntents, Client, Context, EventHandler},
    model::channel::Message,
};
use std::sync::Arc;

use tokio::task;

use serenity::model::guild::Member;
use serenity::model::id::GuildId;
use serenity::model::user::User;

#[derive(Clone, Debug, Parser)]
#[clap(version, long_about = None)]
struct Args {
    #[clap(short, long, value_name = "FILE", default_value_t = String::from(DEFAULT_CONFIG))]
    config: String,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    // Initialize the logger as early as possible.
    logger::init();

    // Load the config.toml file.
    let config = match config::from_file(&args.config) {
        Ok(config) => config,
        Err(err) => {
            log::error!("Failed to read config file {}: {:?}", args.config, err);
            std::process::exit(1);
        }
    };

    // Check the config file.
    // Empty prefix strings are not allowed.
    if config.prefix.is_empty() {
        log::error!("Invalid config file: prefix must be a non-empty string");
        std::process::exit(1);
    }

    signal::init();

    logger::set_log_level(&config);

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

    let state = Arc::new(State::new(config));

    log::info!("[CORE] Loading builtin commands");

    // Load all builtin functions.
    if let Err(err) = builtin::init(&state) {
        log::error!("[CORE] Failed to load builtin functions: {:?}", err);
        log::error!("[CORE] Fatal error, exiting");
        std::process::exit(1);
    }

    if let Err(err) = plugins::init(state.clone()).await {
        log::error!("[CORE] Failed to load plugin: {:?}", err);
    }

    log::info!("[BOT] Connecting");

    let mut client = Client::builder(&state.config.token)
        .intents(gateway_intents)
        .event_handler(Handler { state })
        .await
        .unwrap();

    let shard_manager = client.shard_manager.clone();

    task::spawn(async move {
        signal::subscribe().await;
        log::info!("[CORE] Received shutdown");

        shard_manager.lock().await.shutdown_all().await;
    });

    client.start().await.unwrap();
}

pub struct Handler {
    state: Arc<State>,
}

#[async_trait]
impl EventHandler for Handler {
    async fn guild_member_addition(&self, _ctx: Context, guild_id: GuildId, member: Member) {
        let event = robbot::hook::GuildMemberAdditionData { guild_id, member };

        self.state.hooks().dispatch_event(event).await;
    }

    async fn guild_member_removal(
        &self,
        _ctx: Context,
        guild_id: GuildId,
        user: User,
        member: Option<Member>,
    ) {
        let event = robbot::hook::GuildMemberRemovalData {
            guild_id,
            user,
            member,
        };

        self.state.hooks().dispatch_event(event).await;
    }

    async fn guild_member_update(&self, _ctx: Context, old_member: Option<Member>, member: Member) {
        let event = robbot::hook::GuildMemberUpdateData { old_member, member };

        self.state.hooks().dispatch_event(event).await;
    }

    async fn message(&self, raw_ctx: Context, message: Message) {
        let message = robbot::model::channel::Message::from(message);

        {
            let event = robbot::hook::MessageData(message.clone());

            self.state.hooks().dispatch_event(event).await;
        }

        let msg = match message.content.strip_prefix(&self.state.config.prefix) {
            Some(msg) => msg,
            None => return,
        };

        let mut args = parse_args(msg);
        let mut cmd_args = CommandArguments::from(args.clone());

        let cmd = match self.state.commands().get_command(&mut cmd_args) {
            Some(cmd) => cmd,
            None => return,
        };

        // Only retain the base path of the called command.
        for _ in 0..cmd_args.as_args().len() {
            args.remove(args.len() - 1);
        }

        let ctx = robbot_core::context::Context {
            raw_ctx: raw_ctx.clone(),
            state: self.state.clone(),
            args: cmd_args.clone(),
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

        #[cfg(feature = "permissions")]
        {
            if !permissions::has_permission(&ctx, cmd.permissions())
                .await
                .unwrap()
            {
                let _ = ctx
                    .respond(":no_entry_sign: You are not allowed to run this command.")
                    .await;

                return;
            }
        }

        let path = cmd_args.as_parsed_args().join(" ");

        match cmd.executor() {
            Some(executor) => {
                let res = executor.send(ctx.clone()).await;

                if let Err(err) = res {
                    match err {
                        // Display command help message.
                        Error::InvalidCommandUsage => {
                            let _ = ctx
                                .respond(CreateMessage::new(|m| {
                                    m.embed(|e| {
                                        e.title(format!("Command Help: {}", path));
                                        e.color(builtin::EMBED_COLOR);
                                        e.description(help::command(
                                            &cmd,
                                            &path,
                                            &self.state.config.prefix,
                                        ));
                                    });
                                }))
                                .await;
                        }
                        _ => {
                            let _ = ctx.respond(":warning: Internal Server Error").await;
                            log::error!("Command '{}' returned an error: {:?}", args, err);
                        }
                    }
                }
            }
            None => {
                help::command(&cmd, &path, &self.state.config.prefix);

                // Ignore error
                let _ = ctx
                    .respond(CreateMessage::new(|m| {
                        m.embed(|e| {
                            e.title(format!("Command Help: {}", path));
                            e.color(builtin::EMBED_COLOR);
                            e.description(help::command(&cmd, &path, &self.state.config.prefix));
                        });
                    }))
                    .await;
            }
        }
    }

    async fn ready(&self, ctx: Context, _ready: serenity::model::gateway::Ready) {
        log::info!("[BOT] Bot online");

        let ctx = robbot_core::context::Context::new(ctx, self.state.clone(), ());

        {
            let mut connect_time = self.state.connect_time.write().unwrap();
            *connect_time = Some(std::time::Instant::now());
        }
        {
            let mut context = self.state.context.write().unwrap();
            *context = Some(ctx.clone());
        }

        self.state.tasks().update_context(Some(ctx)).await;
    }
}
