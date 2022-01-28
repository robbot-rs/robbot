mod commands;

const PERMISSION_MANAGE: &str = "permissions.manage";

use robbot::module;
use robbot_core::permissions::{RolePermission, UserPermission};

module! {
    name: "permissions",
    cmds: {
        "permissions": {
            commands::add,
            commands::list,
            commands::remove,
        },
    },
    store: [
        UserPermission,
        RolePermission,
    ],
}
