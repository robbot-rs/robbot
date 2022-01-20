use robbot_core::context::MessageContext;

use serenity::Error;

pub async fn has_permission(ctx: &MessageContext, permissions: &[String]) -> Result<bool, Error> {
    // Skip the permission checks if the command
    // requires no permissions.
    if permissions.is_empty() {
        return Ok(true);
    }

    // All admins defined in the config file are always
    // allowed.
    {
        let config = ctx.state.config.read().unwrap();

        if config.admins.contains(&ctx.event.author.id.0) {
            return Ok(true);
        }
    }

    // Get the message author.
    let member = {
        // Try member from cache.
        match ctx
            .raw_ctx
            .cache
            .member(ctx.event.guild_id.unwrap().0, ctx.event.author.id.0)
            .await
        {
            Some(member) => member,
            None => ctx
                .raw_ctx
                .http
                .get_member(ctx.event.guild_id.unwrap().0, ctx.event.author.id.0)
                .await
                .unwrap(),
        }
    };

    for permission in permissions {
        let has_permission = ctx
            .state
            .permissions()
            .has_permission(&member, permission)
            .await
            .unwrap();

        // User is not allowed to run the command.
        if !has_permission {
            return Ok(false);
        }
    }

    Ok(true)
}
