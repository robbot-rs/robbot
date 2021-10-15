pub mod bot;
mod builtin;
mod core;
mod help;
mod macros;
mod model;
mod plugins;

use crate::core::{
    command::CommandExecutor, hook::Event, router::parse_args, router::route_command, state::State,
};
use async_trait::async_trait;
use serenity::{
    client::{bridge::gateway::GatewayIntents, Client, Context, EventHandler},
    model::{
        channel::{GuildChannel, Message, Reaction},
        guild::Member,
        id::{ChannelId, GuildId, MessageId},
        user::User,
    },
};
use std::sync::Arc;

#[tokio::main]
async fn main() {
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

    // Load config.json file
    let config = load_config();

    // Create a store
    state.store.pool = Some(
        sqlx::MySqlPool::connect(&format!(
            "mysql://{}:{}@{}:{}/{}?ssl-mode=DISABLED",
            config.user, config.password, config.host, config.port, config.database
        ))
        .await
        .unwrap(),
    );

    let state = Arc::new(state);

    builtin::init(state.clone());

    #[cfg(feature = "debug")]
    plugins::debug::init(state.clone());

    plugins::guildsync::init(state.clone());
    plugins::temprole::init(state.clone());
    plugins::events::init(state.clone());

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
    async fn channel_create(&self, ctx: Context, channel: &GuildChannel) {
        self.state
            .hook_controller
            .send_event(Event::ChannelCreate(Box::new(bot::Context::new(
                ctx,
                self.state.clone(),
                channel.to_owned(),
            ))))
            .await;
    }

    async fn channel_delete(&self, ctx: Context, channel: &GuildChannel) {
        self.state
            .hook_controller
            .send_event(Event::ChannelDelete(Box::new(bot::Context::new(
                ctx,
                self.state.clone(),
                channel.to_owned(),
            ))))
            .await;
    }

    async fn guild_member_addition(&self, ctx: Context, guild_id: GuildId, new_member: Member) {
        self.state
            .hook_controller
            .send_event(Event::GuildMemberAddition(Box::new(bot::Context::new(
                ctx,
                self.state.clone(),
                (guild_id, new_member),
            ))))
            .await;
    }

    async fn guild_member_removal(
        &self,
        ctx: Context,
        guild_id: GuildId,
        user: User,
        member_data_if_available: Option<Member>,
    ) {
        self.state
            .hook_controller
            .send_event(Event::GuildMemberRemoval(Box::new(bot::Context::new(
                ctx,
                self.state.clone(),
                (guild_id, user, member_data_if_available),
            ))))
            .await;
    }

    async fn guild_member_update(
        &self,
        ctx: Context,
        old_if_available: Option<Member>,
        new: Member,
    ) {
        self.state
            .hook_controller
            .send_event(Event::GuildMemberUpdate(Box::new(bot::Context::new(
                ctx,
                self.state.clone(),
                (old_if_available, new),
            ))))
            .await;
    }

    async fn message(&self, ctx: Context, message: Message) {
        let message = model::Message::from(message);

        self.state
            .hook_controller
            .send_event(Event::Message(Box::new(bot::Context::new(
                ctx.clone(),
                self.state.clone(),
                message.clone(),
            ))))
            .await;

        let msg = match message.content.strip_prefix('!') {
            Some(msg) => msg,
            None => return,
        };

        #[cfg(feature = "permissions")]
        {
            // TODO: impl permission system
            let admins = vec![305107935606996992];
            if !admins.contains(&message.author.id.0) {
                return;
            }
        }

        let mut args = parse_args(msg);

        let cmd = {
            let commands = self.state.commands.read().unwrap();

            match route_command(&commands, &mut args) {
                Some(cmd) => cmd,
                None => return,
            }
        };

        let ctx = bot::Context {
            raw_ctx: ctx.clone(),
            state: self.state.clone(),
            args: args.iter().map(|s| s.to_string()).collect(),
            event: message.clone(),
        };

        // Return if the command is guild-only and the message is
        // not send from within a guild.
        if cmd.guild_only && message.guild_id.is_none() {
            let _ = ctx
                .respond(":x: This command can only be used in guilds.")
                .await;

            return;
        }

        match &cmd.executor {
            Some(executor) => {
                println!("executor");

                let res = match executor {
                    CommandExecutor::Message(executor) => executor.send(ctx.clone()).await,
                    CommandExecutor::GuildMessage(executor) => {
                        let ctx = bot::Context {
                            raw_ctx: ctx.raw_ctx.clone(),
                            state: ctx.state.clone(),
                            args: ctx.args.clone(),
                            event: model::GuildMessage::from(ctx.event.clone()),
                        };

                        executor.send(ctx).await
                    }
                };

                if let Err(err) = res {
                    match err {
                        // Display command help message.
                        bot::Error::InvalidCommandUsage => {
                            let _ = message
                                .channel_id
                                .send_message(&ctx.raw_ctx, |m| {
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
                            let _ = ctx.respond(":warning: Internal Server Error").await;
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

    async fn reaction_add(&self, ctx: Context, add_reaction: Reaction) {
        self.state
            .hook_controller
            .send_event(Event::ReactionAdd(Box::new(bot::Context::new(
                ctx,
                self.state.clone(),
                add_reaction,
            ))))
            .await;
    }

    async fn reaction_remove(&self, ctx: Context, removed_reaction: Reaction) {
        self.state
            .hook_controller
            .send_event(Event::ReactionRemove(Box::new(bot::Context::new(
                ctx,
                self.state.clone(),
                removed_reaction,
            ))))
            .await;
    }

    async fn reaction_remove_all(
        &self,
        ctx: Context,
        channel_id: ChannelId,
        removed_from_message_id: MessageId,
    ) {
        self.state
            .hook_controller
            .send_event(Event::ReactionRemoveAll(Box::new(bot::Context::new(
                ctx,
                self.state.clone(),
                (channel_id, removed_from_message_id),
            ))))
            .await;
    }

    async fn ready(&self, ctx: Context, _: serenity::model::gateway::Ready) {
        println!("[BOT] Bot online");

        {
            let mut connect_time = self.state.gateway_connect_time.write().unwrap();
            *connect_time = Some(std::time::Instant::now());
        }

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
    token: String,
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
