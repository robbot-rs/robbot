#[cfg(feature = "debug")]
pub mod debug;

#[cfg(feature = "permissions")]
pub mod permissions;

pub mod log;

// pub mod events;
// pub mod guildsync;

// pub mod customcommands;
// pub mod temprole;

use robbot::Result;
use robbot_core::state::State;

use std::sync::Arc;

pub async fn init(state: Arc<State>) -> Result {
    log::init(&state).await?;

    #[cfg(feature = "debug")]
    debug::init(&state).await?;

    #[cfg(feature = "permissions")]
    permissions::init(&state).await?;

    Ok(())
}
