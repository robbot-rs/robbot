use super::utils::update_link;

use robbot::prelude::*;
use robbot_core::context::Context;

pub(super) async fn _sync<T>(ctx: Context<T>) -> Result {
    let links = ctx.state.store().get_all().await.unwrap();

    for link in links {
        update_link(&ctx, link).await?;
    }

    Ok(())
}
