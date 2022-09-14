use super::PERMISSION_MANAGE;

use robbot::arguments::ArgumentsExt;
use robbot::arguments::{RoleMention, UserMention};
use robbot::builder::CreateMessage;
use robbot::store::{delete, insert};
use robbot::{command, Error, Result};
use robbot_core::context::MessageContext;
use robbot_core::permissions::{RolePermission, UserPermission};

use std::fmt::Write;

#[command(
    description = "Add new permissions to a user or role.",
    usage = "<@User> <Permission...>",
    example = "@Robbbbbbb permissions.manage",
    guild_only = true,
    permissions = [PERMISSION_MANAGE],
)]
async fn add(mut ctx: MessageContext) -> Result {
    let id = ctx.args.pop().ok_or(Error::InvalidCommandUsage)?;

    // Expect at least a single permission node.
    if ctx.args.is_empty() {
        return Err(Error::InvalidCommandUsage);
    }

    let guild_id = ctx.event.guild_id.unwrap();

    if id.contains("@&") {
        // Expect a role.
        let role: RoleMention = id.parse().or(Err(Error::InvalidCommandUsage))?;

        for node in ctx.args.as_args() {
            let node = RolePermission {
                guild_id,
                role_id: role.id,
                node,
            };

            insert!(ctx.state.store(), node).await?;
        }
    } else {
        // Expect a user.
        let user: UserMention = id.parse().or(Err(Error::InvalidCommandUsage))?;

        for node in ctx.args.as_args() {
            let node = UserPermission {
                guild_id,
                user_id: user.id,
                node,
            };

            insert!(ctx.state.store(), node).await?;
        }
    }

    let _ = ctx
        .respond(format!(
            ":white_check_mark: Added permissions `{}` to {}.",
            ctx.args.as_ref().join("`,`"),
            id
        ))
        .await;
    Ok(())
}

#[command(
    description = "List all permissions of a user or role.",
    usage = "<@User>",
    example = "@Robbbbbbb",
    guild_only = true,
    permissions = [PERMISSION_MANAGE],
)]
async fn list(mut ctx: MessageContext) -> Result {
    let id = ctx.args.pop().ok_or(Error::InvalidCommandUsage)?;

    let guild_id = ctx.event.guild_id.unwrap();

    let mut description = String::new();

    if id.contains("@&") {
        // Expect a role.
        let role: RoleMention = id.parse().or(Err(Error::InvalidCommandUsage))?;

        let nodes = ctx
            .state
            .permissions()
            .role_permissions(role.id, guild_id)
            .await?;

        for node in nodes {
            let _ = writeln!(description, "{}", node.node);
        }
    } else {
        // Expect a user.
        let user: UserMention = id.parse().or(Err(Error::InvalidCommandUsage))?;

        let nodes = ctx
            .state
            .permissions()
            .user_permissions(user.id, guild_id)
            .await?;

        for node in nodes {
            let _ = writeln!(description, "{}", node.node);
        }
    }

    let _ = ctx
        .respond(CreateMessage::new(|m| {
            m.embed(|e| {
                e.title("Permissions");
                e.description(description);
            });
        }))
        .await;
    Ok(())
}

#[command(
    description = "Remove permissions from a user.",
    usage = "<@User> <Permission...>",
    example = "@Robbbbbbb",
    guild_only = true,
    permissions = [PERMISSION_MANAGE],
)]
async fn remove(mut ctx: MessageContext) -> Result {
    let id = ctx.args.pop().ok_or(Error::InvalidCommandUsage)?;

    // Expect at least a single permission node.
    if ctx.args.is_empty() {
        return Err(Error::InvalidCommandUsage);
    }

    let guild_id = ctx.event.guild_id.unwrap();

    if id.contains("@&") {
        // Expect a role.
        let role: RoleMention = id.parse().or(Err(Error::InvalidCommandUsage))?;

        for node in ctx.args.as_args() {
            delete!(ctx.state.store(), RolePermission => {
                guild_id == guild_id,
                role_id == role.id,
                node == node,
            })
            .await?;
        }
    } else {
        // Expect a user.
        let user: UserMention = id.parse().or(Err(Error::InvalidCommandUsage))?;

        for node in ctx.args.as_args() {
            delete!(ctx.state.store(), UserPermission => {
                guild_id == guild_id,
                user_id == user.id,
                node == node,
            })
            .await?;
        }
    }

    let _ = ctx
        .respond(format!(
            ":white_check_mark: Removed permissions `{}` from {}.",
            ctx.args.as_ref().join("`,`"),
            id
        ))
        .await;
    Ok(())
}
