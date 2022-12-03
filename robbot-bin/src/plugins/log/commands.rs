use robbot::arguments::ChannelMention;
use robbot::prelude::ArgumentsExt;
use robbot::store::{delete, insert};
use robbot::{command, Error, Result};
use robbot_core::context::GuildMessageContext;

use super::LogChannel;

#[command(
    description = "Setup a channel for logging.",
    usage = "<@Channel>",
    permissions = ["admin"]
)]
async fn set(mut ctx: GuildMessageContext) -> Result {
    let channel: ChannelMention = ctx.args.pop_parse()?;

    insert!(
        ctx.state.store(),
        LogChannel {
            channel_id: channel.id,
            guild_id: ctx.event.guild_id
        }
    )
    .await?;

    ctx.respond(format!(
        ":white_check_mark: Configured {} for logging.",
        channel
    ))
    .await?;
    Ok(())
}

#[command(
    description = "Unset a logging channel.",
    usage = "<@Channel> | all",
    permissions = ["admin"],
)]
async fn unset(mut ctx: GuildMessageContext) -> Result {
    let target = ctx.args.pop().ok_or(Error::InvalidCommandUsage)?;

    let guild_id = ctx.event.guild_id;

    // Remove all logging channels.
    if target == "all" {
        delete!(ctx.state.store(), LogChannel => {
            guild_id == guild_id,
        })
        .await?;

        ctx.respond(":white_check_mark: Unset all logging channels.")
            .await?;
        return Ok(());
    }

    // Remove a single logging channel.
    let channel: ChannelMention = target.parse().or(Err(Error::InvalidCommandUsage))?;

    delete!(ctx.state.store(), LogChannel => {
        guild_id == guild_id,
        channel_id == channel.id,
    })
    .await?;

    ctx.respond(format!(
        ":white_check_mark: Unset logging channel: {}",
        channel
    ))
    .await?;
    Ok(())
}
