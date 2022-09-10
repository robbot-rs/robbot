// use crate::core::store::mysql::MysqlStore;
// use futures::TryStreamExt;
// use serenity::model::id::GuildId;
// use sqlx::Row;

// #[derive(Clone, Debug)]
// pub struct Event {
//     pub id: u64,
//     pub guild_id: GuildId,
//     pub title: String,
//     pub description: String,
//     pub time: u64,
//     pub timezone: String,
//     pub repeat_time: u64,
// }

// impl Event {
//     pub fn new(
//         id: u64,
//         guild_id: GuildId,
//         title: String,
//         description: String,
//         time: u64,
//         timezone: String,
//         repeat_time: u64,
//     ) -> Self {
//         Self {
//             id,
//             guild_id,
//             title,
//             description,
//             time,
//             timezone,
//             repeat_time,
//         }
//     }

//     pub async fn get(guild_id: GuildId, store: &MysqlStore) -> sqlx::Result<Vec<Self>> {
//         let mut rows = sqlx::query("SELECT id, guild_id, title, description, _time, timezone, repeat_time FROM events_events WHERE guild_id = ?").bind(guild_id.0).fetch(&store.pool);

//         let mut entries = Vec::new();
//         while let Some(row) = rows.try_next().await? {
//             let id = row.try_get("id")?;
//             let guild_id = row.try_get("guild_id")?;
//             let title = row.try_get("title")?;
//             let description = row.try_get("description")?;
//             let time = row.try_get("_time")?;
//             let timezone = row.try_get("timezone")?;
//             let repeat_time = row.try_get("repeat_time")?;

//             entries.push(Self {
//                 id,
//                 guild_id: GuildId(guild_id),
//                 title,
//                 description,
//                 time,
//                 timezone,
//                 repeat_time,
//             });
//         }

//         Ok(entries)
//     }

//     pub async fn get_id(id: u64, store: &MysqlStore) -> sqlx::Result<Self> {
//         let row = sqlx::query("SELECT guild_id, title, description, _time, timezone, repeat_time FROM events_events WHERE id = ?")
//         .bind(id)
//         .fetch_one(&store.pool).await?;

//         Ok(Self {
//             id,
//             guild_id: GuildId(row.try_get("guild_id")?),
//             title: row.try_get("title")?,
//             description: row.try_get("description")?,
//             time: row.try_get("_time")?,
//             timezone: row.try_get("timezone")?,
//             repeat_time: row.try_get("repeat_time")?,
//         })
//     }

//     pub async fn insert(&self, store: &MysqlStore) -> sqlx::Result<()> {
//         sqlx::query("INSERT INTO events_events (guild_id, title, description, _time, timezone, repeat_time) VALUES (?, ?, ?, ?, ?, ?)")
//         .bind(self.guild_id.0)
//         .bind(&self.title)
//         .bind(&self.description)
//         .bind(self.time).bind(&self.timezone).bind(self.repeat_time)
//         .execute(&store.pool).await?;
//         Ok(())
//     }

//     pub async fn delete(&self, store: &MysqlStore) -> sqlx::Result<()> {
//         sqlx::query("DELETE FROM events_events WHERE id = ?")
//             .bind(self.id)
//             .execute(&store.pool)
//             .await?;
//         Ok(())
//     }

//     pub async fn update_time(&self, new_time: u64, store: &MysqlStore) -> sqlx::Result<()> {
//         sqlx::query("UPDATE events_events SET _time = ? WHERE id = ?")
//             .bind(new_time)
//             .bind(self.id)
//             .execute(&store.pool)
//             .await?;
//         Ok(())
//     }
// }
