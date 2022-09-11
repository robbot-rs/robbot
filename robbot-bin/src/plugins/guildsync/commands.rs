use super::{gw2api, utils, GuildLink, GuildMember};
use super::{PERMISSION_MANAGE, PERMISSION_MANAGE_MEMBERS};

use robbot::arguments::{ArgumentsExt, UserMention};
use robbot::builder::CreateMessage;
use robbot::prelude::*;
use robbot::store::{delete, get};
use robbot_core::context::MessageContext;
use serenity::model::id::UserId;
use std::fmt::Write;
use tokio::join;

use ::gw2api::v2::account::Account;
use ::gw2api::Client;

#[command(
    name = "verify",
    description = "Verify using an API token.",
    usage = "<TOKEN>",
    example = "564F181A-F0FC-114A-A55D-3C1DCD45F3767AF3848F-AB29-4EBF-9594-F91E6A75E015"
)]
async fn verify_api(mut ctx: MessageContext) -> Result {
    ctx.raw_ctx
        .http
        .delete_message(ctx.event.channel_id.0, ctx.event.id.0)
        .await?;

    let token: String = ctx.args.pop_parse()?;

    // Api key are always exactly 72 ascii chars in length.
    if token.len() != 72 {
        let _ = ctx
            .respond("This doesn't seem like a valid token format.")
            .await;
        return Ok(());
    }

    let client: Client = Client::builder().access_token(token).into();

    // TODO: Handle other errors properly.
    match Account::get(&client).await {
        Ok(account) => {
            // Try to verify the user
            match verify_user(&ctx, ctx.event.author.id.into(), account.name).await {
                Ok(()) => {
                    let _ = ctx.respond("").await;

                    let _ = ctx
                        .send_message(
                            ctx.event.channel_id,
                            format!(
                                "{} :white_check_mark: Verification successful.",
                                UserMention::new(ctx.event.author.id)
                            ),
                        )
                        .await;

                    return Ok(());
                }
                // If the user that used this command is not in the server anymore he left
                // immediately.
                Err(VerifyError::UserNotInGuild) => return Ok(()),
                Err(VerifyError::AlreadyLinked) => {
                    let _ = ctx
                        .send_message(
                            ctx.event.channel_id,
                            format!(
                            "{} :x: Seems like there already is a user linked with your account.",
                            UserMention::new(ctx.event.author.id)
                        ),
                        )
                        .await;

                    return Ok(());
                }
                Err(VerifyError::AccountNotInGuild) => {
                    let _ = ctx.send_message(
                        ctx.event.channel_id,
                        format!(
                            "{} :x: Cannot find you in the guild. Note that it might take up to an hour for the API to update if you just joined the guild.",
                            UserMention::new(ctx.event.author.id)
                        )
                    ).await;

                    return Ok(());
                }
                Err(VerifyError::Other(err)) => return Err(err.into()),
                Err(VerifyError::Sqlx(err)) => return Err(err.into()),
                Err(VerifyError::Req(err)) => return Err(err.into()),
            }
        }
        Err(err) => {
            log::warn!("Failed to fetch user account: {}", err);

            let _ = ctx
                .respond(":x: Failed to fetch your account. Is your token valid?")
                .await;
        }
    }

    Ok(())
}

#[command(
    description = "Verify and link a user to a guild member.",
    usage = "<@User> <Account Name>",
    permissions = [PERMISSION_MANAGE_MEMBERS],
)]
async fn verify(mut ctx: MessageContext) -> Result {
    let user_id: UserId = ctx.args.pop_parse()?;
    let account_name: String = ctx.args.join_rest()?;

    let guild_id = ctx.event.guild_id.unwrap();

    let mut user = match ctx.get_member(guild_id, user_id.into()).await {
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
                user_id == user_id.into(),
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

            let member = GuildMember::new(guild_link.id, account_name, user_id.into());

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
        if member.user_id == user_id.into() {
            delete!(ctx.state.store(), GuildMember => {
                id == member.id,
            })
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
        if member.user_id == user_id.into() {
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

async fn verify_user(
    ctx: &MessageContext,
    user_id: UserId,
    account_name: String,
) -> std::result::Result<(), VerifyError> {
    let guild_link = get_guild_link(ctx).await?;

    let guild_id = ctx.event.guild_id.unwrap();

    let ranks = guild_link.ranks(ctx).await?;

    let mut user = match ctx.get_member(guild_id, user_id.into()).await {
        Ok(user) => user,
        Err(_) => return Err(VerifyError::UserNotInGuild),
    };

    let guild_members =
        gw2api::GuildMember::get(&guild_link.gw_guild_id, &guild_link.api_token).await?;

    for guild_member in guild_members {
        if account_name == guild_member.name {
            let members = get!(ctx.state.store(), GuildMember => {
                link_id == guild_link.id,
                user_id == user_id.into(),
            })
            .await?;

            if !members.is_empty() {
                return Err(VerifyError::AlreadyLinked);
            }

            let member = GuildMember::new(guild_link.id, account_name, user_id.into());

            ctx.state.store().insert(member.clone()).await?;
            utils::update_user(ctx, &mut user, Some(&guild_member.rank), &ranks).await?;

            return Ok(());
        }
    }

    Err(VerifyError::AccountNotInGuild)
}

enum VerifyError {
    UserNotInGuild,
    /// The user is already linked.
    AlreadyLinked,
    /// The account was not found in the guild.
    AccountNotInGuild,
    Other(Error),
    Sqlx(sqlx::Error),
    Req(reqwest::Error),
}

impl From<Error> for VerifyError {
    fn from(err: Error) -> Self {
        Self::Other(err)
    }
}

impl From<sqlx::Error> for VerifyError {
    fn from(err: sqlx::Error) -> Self {
        Self::Sqlx(err)
    }
}

impl From<reqwest::Error> for VerifyError {
    fn from(err: reqwest::Error) -> Self {
        Self::Req(err)
    }
}
