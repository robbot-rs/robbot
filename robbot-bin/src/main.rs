mod builtin;
mod config;
mod help;
mod logger;
mod macros;
mod model;
mod permissions;
mod plugins;

/// Path of the default config.toml file.
const DEFAULT_CONFIG: &str = "./config.toml";

use async_trait::async_trait;
use clap::{App, Arg};
use robbot::{
    arguments::CommandArguments, executor::Executor as _, Command as _, Context as ContextExt,
    Error,
};
use robbot_core::{router::parse_args, state::State};
use serenity::{
    client::{bridge::gateway::GatewayIntents, Client, Context, EventHandler},
    model::channel::Message,
};
use std::sync::{Arc, RwLock};

use serenity::model::guild::Member;
use serenity::model::id::GuildId;
use serenity::model::user::User;

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

        let msg = match message.content.strip_prefix('!') {
            Some(msg) => msg,
            None => return,
        };

        let mut args = parse_args(msg);
        let mut cmd_args = CommandArguments::new(args.clone());

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
            args: cmd_args,
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

        match cmd.executor() {
            Some(executor) => {
                let res = executor.send(ctx.clone()).await;

                if let Err(err) = res {
                    match err {
                        // Display command help message.
                        Error::InvalidCommandUsage => {
                            let _ = serenity::model::id::ChannelId(message.channel_id.0)
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
                            log::error!("Command '{}' returned an error: {:?}", args, err);
                        }
                    }
                }
            }
            None => {
                help::command(&cmd);

                // Ignore error
                let _ = serenity::model::id::ChannelId(message.channel_id.0)
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
