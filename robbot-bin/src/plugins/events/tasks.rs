use super::Event;

use chrono::{NaiveDateTime, TimeZone};
use robbot::model::id::GuildId;
use robbot::prelude::*;
use serenity::{model::id::ChannelId, utils::Color};

use robbot::store::get;
use robbot_core::context::{Context, TaskContext};

const EMBED_COLOR: Color = Color::from_rgb(0xFF, 0xA6, 0x00);

pub(super) async fn _announce(ctx: TaskContext) -> Result {
    // BDS
    const GID: GuildId = GuildId(583806437935808581);
    // FAIR
    // const GID: GuildId = GuildId(639101079035969536);

    let events = get!(ctx.state.store(), Event => {
        guild_id == GID,
    })
    .await?;

    let now = chrono::Utc::now();

    for mut event in events {
        let timezone: chrono_tz::Tz = event.timezone.parse().unwrap();
        // let event_time = timezone.timestamp(event.time as i64, 0);
        let event_time = timezone
            .from_local_datetime(&NaiveDateTime::from_timestamp(event.time as i64, 0))
            .unwrap();

        if now.timestamp() >= (event_time.timestamp() - 608400) as i64 {
            // // BDS
            announce_event(
                &ctx,
                ChannelId(776890081247494185),
                event.clone(),
                event_time.timestamp(),
            )
            .await?;

            // FAIR
            // announce_event(
            //     &ctx,
            //     ChannelId(764062446922498098),
            //     event.clone(),
            //     event_time.timestamp(),
            // )
            // .await?;

            ctx.state.store().delete(event.clone()).await?;

            match event.repeat_time {
                // Event is one time.
                0 => {}
                _ => {
                    event.time += event.repeat_time;

                    ctx.state.store().insert(event).await?;
                }
            }
        }
    }

    Ok(())
}

async fn announce_event<T>(
    ctx: &Context<T>,
    channel_id: ChannelId,
    event: Event,
    timestamp: i64,
) -> Result {
    let description = format!(
        "{}\n\n**When:** <t:{}:R>\n**Can you participate?**\n:white_check_mark:: Yes\n:x:: No",
        event.description, timestamp,
    );

    let message = channel_id
        .send_message(&ctx.raw_ctx, |m| {
            m.embed(|e| {
                e.title(format!("__{}__", event.title));
                e.color(EMBED_COLOR);
                e.description(description);
                e
            });
            m
        })
        .await?;

    for reaction in &['✅', '❌'] {
        message.react(&ctx.raw_ctx, *reaction).await?;
    }

    Ok(())
}
