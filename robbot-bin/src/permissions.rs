use robbot::Error;
use robbot_core::context::MessageContext;

/// Returns whether the command caller (determined by `ctx.author`) satisfies
/// all `permissions`. If `has_permission` returns an Error, the command should
/// either be aborted or rejected.
pub async fn has_permission(ctx: &MessageContext, permissions: &[String]) -> Result<bool, Error> {
    // Skip the permission checks if the command
    // requires no permissions.
    if permissions.is_empty() {
        return Ok(true);
    }

    // Commands from DMs are always allowed from any user.
    let guild_id = match ctx.event.guild_id {
        Some(guild_id) => guild_id,
        None => return Ok(true),
    };

    // All admins defined in the config file are always allowed.
    if ctx.state.config.admins.contains(&ctx.event.author.id.0) {
        return Ok(true);
    }

    // Get the message author.
    let member = {
        // Try member from cache.
        match ctx
            .raw_ctx
            .cache
            .member(guild_id.0, ctx.event.author.id.0)
            .await
        {
            Some(member) => member,
            None => {
                ctx.raw_ctx
                    .http
                    .get_member(guild_id.0, ctx.event.author.id.0)
                    .await?
            }
        }
    };

    for permission in permissions {
        let has_permission = ctx
            .state
            .permissions()
            .has_permission(&member, permission)
            .await?;

        // User is not allowed to run the command.
        if !has_permission {
            return Ok(false);
        }
    }

    Ok(true)
}
