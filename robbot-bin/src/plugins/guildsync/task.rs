use super::utils::update_links;

use robbot::prelude::*;
use robbot::store::get;
use robbot::task;
use robbot_core::context::Context;

use super::GuildLink;

#[task(interval = "1h", on_load = true)]
pub(super) async fn sync<T>(ctx: Context<T>) -> Result
where
    T: Sync + Send,
{
    let links = get!(ctx.state.store(), GuildLink).await?;

    // A list of completed servers, so we don't double sync a server.
    let mut done = Vec::new();

    for link in links {
        if !done.contains(&link.guild_id) {
            update_links(&ctx, link.guild_id).await?;
            done.push(link.guild_id);
        }
    }

    Ok(())
}
