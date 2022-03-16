use super::guild;

use serenity::model::guild::PartialMember;

impl From<PartialMember> for guild::PartialMember {
    fn from(src: PartialMember) -> Self {
        Self {
            deaf: src.deaf,
            joined_at: src.joined_at,
            mute: src.mute,
            nick: src.nick,
            roles: src.roles.into_iter().map(|v| v.into()).collect(),
            pending: src.pending,
            guild_id: src.guild_id.map(|v| v.into()),
            user: src.user.map(|v| v.into()),
        }
    }
}
