use super::{AssignedRole, TempRole};
use crate::bot::Context;
use crate::core::store::StoreData;
use chrono::Utc;
use robbot::prelude::*;
use serenity::model::{guild::Guild, id::GuildId};

/// `sync_roles` finds all assigned temporary roles not found by the
/// `guild_member_update` hook. This can happen when the connection is
/// interrupted. The assigned lifetime starts counting when the role is
/// first found on the member.
pub async fn sync_roles(ctx: Context<()>) -> Result {
    let now = Utc::now().timestamp();

    let guilds = ctx
        .raw_ctx
        .http
        .get_guilds(&serenity::http::GuildPagination::After(GuildId(0)), 100)
        .await?;

    for guild in guilds {
        let guild = ctx.raw_ctx.http.get_guild(guild.id.0).await?;

        let roles = ctx
            .state
            .store
            .as_ref()
            .unwrap()
            .get(Some(TempRole::query().guild_id(guild.id)))
            .await?;

        let members = guild.members(&ctx.raw_ctx, None, None).await?;

        for role in roles {
            let assigned_roles = ctx
                .state
                .store
                .as_ref()
                .unwrap()
                .get(Some(
                    AssignedRole::query()
                        .guild_id(guild.id)
                        .role_id(role.role_id),
                ))
                .await?;

            for member in &members {
                if member.roles.contains(&role.role_id)
                    && assigned_roles
                        .iter()
                        .find(|r| {
                            r.guild_id == role.guild_id
                                && r.role_id == role.role_id
                                && r.user_id == member.user.id
                        })
                        .is_none()
                {
                    let assigned_role = AssignedRole::new(
                        role.guild_id,
                        role.role_id,
                        member.user.id,
                        now + role.lifetime as i64,
                    );

                    ctx.state
                        .store
                        .as_ref()
                        .unwrap()
                        .insert(assigned_role)
                        .await?;
                }
            }
        }
    }

    Ok(())
}

pub async fn clear_roles(ctx: Context<()>) -> Result {
    let assigned_roles: Vec<AssignedRole> = ctx.state.store.as_ref().unwrap().get_all().await?;

    let now = chrono::Utc::now();

    for role in assigned_roles {
        // Skip if time is not expired yet.
        if now.timestamp() - role.remove_timestamp < 0 {
            continue;
        }

        let guild = Guild::get(&ctx.raw_ctx, role.guild_id).await?;

        let mut member = match guild.member(&ctx.raw_ctx, role.user_id).await {
            Ok(member) => member,
            Err(err) => match err {
                serenity::Error::Model(err) => {
                    if let serenity::model::ModelError::MemberNotFound = err {
                        // Member is not in guild anymore. Remove from database
                        ctx.state
                            .store
                            .as_ref()
                            .unwrap()
                            .delete(AssignedRole::query().id(role.id))
                            .await?;
                        return Ok(());
                    }

                    return Err(err.into());
                }
                _ => return Err(err.into()),
            },
        };

        // If the role was manually removed, delete it from the
        // database.
        if !member.roles.contains(&role.role_id) {
            ctx.state
                .store
                .as_ref()
                .unwrap()
                .delete(AssignedRole::query().id(role.id))
                .await?;
            continue;
        }

        // If role timestamp expired, remove the role and delete it
        // from the database.
        member.remove_role(&ctx.raw_ctx, role.role_id).await?;
        ctx.state
            .store
            .as_ref()
            .unwrap()
            .delete(AssignedRole::query().id(role.id))
            .await?;
    }

    Ok(())
}
