use robbot::arguments::ChannelMention;
use robbot::prelude::ArgumentsExt;
use robbot::store::{delete, insert};
use robbot::{command, Error, Result};
use robbot_core::context::MessageContext;

use super::LogChannel;

#[command(
    description = "Setup a channel for logging.",
    usage = "<@Channel>",
    guild_only = true,
    permissions = ["admin"]
)]
async fn set(mut ctx: MessageContext) -> Result {
    let channel: ChannelMention = ctx.args.pop_parse()?;

    let guild_id = ctx.event.guild_id.unwrap();

    insert!(
        ctx.state.store(),
        LogChannel {
            channel_id: channel.id,
            guild_id
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
    guild_only = true,
    permissions = ["admin"],
)]
async fn unset(mut ctx: MessageContext) -> Result {
    let target = ctx.args.pop().ok_or(Error::InvalidCommandUsage)?;

    let guild_id = ctx.event.guild_id.unwrap();

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
