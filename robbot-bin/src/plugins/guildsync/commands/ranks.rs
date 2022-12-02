use std::fmt::Write;

use robbot::arguments::RoleMention;
use robbot::builder::CreateMessage;
use robbot::prelude::ArgumentsExt;
use robbot::store::delete;
use robbot::{command, Result};
use robbot_core::context::MessageContext;

use super::super::PERMISSION_MANAGE;
use crate::plugins::guildsync::{GuildLink, GuildRank};

#[command(
        description = "List the currently rank to role mappings for a link.",
        usage = "<LINK_ID | GUILD_ID | GUILD_NAME>",
        example = "1",
        permissions = [PERMISSION_MANAGE]
    )]
pub async fn list(mut ctx: MessageContext) -> Result {
    let link = match GuildLink::extract(&mut ctx).await? {
        Some(link) => link,
        None => {
            ctx.respond(":x: Cannot find matching link.").await?;
            return Ok(());
        }
    };

    let mut description = String::new();
    for rank in link.ranks(&ctx).await? {
        let _ = writeln!(
            description,
            "{} => `{}`",
            rank.role_id.mention(),
            rank.rank_name
        );
    }

    ctx.respond(CreateMessage::new(|m| {
        m.embed(|e| {
            e.title("__Rank Mappings__");
            e.description(description);
        });
    }))
    .await?;

    Ok(())
}

#[command(
        description = "Set/Overwrite a rank to role mapping.",
        usage = "<LINK_ID | GUILD_ID | GUILD_NAME> <RANK_NAME> <ROLE>",
        example = "1 Member <@&12345>",
        permissions = [PERMISSION_MANAGE]
    )]
pub async fn set(mut ctx: MessageContext) -> Result {
    let link = match GuildLink::extract(&mut ctx).await? {
        Some(link) => link,
        None => {
            ctx.respond(":x: Cannot find matching link.").await?;
            return Ok(());
        }
    };

    let rank_name: String = ctx.args.pop_parse()?;
    let role: RoleMention = ctx.args.pop_parse()?;

    // Remove existing rank mappings.
    for rank in link.ranks(&ctx).await? {
        if rank.rank_name == rank_name {
            delete!(ctx.state.store(), GuildRank => {
                id == rank.id,
            })
            .await?;
        }
    }

    let rank = GuildRank::new(link.id, rank_name.clone(), role.id);
    ctx.state.store().insert(rank).await?;

    ctx.respond(format!(
        ":white_check_mark: Successfully mapped role {} to rank `{}`.",
        role, rank_name
    ))
    .await?;
    Ok(())
}

#[command(
        description = "Clear a rank to role mapping.",
        usage = "<LINK_ID | GUILD_ID | GUILD_NAME> <RANK_NAME>",
        example = "1 Member",
        permissions = [PERMISSION_MANAGE]
    )]
pub async fn unset(mut ctx: MessageContext) -> Result {
    let link = match GuildLink::extract(&mut ctx).await? {
        Some(link) => link,
        None => {
            ctx.respond(":x: Cannot find matching link.").await?;
            return Ok(());
        }
    };

    let rank_name: String = ctx.args.pop_parse()?;

    delete!(ctx.state.store(), GuildRank => {
        link_id == link.id,
        rank_name == rank_name.clone(),
    })
    .await?;

    ctx.respond(format!(
        ":white_check_mark: Successfully unmapped `{}`.",
        rank_name
    ))
    .await?;
    Ok(())
}
