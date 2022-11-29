use robbot::{task, Result};
use robbot_core::context::TaskContext;

#[task(interval = "1d", on_load = true)]
pub(super) async fn update_context(ctx: TaskContext) -> Result {
    let mut cell = super::CONTEXT.write();
    *cell = Some(ctx);

    Ok(())
}
