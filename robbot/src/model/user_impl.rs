use super::user;

use serenity::model::user::User;

impl From<User> for user::User {
    fn from(src: User) -> Self {
        Self {
            id: src.id.into(),
            avatar: src.avatar,
            bot: src.bot,
            discriminator: src.discriminator,
            name: src.name,
            banner: src.banner,
            accent_color: src.accent_colour.and_then(|c| Some(c.into())),
        }
    }
}
