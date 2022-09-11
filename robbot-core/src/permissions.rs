use crate::context::Context;
use crate::store::mysql::MysqlStore;
use crate::store::Error;

use robbot::model::channel::Message;
use robbot::model::id::{GuildId, RoleId, UserId};
use robbot::model::InvalidModelData;
use robbot::store::get;
use robbot::store::lazy::LazyStore;
use robbot::StoreData;

// TODO: Make PermissionHandler with any type of store.
#[derive(Clone, Debug)]
pub struct PermissionHandler {
    store: LazyStore<MysqlStore>,
}

impl PermissionHandler {
    /// Creates a new `PermissionHandler`. The pointer `store` must be valid
    /// for the lifetime of `PermissionHandler`.
    pub fn new(store: LazyStore<MysqlStore>) -> Self {
        Self { store }
    }

    /// Returns `true` if the member effectively has the permission node.
    pub async fn has_permission(
        &self,
        user_id: UserId,
        guild_id: GuildId,
        roles: &[RoleId],
        node: impl AsRef<str>,
    ) -> Result<bool, Error> {
        // Check the user for permissions.
        if self
            .user_has_permission(user_id, guild_id, node.as_ref())
            .await?
        {
            return Ok(true);
        }

        // Check all roles for permissions.
        for role_id in roles {
            if self
                .role_has_permission(*role_id, guild_id, node.as_ref())
                .await?
            {
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// Returns `true` if the user has the given permission node.
    pub async fn user_has_permission(
        &self,
        user_id: UserId,
        guild_id: GuildId,
        node: impl AsRef<str>,
    ) -> Result<bool, Error> {
        let user_nodes = self.user_permissions(user_id, guild_id).await?;

        for user_node in user_nodes {
            if user_node.node == node.as_ref() {
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// Returns `true` if the role has the given permission node.
    pub async fn role_has_permission(
        &self,
        role_id: RoleId,
        guild_id: GuildId,
        node: impl AsRef<str>,
    ) -> Result<bool, Error> {
        let role_nodes = self.role_permissions(role_id, guild_id).await?;

        for role_node in role_nodes {
            if role_node.node == node.as_ref() {
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// Returns all permissions for a user in a single guild.
    pub async fn user_permissions(
        &self,
        user_id: UserId,
        guild_id: GuildId,
    ) -> Result<Vec<UserPermission>, Error> {
        let permissions = get!(self.store, UserPermission => {
            guild_id == guild_id,
            user_id == user_id,
        })
        .await?;

        Ok(permissions)
    }

    /// Returns all permissions for a role in a single guild.
    pub async fn role_permissions(
        &self,
        role_id: RoleId,
        guild_id: GuildId,
    ) -> Result<Vec<RolePermission>, Error> {
        let permissions = get!(self.store, RolePermission => {
            guild_id == guild_id,
            role_id == role_id
        })
        .await?;

        Ok(permissions)
    }
}

/// A permission node for a user in a single guild.
#[derive(Clone, Debug, StoreData)]
pub struct UserPermission {
    pub guild_id: GuildId,
    pub user_id: UserId,
    pub node: String,
}

/// A permission node for a role in a single guild.
#[derive(Clone, Debug, StoreData)]
pub struct RolePermission {
    pub guild_id: GuildId,
    pub role_id: RoleId,
    pub node: String,
}

/// Returns whether the command caller (determined by `ctx.author`) satisfies
/// all `permissions`. If `has_permission` returns an Error, the command should
/// either be aborted or rejected.
pub async fn has_permission(ctx: &Context<Message>, permissions: &[String]) -> Result<bool, Error> {
    // Skip the permission checks if the command requires no permissions.
    if permissions.is_empty() {
        return Ok(true);
    }

    // Commands from DMs are always allowed from any user.
    let guild_id = match ctx.event.guild_id {
        Some(guild_id) => guild_id,
        None => return Ok(true),
    };

    // All admins defined in the config file are always allowed.
    if ctx.state.config.admins.contains(&ctx.event.author.id) {
        return Ok(true);
    }

    let member = match &ctx.event.member {
        Some(member) => member,
        None => return Err(InvalidModelData.into()),
    };

    let user_id = ctx.event.author.id;

    for permission in permissions {
        let has_permission = ctx
            .state
            .permissions()
            .has_permission(user_id, guild_id, &member.roles, permission)
            .await?;

        if !has_permission {
            return Ok(false);
        }
    }

    Ok(true)
}
