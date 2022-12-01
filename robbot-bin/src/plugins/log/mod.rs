//! A general purpose Discord server event logger.
//!
//! The `log` module does not do anything on its own, it is a facade for other modules to talk to.
//!
//! # Logging events
//!
//! `log` exports the [`log`] function which allows other modules to log [`LogEvent`]s. Note that
//! there is no guarantee that the event was ever logged, and there currently is no functionality
//! to check that.
//!
mod commands;
mod tasks;

use chrono::Utc;
use futures::Future;
use parking_lot::RwLock;
use robbot::builder::CreateMessage;
use robbot::model::id::{ChannelId, GuildId};
use robbot::store::get_one;
use robbot::util::color::Color;
use robbot::{module, Error, StoreData};
use robbot_core::context::Context;

static CONTEXT: RwLock<Option<Context<()>>> = RwLock::new(None);

const COLOR_ERROR: Color = Color::from_rgb(255, 0, 0);
const COLOR_WARN: Color = Color::from_rgb(252, 240, 20);
const COLOR_INFO: Color = Color::from_rgb(15, 86, 252);

module! {
    name: "log",
    cmds: {
        "log": {
            commands::set,
            commands::unset,
        },
    },
    tasks: [
        tasks::update_context,
    ],
    store: [
        LogChannel,
    ]
}

#[derive(Clone, Debug)]
pub struct LogEvent {
    pub level: LogLevel,
    pub guild_id: GuildId,
    pub target: Option<String>,
    pub content: String,
}

#[allow(unused)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
}

impl LogLevel {
    fn color(self) -> Color {
        match self {
            Self::Error => COLOR_ERROR,
            Self::Warn => COLOR_WARN,
            Self::Info => COLOR_INFO,
        }
    }
}

#[derive(Clone, Debug, StoreData)]
struct LogChannel {
    guild_id: GuildId,
    channel_id: ChannelId,
}

pub fn log(event: LogEvent) {
    tokio::task::spawn(async move {
        if let Err(err) = log_impl(event).await {
            log::error!("Failed to log event: {:?}", err);
        }
    });
}

fn log_impl(event: LogEvent) -> impl Future<Output = Result<(), Error>> {
    async move {
        let ctx = CONTEXT.read().clone();

        if let Some(ctx) = ctx {
            let channel = get_one!(ctx.state.store(), LogChannel => {
                guild_id == event.guild_id,
            })
            .await?;

            if let Some(channel) = channel {
                let footer = format!("*INFO* at {}", Utc::now());

                ctx.send_message(
                    channel.channel_id,
                    CreateMessage::new(|m| {
                        m.embed(|e| {
                            e.title("Event");
                            e.description(event.content);
                            e.color(event.level.color());
                            e.footer(|f| {
                                f.text(footer);
                            });
                        });
                    }),
                )
                .await?;
            }
        } else {
            log::trace!("Logging context not initialized, skipping event");
        }

        Ok(())
    }
}
