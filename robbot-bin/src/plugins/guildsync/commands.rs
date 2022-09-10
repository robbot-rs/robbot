use super::{gw2api, utils, GuildLink, GuildMember};
use super::{PERMISSION_MANAGE, PERMISSION_MANAGE_MEMBERS};

use robbot::arguments::ArgumentsExt;
use robbot::builder::CreateMessage;
use robbot::prelude::*;
use robbot::store::get;
use robbot_core::context::MessageContext;
use serenity::model::id::UserId;
use std::fmt::Write;
use tokio::join;

#[command(
    description = "Verify and link a user to a guild member.",
    usage = "<@User> <Account Name>",
    permissions = [PERMISSION_MANAGE_MEMBERS],
)]
async fn verify(mut ctx: MessageContext) -> Result {
    let user_id: UserId = ctx.args.pop_parse()?;
    let account_name: String = ctx.args.join_rest()?;

    let mut user = match ctx
        .event
        .guild_id
        .unwrap()
        .member(&ctx.raw_ctx, user_id)
        .await
    {
        Ok(user) => user,
        Err(_) => {
            let _ = ctx
                .respond(format!(":x: Failed to find <@{}> in server.", user_id))
                .await;
            return Ok(());
        }
    };

    let guild_link = get_guild_link(&ctx).await?;

    let ranks = guild_link.ranks(&ctx).await?;

    let guild_members =
        gw2api::GuildMember::get(&guild_link.gw_guild_id, &guild_link.api_token).await?;

    for guild_member in guild_members {
        if account_name == guild_member.name {
            let members = get!(ctx.state.store(), GuildMember => {
                link_id == guild_link.id,
                user_id == user_id,
            })
            .await?;

            log::debug!("{:?}", members);

            if !members.is_empty() {
                let _ = ctx
                    .respond(format!(
                        ":x: <@{}> is already linked with `{}`.",
                        user_id, members[0].account_name
                    ))
                    .await;
                return Ok(());
            }

            let member = GuildMember::new(guild_link.id, account_name, user_id);

            match join!(
                // Insert the user into the store.
                ctx.state.store().insert(member.clone()),
                // Update the user's roles.
                utils::update_user(&ctx, &mut user, Some(&guild_member.rank), &ranks)
            ) {
                // Store insertion failed.
                res if res.0.is_err() => {
                    let err = res.0.unwrap_err();
                    return Err(err.into());
                }
                // Updating roles failed.
                res if res.1.is_err() => {
                    let _ = ctx
                        .respond(format!(
                            ":warning: Failed to assign roles to <@{}>: `{:?}`",
                            member.user_id,
                            res.1.unwrap_err()
                        ))
                        .await;
                }
                _ => (),
            }

            let _ = ctx
                .respond(format!(
                    ":white_check_mark: Successfully linked <@{}> with `{}`.",
                    member.user_id, member.account_name
                ))
                .await?;
            return Ok(());
        }
    }

    let _ = ctx
        .respond(format!(
            ":x: Cannot find `{}` in Guild Rooster.",
            account_name
        ))
        .await;
    Ok(())
}

#[command(description = "Unverify and unlink a user or guild member.", permissions = [PERMISSION_MANAGE_MEMBERS])]
async fn unverify(mut ctx: MessageContext) -> Result {
    let user_id: UserId = ctx.args.pop_parse()?;

    let guild_link = get_guild_link(&ctx).await?;

    let members = guild_link.members(&ctx).await?;

    for member in members {
        if member.user_id == user_id {
            ctx.state
                .store()
                .delete(GuildMember::query().id(member.id))
                .await?;

            ctx.respond(format!(
                ":white_check_mark: Successfully unverified <@{}>.",
                user_id
            ))
            .await?;
            return Ok(());
        }
    }

    ctx.respond(format!(":x: <@{}> is not linked to any account.", user_id))
        .await?;

    Ok(())
}

#[command(description = "Identify a guild member by user.", permissions = [PERMISSION_MANAGE_MEMBERS])]
async fn whois(mut ctx: MessageContext) -> Result {
    let user_id: UserId = ctx.args.pop_parse()?;

    let guild_link = get_guild_link(&ctx).await?;

    let members = guild_link.members(&ctx).await?;

    for member in members {
        if member.user_id == user_id {
            ctx.respond(format!(
                ":white_check_mark: <@{}> is `{}`.",
                user_id, member.account_name
            ))
            .await?;
            return Ok(());
        }
    }

    ctx.respond(format!(
        ":white_check_mark: <@{}> is not linked to any account.",
        user_id
    ))
    .await?;

    Ok(())
}

#[command(description = "Resynchronize all links.", permissions = [PERMISSION_MANAGE])]
async fn sync(ctx: MessageContext) -> Result {
    let guildlink = get_guild_link(&ctx).await?;
    match utils::update_link(&ctx, guildlink).await {
        Ok(()) => {
            ctx.respond(":white_check_mark: Task successful.").await?;
        }
        Err(err) => {
            ctx.respond(":x: Task failed.").await?;
            log::error!("[Task] Manual task sync failed: {:?}", err);
        }
    }
    Ok(())
}

#[command(description = "List all guild members.", permissions = [PERMISSION_MANAGE_MEMBERS])]
async fn list(ctx: MessageContext) -> Result {
    let guild_link = get_guild_link(&ctx).await?;

    let members = guild_link.members(&ctx).await?;

    let mut description = String::new();
    for member in members {
        let _ = writeln!(
            description,
            "**{}**: <@{}>",
            member.account_name, member.user_id
        );
    }

    ctx.respond(CreateMessage::new(|m| {
        m.embed(|e| {
            e.title("__Linked Accounts__");
            e.description(description);
        });
    }))
    .await?;
    Ok(())
}

pub(super) async fn get_guild_link(ctx: &MessageContext) -> std::result::Result<GuildLink, Error> {
    let guild_links = get!(ctx.state.store(), GuildLink => {
        guild_id == ctx.event.guild_id.unwrap()
    })
    .await?;

    if guild_links.len() > 1 {
        unimplemented!();
    }

    Ok(guild_links.into_iter().next().unwrap())
}
