use crate::store::mysql::MysqlStore;
use crate::store::{Error, MainStore};

use robbot::model::id::{GuildId, RoleId, UserId};
use robbot::StoreData;

use serenity::model::guild::Member;

// TODO: Make PermissionHandler with any type of store.
#[derive(Clone)]
pub struct PermissionHandler {
    store: MainStore<MysqlStore>,
}

impl PermissionHandler {
    /// Creates a new `PermissionHandler`. The pointer `store` must be valid
    /// for the lifetime of `PermissionHandler`.
    pub(crate) fn new(store: MainStore<MysqlStore>) -> Self {
        Self { store }
    }

    /// Returns `true` if the member effectively has the permission node.
    pub async fn has_permission(
        &self,
        member: &Member,
        node: impl AsRef<str>,
    ) -> Result<bool, Error> {
        // Check the user for permissions.
        if self
            .user_has_permission(
                UserId(member.user.id.0),
                GuildId(member.guild_id.0),
                node.as_ref(),
            )
            .await?
        {
            return Ok(true);
        }

        // Check all roles for permissions.
        for role in &member.roles {
            if self
                .role_has_permission(RoleId(role.0), GuildId(member.guild_id.0), node.as_ref())
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
        let query = UserPermission::query().guild_id(guild_id).user_id(user_id);

        self.store.get(query).await
    }

    /// Returns all permissions for a role in a single guild.
    pub async fn role_permissions(
        &self,
        role_id: RoleId,
        guild_id: GuildId,
    ) -> Result<Vec<RolePermission>, Error> {
        let query = RolePermission::query().guild_id(guild_id).role_id(role_id);

        self.store.get(query).await
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
