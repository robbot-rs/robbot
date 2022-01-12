use robbot_core::permissions::{RolePermission, UserPermission};

use robbot::arguments::ArgumentsExt;
use robbot::builder::CreateMessage;
use robbot::{command, Context, Error, Result, StoreData};
use robbot_core::context::MessageContext;

use serenity::model::id::{RoleId, UserId};
use std::fmt::Write;

pub(super) const COMMANDS: &[fn() -> robbot_core::command::Command] = &[add, list, remove];

#[command(
    description = "Add new permissions to a user or role.",
    usage = "<@User> <Permission...>",
    example = "@Robbbbbbb permissions.manage",
    guild_only = true
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
        let role_id: RoleId = id.parse().or(Err(Error::InvalidCommandUsage))?;

        for node in ctx.args.as_args() {
            let node = RolePermission {
                guild_id,
                role_id,
                node,
            };

            ctx.state.store().insert(node).await.unwrap();
        }
    } else {
        // Expect a user.
        let user_id: UserId = id.parse().or(Err(Error::InvalidCommandUsage))?;

        for node in ctx.args.as_args() {
            let node = UserPermission {
                guild_id,
                user_id,
                node,
            };

            ctx.state.store().insert(node).await.unwrap();
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
    guild_only = true
)]
async fn list(mut ctx: MessageContext) -> Result {
    let id = ctx.args.pop().ok_or(Error::InvalidCommandUsage)?;

    let guild_id = ctx.event.guild_id.unwrap();

    let mut description = String::new();

    if id.contains("@&") {
        // Expect a role.
        let role_id: RoleId = id.parse().or(Err(Error::InvalidCommandUsage))?;

        let nodes = ctx
            .state
            .permissions()
            .role_permissions(role_id, guild_id)
            .await
            .unwrap();

        for node in nodes {
            let _ = writeln!(description, "{}", node.node);
        }
    } else {
        // Expect a user.
        let user_id: UserId = id.parse().or(Err(Error::InvalidCommandUsage))?;

        let nodes = ctx
            .state
            .permissions()
            .user_permissions(user_id, guild_id)
            .await
            .unwrap();

        for node in nodes {
            let _ = writeln!(description, "{}", node.node);
        }
    }

    let _ = ctx
        .respond(CreateMessage::new(|m| {
            m.embed(|e| {
                e.title(format!("User Permissions"));
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
    guild_only = true
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
        let role_id: RoleId = id.parse().or(Err(Error::InvalidCommandUsage))?;

        for node in ctx.args.as_args() {
            let query = RolePermission::query()
                .guild_id(guild_id)
                .role_id(role_id)
                .node(node);

            ctx.state.store().delete(query).await.unwrap();
        }
    } else {
        // Expect a user.
        let user_id: UserId = id.parse().or(Err(Error::InvalidCommandUsage))?;

        for node in ctx.args.as_args() {
            let query = UserPermission::query()
                .guild_id(guild_id)
                .user_id(user_id)
                .node(node);

            ctx.state.store().delete(query).await.unwrap();
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
