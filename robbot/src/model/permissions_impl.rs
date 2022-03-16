use serenity::model::Permissions;

use super::permissions;

impl From<Permissions> for permissions::Permissions {
    fn from(src: Permissions) -> Self {
        Self { bits: src.bits }
    }
}
