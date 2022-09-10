use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GuildMember {
    pub name: String,
    pub rank: String,
    pub joined: String,
}

impl GuildMember {
    pub async fn get(guild_id: &str, access_token: &str) -> Result<Vec<Self>, reqwest::Error> {
        reqwest::get(format!(
            "https://api.guildwars2.com/v2/guild/{}/members?access_token={}",
            guild_id, access_token
        ))
        .await?
        .json()
        .await
    }
}
