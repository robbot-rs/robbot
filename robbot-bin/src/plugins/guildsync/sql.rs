// use crate::core::store::Store;
// use futures::TryStreamExt;
// use serenity::model::id::{GuildId, RoleId, UserId};
// use sqlx::Row;

// #[derive(Clone, Debug)]
// pub(super) struct GuildMember {
//     pub id: u64,
//     pub guild_sync_id: u64,
//     pub account_name: String,
//     pub user_id: UserId,
// }

// impl GuildMember {
//     /// Returns if the data hold by `GuildMember` is correct.
//     pub fn validate(&self) -> bool {
//         self.account_name.len() >= 7 && self.account_name.len() <= 32
//     }

//     /// Returns whether `GuildMember.account_name` is in `guild_members`.
//     pub fn in_guild(&self, guild_members: &[super::gw2api::GuildMember]) -> bool {
//         guild_members
//             .iter()
//             .find(|member| member.name == self.account_name)
//             .is_some()
//     }
// }

// impl GuildMember {
//     pub fn new(guild_sync_id: u64, account_name: String, user_id: UserId) -> Self {
//         Self {
//             id: 0,
//             guild_sync_id,
//             account_name,
//             user_id,
//         }
//     }

//     pub async fn get(guild_sync_id: u64, store: &Store) -> sqlx::Result<Vec<Self>> {
//         let mut rows = sqlx::query(
//             "SELECT id, account_name, user_id FROM guildsync_guildmember WHERE guild_sync_id = ?",
//         )
//         .bind(guild_sync_id)
//         .fetch(store.pool.as_ref().unwrap());

//         let mut entries = Vec::new();

//         while let Some(row) = rows.try_next().await? {
//             let id = row.try_get("id")?;
//             let account_name = row.try_get("account_name")?;
//             let user_id = row.try_get("user_id")?;

//             entries.push(Self {
//                 id,
//                 guild_sync_id,
//                 account_name,
//                 user_id: UserId(user_id),
//             });
//         }

//         Ok(entries)
//     }

//     pub async fn get_with_user(
//         guild_sync_id: u64,
//         user_id: UserId,
//         store: &Store,
//     ) -> sqlx::Result<Vec<Self>> {
//         let mut rows = sqlx::query("SELECT id, account_name FROM guildsync_guildmember WHERE guild_sync_id = ? AND user_id = ?")
//         .bind(guild_sync_id)
//         .bind(user_id.0)
//         .fetch(store.pool.as_ref().unwrap());

//         let mut entries = Vec::new();
//         while let Some(row) = rows.try_next().await? {
//             let id = row.try_get("id")?;
//             let account_name = row.try_get("account_name")?;

//             entries.push(Self {
//                 id,
//                 guild_sync_id,
//                 account_name,
//                 user_id,
//             });
//         }

//         Ok(entries)
//     }

//     pub async fn insert(&self, store: &Store) -> sqlx::Result<()> {
//         sqlx::query("INSERT INTO guildsync_guildmember (guild_sync_id, account_name, user_id) VALUES (?, ?, ?)")
//             .bind(self.guild_sync_id).bind(&self.account_name).bind(self.user_id.0).execute(store.pool.as_ref().unwrap()).await?;
//         Ok(())
//     }

//     pub async fn delete(&self, store: &Store) -> sqlx::Result<()> {
//         sqlx::query("DELETE FROM guildsync_guildmember WHERE id = ?")
//             .bind(self.id)
//             .execute(store.pool.as_ref().unwrap())
//             .await?;
//         Ok(())
//     }
// }

// #[derive(Clone, Debug)]
// pub(super) struct GuildSync {
//     pub id: u64,
//     pub guild_id: GuildId,
//     /// GW2 Guild UID
//     pub gw_guild_id: String,
//     pub api_token: String,
// }

// impl GuildSync {
//     pub fn new(guild_id: GuildId, gw_guild_id: String, api_token: String) -> Self {
//         Self {
//             id: 0,
//             guild_id,
//             gw_guild_id,
//             api_token,
//         }
//     }

//     pub async fn get_all(store: &Store) -> sqlx::Result<Vec<Self>> {
//         let mut rows =
//             sqlx::query("SELECT id, guild_id, gw_guild_id, api_token FROM guildsync_guildlink")
//                 .fetch(store.pool.as_ref().unwrap());

//         let mut entries = Vec::new();

//         while let Some(row) = rows.try_next().await? {
//             let id = row.try_get("id")?;
//             let guild_id = row.try_get("guild_id")?;
//             let gw_guild_id = row.try_get("gw_guild_id")?;
//             let api_token = row.try_get("api_token")?;

//             entries.push(Self {
//                 id,
//                 guild_id: GuildId(guild_id),
//                 gw_guild_id,
//                 api_token,
//             });
//         }

//         Ok(entries)
//     }

//     pub async fn insert(&self, store: &Store) -> sqlx::Result<()> {
//         sqlx::query("INSERT INTO guildsync_guildlink (guild_id, gw_guild_id) VALUES (?, ?)")
//             .bind(self.guild_id.0)
//             .bind(&self.gw_guild_id)
//             .execute(store.pool.as_ref().unwrap())
//             .await?;
//         Ok(())
//     }
// }

// /// Rank <==> Role association
// #[derive(Clone, Debug)]
// pub(super) struct GuildRank {
//     pub guild_sync_id: u64,
//     pub rank_name: String,
//     pub role_id: RoleId,
// }

// impl GuildRank {
//     pub fn new(guild_sync_id: u64, rank_name: String, role_id: RoleId) -> Self {
//         Self {
//             guild_sync_id,
//             rank_name,
//             role_id,
//         }
//     }

//     pub async fn get(store: &Store, guild_sync_id: u64) -> sqlx::Result<Vec<Self>> {
//         let mut rows = sqlx::query("SELECT rank_name, role_id FROM guildsync_guildrank")
//             .fetch(store.pool.as_ref().unwrap());

//         let mut entries = Vec::new();

//         while let Some(row) = rows.try_next().await? {
//             let rank_name = row.try_get("rank_name")?;
//             let role_id = row.try_get("role_id")?;

//             entries.push(Self {
//                 guild_sync_id,
//                 rank_name,
//                 role_id: RoleId(role_id),
//             });
//         }

//         Ok(entries)
//     }

//     pub async fn insert(&self, store: &Store) -> sqlx::Result<()> {
//         sqlx::query(
//             "INSERT INTO guildsync_guildrank (guild_sync_id, rank_name, role_id) VALUES (?, ?, ?)",
//         )
//         .bind(self.guild_sync_id)
//         .bind(&self.rank_name)
//         .bind(self.role_id.0)
//         .execute(store.pool.as_ref().unwrap())
//         .await?;
//         Ok(())
//     }

//     pub async fn delete(&self, store: &Store) -> sqlx::Result<()> {
//         sqlx::query("DELETE FROM guildsync_guildrank WHERE guild_sync_id = ? AND rank_name = ?")
//             .bind(self.guild_sync_id)
//             .bind(&self.rank_name)
//             .execute(store.pool.as_ref().unwrap())
//             .await?;
//         Ok(())
//     }
// }
