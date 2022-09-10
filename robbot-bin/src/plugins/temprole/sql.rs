use crate::core::store::Store;
use futures::TryStreamExt;
use serenity::model::id::{GuildId, RoleId, UserId};
use sqlx::Row;

#[derive(Clone, Debug)]
pub(super) struct TempRole {
    pub id: u64,
    pub guild_id: GuildId,
    pub role_id: RoleId,
    /// Role lifetime in seconds.
    pub lifetime: u64,
}

impl TempRole {
    pub fn new(guild_id: GuildId, role_id: RoleId, lifetime: u64) -> Self {
        Self {
            id: 0,
            guild_id,
            role_id,
            lifetime,
        }
    }

    pub async fn get(guild_id: GuildId, store: &Store) -> sqlx::Result<Vec<Self>> {
        let mut rows =
            sqlx::query("SELECT id, role_id, lifetime FROM temprole_roles WHERE guild_id = ?")
                .bind(guild_id.0)
                .fetch(store.pool.as_ref().unwrap());

        let mut entries = Vec::new();
        while let Some(row) = rows.try_next().await? {
            let id = row.try_get("id")?;
            let role_id = row.try_get("role_id")?;
            let lifetime = row.try_get("lifetime")?;

            entries.push(Self {
                id,
                guild_id,
                role_id: RoleId(role_id),
                lifetime,
            });
        }

        Ok(entries)
    }

    pub async fn insert(&self, store: &Store) -> sqlx::Result<()> {
        sqlx::query("INSERT INTO temprole_roles (guild_id, role_id, lifetime) VALUES (?, ?, ?)")
            .bind(self.guild_id.0)
            .bind(self.role_id.0)
            .bind(self.lifetime)
            .execute(store.pool.as_ref().unwrap())
            .await?;
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub(super) struct AssignedRole {
    pub id: u64,
    pub guild_id: GuildId,
    pub role_id: RoleId,
    pub user_id: UserId,
    pub remove_timestamp: i64,
}

impl AssignedRole {
    pub fn new(guild_id: GuildId, role_id: RoleId, user_id: UserId, remove_timestamp: i64) -> Self {
        Self {
            id: 0,
            guild_id,
            role_id,
            user_id,
            remove_timestamp,
        }
    }

    pub async fn get(store: &Store) -> sqlx::Result<Vec<Self>> {
        let mut rows = sqlx::query(
            "SELECT id, guild_id, role_id, user_id, remove_timestamp FROM temprole_assigned",
        )
        .fetch(store.pool.as_ref().unwrap());

        let mut entries = Vec::new();
        while let Some(row) = rows.try_next().await? {
            let id = row.try_get("id")?;
            let guild_id = row.try_get("guild_id")?;
            let role_id = row.try_get("role_id")?;
            let user_id = row.try_get("user_id")?;
            let remove_timestamp = row.try_get("remove_timestamp")?;

            entries.push(Self {
                id,
                guild_id: GuildId(guild_id),
                role_id: RoleId(role_id),
                user_id: UserId(user_id),
                remove_timestamp,
            });
        }

        Ok(entries)
    }

    pub async fn get_with_role(
        guild_id: GuildId,
        role_id: RoleId,
        store: &Store,
    ) -> sqlx::Result<Vec<Self>> {
        let mut rows = sqlx::query("SELECT id, user_id, remove_timestamp FROm temprole_assigned")
            .fetch(store.pool.as_ref().unwrap());

        let mut entries = Vec::new();
        while let Some(row) = rows.try_next().await? {
            let id = row.try_get("id")?;
            let user_id = row.try_get("user_id")?;
            let remove_timestamp = row.try_get("remove_timestamp")?;

            entries.push(Self {
                id,
                guild_id,
                role_id,
                user_id: UserId(user_id),
                remove_timestamp,
            });
        }

        Ok(entries)
    }

    pub async fn insert(&self, store: &Store) -> sqlx::Result<()> {
        sqlx::query(
            "INSERT INTO temprole_assigned (guild_id, role_id, user_id, remove_timestamp) VALUES (?, ?, ?, ?)",
        )
        .bind(self.guild_id.0)
        .bind(self.role_id.0)
        .bind(self.user_id.0)
        .bind(self.remove_timestamp)
        .execute(store.pool.as_ref().unwrap())
        .await?;
        Ok(())
    }

    pub async fn delete(&self, store: &Store) -> sqlx::Result<()> {
        sqlx::query("DELETE FROM temprole_assigned WHERE id = ?")
            .bind(self.id)
            .execute(store.pool.as_ref().unwrap())
            .await?;
        Ok(())
    }
}
