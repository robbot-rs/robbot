mod commands;
mod permissions;

use robbot::Result;
use robbot_core::permissions::{RolePermission, UserPermission};
use robbot_core::state::State;

pub async fn init(state: &State) -> Result {
    state.store().create::<UserPermission>().await?;
    state.store().create::<RolePermission>().await?;

    state.commands().load_command(permissions(), None)?;

    for cmd in commands::COMMANDS {
        state.commands().load_command(cmd(), Some("permissions"))?;
    }

    Ok(())
}

crate::command!(permissions, description: "Permission handler");
