use std::fmt::Write;

use gw2api::v2::account::Account;
use gw2api::v2::guild::Guild;
use gw2api::Client;
use robbot::builder::CreateMessage;
use robbot::prelude::ArgumentsExt;
use robbot::store::{delete, get};
use robbot::{command, Error, Result};
use robbot_core::context::MessageContext;

use super::PERMISSION_MANAGE;
use crate::plugins::guildsync::{GuildLink, GuildMember, GuildRank};

#[command(
    description = "Create a new link for synchronisation.",
    usage = "<API_TOKEN> <GUILD_ID | GUILD_NAME >",
    permissions = [PERMISSION_MANAGE]
)]
pub async fn create(mut ctx: MessageContext) -> Result {
    let token: String = ctx.args.pop_parse()?;
    let guild: String = ctx.args.join_rest()?;

    if guild.is_empty() {
        return Err(Error::InvalidCommandUsage);
    }

    let client: Client = Client::builder().access_token(&token).into();
    let account = Account::get(&client).await?;

    let guilds = match account.guild_leader {
        Some(guilds) => guilds,
        None => {
            ctx.respond(":x: API token missing `guilds` scope.").await?;
            return Ok(());
        }
    };

    let guild_id = if !guilds.contains(&guild) {
        // Do a guild lookup with `guild` name.
        let mut guilds = Guild::search(&client, &guild).await?;
        if guilds.len() == 0 {
            ctx.respond(format!(":x: Cannot find guild with name `{}`.", guild))
                .await?;
            return Ok(());
        }

        guilds.remove(0)
    } else {
        guild
    };

    let guild = Guild::get(&client, &guild_id).await?;

    // Insert the new link.
    let link = GuildLink::new(ctx.event.guild_id.unwrap(), guild.id, token);
    ctx.state.store().insert(link).await?;

    ctx.respond(format!(
        ":white_check_mark: Successfully created new link using `{}`.",
        guild.name
    ))
    .await?;

    Ok(())
}

#[command(
    description = "Delete an existing link.",
    usage = "<LINK_ID | GUILD_ID | GUILD_NAME>",
    permissions = [PERMISSION_MANAGE]
)]
pub async fn delete(mut ctx: MessageContext) -> Result {
    let link = match GuildLink::extract_exact(&mut ctx).await? {
        Some(link) => link,
        None => {
            ctx.respond(":x: Cannot find link with id, guild id or guild name.")
                .await?;
            return Ok(());
        }
    };

    delete!(ctx.state.store(), GuildRank => {
        link_id == link.id
    })
    .await?;

    delete!(ctx.state.store(), GuildMember => {
        link_id == link.id
    })
    .await?;

    delete!(ctx.state.store(), GuildLink => {
        id == link.id
    })
    .await?;

    Ok(())
}

#[command(description = "List all existing links.", usage = "", example = "", permissions = [PERMISSION_MANAGE])]
pub async fn list(ctx: MessageContext) -> Result {
    let links = get!(ctx.state.store(), GuildLink => {
        guild_id == ctx.event.guild_id.unwrap()
    })
    .await?;

    let mut description = String::new();
    for link in links {
        let client: Client = Client::builder().access_token(&link.api_token).into();
        let guild = Guild::get(&client, &link.gw_guild_id).await?;

        let api_token = link.api_token.get(0..12).unwrap_or("?");

        let _ = writeln!(
            description,
            "[{}]: **{}** => `{}...`",
            link.id, guild.name, api_token
        );
    }

    ctx.respond(CreateMessage::new(|m| {
        m.embed(|e| {
            e.title("__Linked Guilds__");
            e.description(description);
        });
    }))
    .await?;
    Ok(())
}

#[command(
    description = "Show details about a link.",
    usage = "<LINK_ID | GUILD_ID | GUILD_NAME>",
    example = "1",
    permissions = [PERMISSION_MANAGE]
)]
pub async fn details(mut ctx: MessageContext) -> Result {
    let link = match GuildLink::extract(&mut ctx).await? {
        Some(link) => link,
        None => {
            ctx.respond(":x: Cannot find matching link").await?;
            return Ok(());
        }
    };

    let ranks = link.ranks(&ctx).await?;

    let client: Client = Client::builder().access_token(&link.api_token).into();
    let guild = Guild::get(&client, &link.gw_guild_id).await?;

    let mut description = String::new();
    for rank in ranks {
        let _ = writeln!(
            description,
            "*{}* => {}",
            rank.rank_name,
            rank.role_id.mention()
        );
    }

    ctx.respond(CreateMessage::new(|m| {
        m.embed(|e| {
            e.title(format!("__{}__", guild.name));
            e.description(description);
        });
    }))
    .await?;
    Ok(())
}
