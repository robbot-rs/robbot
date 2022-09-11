#[cfg(feature = "debug")]
pub mod debug;

#[cfg(feature = "permissions")]
pub mod permissions;

pub mod events;
pub mod guildsync;

//pub mod customcommands;
//pub mod temprole;

use robbot::Result;
use robbot_core::state::State;

use std::sync::Arc;

pub async fn init(state: Arc<State>) -> Result {
    #[cfg(feature = "debug")]
    debug::init(&state).await?;

    #[cfg(feature = "permissions")]
    permissions::init(&state).await?;

    // customcommands::init(&state).await;
    guildsync::init(&state).await?;
    events::init(&state).await?;

    Ok(())
}
