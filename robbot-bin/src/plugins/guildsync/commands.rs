use super::predicates::PREDIATE_CACHE;
use super::utils::{patch_member, ApiGuildMember, ApiGuildMembers};
use super::{gw2api, utils, GuildLink, GuildMember};
use super::{PERMISSION_MANAGE, PERMISSION_MANAGE_MEMBERS};

use ::gw2api::v2::guild::Guild;
use robbot::arguments::{ArgumentsExt, UserMention};
use robbot::builder::{CreateMessage, EditMessage};
use robbot::model::guild::Member;
use robbot::prelude::*;
use robbot::store::{delete, get};
use robbot_core::context::MessageContext;
use serenity::model::id::UserId;
use std::fmt::Write;
use std::time::Instant;
use tokio::join;

use ::gw2api::v2::account::Account;
use ::gw2api::Client;

#[command(
    name = "list",
    description = "List all configured links for this server."
)]
async fn setup_list(ctx: MessageContext) -> Result {
    // Fetch all `GuildLink`s for this guild.
    let links = get!(ctx.state.store(), GuildLink => {
        guild_id == ctx.event.guild_id.unwrap(),
    })
    .await?;

    let mut content = String::new();
    for link in links {
        let client: Client = Client::builder().access_token(&link.api_token).into();
        // TODO: This can be done in parallel with a join.
        let guild = Guild::get(&client, &link.gw_guild_id).await?;

        let api_token = link.api_token.get(0..12).unwrap_or("?");

        // String::write_fmt cannot fail.
        let _ = writeln!(
            content,
            "[{}]: **{}** => `{}...`",
            link.id, guild.name, api_token
        );
    }

    ctx.respond(CreateMessage::new(|m| {
        m.embed(|e| {
            e.title("__Linked Guilds__");
            e.description(content);
        });
    }))
    .await?;
    Ok(())
}

#[command(
    name = "create",
    description = "Create a new Guild for synchronisation.",
    usage = "<API_TOKEN> <GUILD_ID | GUILD_NAME>"
)]
async fn setup_create(mut ctx: MessageContext) -> Result {
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
            ctx.respond(":x: Token missing the `guilds` scope.").await?;
            return Ok(());
        }
    };

    let guild_id = if !guilds.contains(&guild) {
        let mut guilds = Guild::search(&client, &guild).await?;
        if guilds.len() == 0 {
            let _ = ctx
                .respond(format!(":x: Cannot find guild with name `{}`.", guild))
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

    let _ = ctx
        .respond(format!(
            ":white_check_mark: Successfully created new link using `{}`.",
            guild.name,
        ))
        .await?;
    Ok(())
}

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
            .send_message(
                ctx.event.channel_id,
                format!(
                    "{} This doesn't seem like a valid token format.",
                    ctx.event.author.id.mention()
                ),
            )
            .await;

        return Ok(());
    }

    let member = ctx
        .member(ctx.event.guild_id.unwrap(), ctx.event.author.id)
        .await?;

    let client: Client = Client::builder().access_token(token).into();

    // TODO: Handle other errors properly.
    match Account::get(&client).await {
        Ok(account) => {
            // Try to verify the user
            match verify_user(&ctx, member, account.name).await {
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

    let member = match ctx.member(guild_id, user_id.into()).await {
        Ok(user) => user,
        Err(_) => {
            let _ = ctx
                .respond(format!(":x: Failed to find <@{}> in server.", user_id))
                .await;
            return Ok(());
        }
    };

    let user_id = member.user.id;

    match verify_user(&ctx, member, account_name.clone()).await {
        Ok(()) => {
            ctx.respond(format!(
                ":white_check_mark: Successfully linked {} with `{}`.",
                UserMention::new(user_id),
                account_name
            ))
            .await?;
        }
        Err(VerifyError::AlreadyLinked) => {
            ctx.respond(format!(":x: `{}` is already linked.", account_name))
                .await?;
        }
        // ???
        Err(VerifyError::UserNotInGuild) | Err(VerifyError::AccountNotInGuild) => {
            ctx.respond(format!(":x: Cannot find `{}` in the guild.", account_name))
                .await?;
        }
        Err(VerifyError::Other(err)) => return Err(err.into()),
        Err(VerifyError::Sqlx(err)) => return Err(err.into()),
        Err(VerifyError::Req(err)) => return Err(err.into()),
    }

    Ok(())
}

#[command(description = "Unverify and unlink a user or guild member.", permissions = [PERMISSION_MANAGE_MEMBERS])]
async fn unverify(mut ctx: MessageContext) -> Result {
    Ok(())
}

#[command(description = "Identify a guild member by user.", permissions = [PERMISSION_MANAGE_MEMBERS])]
async fn whois(mut ctx: MessageContext) -> Result {
    Ok(())
}

#[command(description = "Resynchronize all links.", permissions = [PERMISSION_MANAGE])]
async fn sync(ctx: MessageContext) -> Result {
    let msg = ctx
        .respond(":information_source: Started synchronisation process...")
        .await?;

    let now = Instant::now();
    utils::update_links(&ctx, ctx.event.guild_id.unwrap()).await?;

    ctx.edit_message(
        msg.channel_id,
        msg.id,
        EditMessage::new(|m| {
            m.content(format!(
                ":white_check_mark: Synchronisation proccess complete in {}s.",
                now.elapsed().as_secs()
            ));
        }),
    )
    .await?;

    Ok(())
}

#[command(description = "List all guild members.", permissions = [PERMISSION_MANAGE_MEMBERS])]
async fn list(mut ctx: MessageContext) -> Result {
    let link = match GuildLink::extract(&mut ctx).await? {
        Some(link) => link,
        None => {
            ctx.respond(":x: Cannot find matching link.").await?;
            return Ok(());
        }
    };

    let members = link.members(&ctx).await?;

    let mut description = String::new();
    for member in members {
        let _ = writeln!(
            description,
            "**{}**: <@{}>",
            member.account_name, member.user_id
        );
    }

    let guild = ::gw2api::v2::guild::Guild::get(&Client::new(), &link.gw_guild_id).await?;

    ctx.respond(CreateMessage::new(|m| {
        m.embed(|e| {
            e.title(format!("__{}__", guild.name));
            e.description(description);
        });
    }))
    .await?;
    Ok(())
}

async fn verify_user(
    ctx: &MessageContext,
    member: Member,
    account_name: String,
) -> std::result::Result<(), VerifyError> {
    let guild_id = member.guild_id;
    let user_id = member.user.id;
    // let guild_id = ctx.event.guild_id.unwrap();

    // let mut user = match ctx.member(guild_id, user_id.into()).await {
    //     Ok(user) => user,
    //     Err(_) => return Err(VerifyError::UserNotInGuild),
    // };

    // let guild_members =
    //     gw2api::GuildMember::get(&guild_link.gw_guild_id, &guild_link.api_token).await?;

    // for guild_member in guild_members {
    //     if account_name == guild_member.name {
    //         let members = get!(ctx.state.store(), GuildMember => {
    //             link_id == guild_link.id,
    //             user_id == user_id.into(),
    //         })
    //         .await?;

    //         if !members.is_empty() {
    //             return Err(VerifyError::AlreadyLinked);
    //         }

    //         ctx.state.store().insert(member.clone()).await?;

    //         return Ok(());
    //     }
    // }

    // Err(VerifyError::AccountNotInGuild)

    let predicates = match PREDIATE_CACHE.lock().get(member.guild_id) {
        Some(preds) => preds,
        // If PREDIATE_CACHE is not initialized, there is currently a sync in progress.
        None => return Ok(()),
    };

    let links = get!(ctx.state.store(), GuildLink => {
        guild_id == member.guild_id,
    })
    .await?;

    // We only filter for member.
    let mut api_members = ApiGuildMembers::new();
    for link in &links {
        let client: Client = Client::builder().access_token(&link.api_token).into();
        let members = ::gw2api::v2::guild::GuildMembers::get(&client, &link.gw_guild_id).await?;

        for m in members.0 {
            if m.name == account_name {
                api_members.push(ApiGuildMember::new(link, member.user.id, m));

                ctx.state
                    .store()
                    .insert(GuildMember::new(
                        link.id,
                        account_name.to_owned(),
                        member.user.id,
                    ))
                    .await?;
                continue;
            }
        }
    }

    if api_members.is_empty() {
        return Err(VerifyError::UserNotInGuild);
    }

    let edit = patch_member(&predicates, member, &api_members);
    ctx.edit_member(guild_id, user_id, edit)
        .await
        .map_err(|err| Error::from(err))?;
    Ok(())
}

enum VerifyError {
    UserNotInGuild,
    /// The user is already linked.
    AlreadyLinked,
    /// The account was not found in the guild.
    AccountNotInGuild,
    Other(Error),
    Sqlx(sqlx::Error),
    Req(::gw2api::Error),
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

impl From<::gw2api::Error> for VerifyError {
    fn from(err: ::gw2api::Error) -> Self {
        Self::Req(err)
    }
}
