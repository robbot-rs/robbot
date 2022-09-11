use super::utils::update_link;

use robbot::prelude::*;
use robbot::store::get;
use robbot_core::context::Context;

use super::GuildLink;

pub(super) async fn _sync<T>(ctx: Context<T>) -> Result
where
    T: Sync + Send,
{
    let links = get!(ctx.state.store(), GuildLink).await?;

    for link in links {
        update_link(&ctx, link).await?;
    }

    Ok(())
}
