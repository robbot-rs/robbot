//! # Debug plugin
//! Adds a few info commands under the debug module
//! in debug build mode.
use crate::{
    bot::{self, MessageContext},
    command,
    core::state::State,
};
use std::{fmt::Write, sync::Arc};

pub fn init(state: Arc<State>) {
    state.add_command(debug(), None);
    state.add_command(parse_args(), Some("debug"));
    state.add_command(taskqueue(), Some("debug"));
}

command!(
    debug,
    description: "Debugging and core info command. Unavaliable in release build."
);

command!(
    parse_args,
    description: "Print out all parsed arguments.",
    executor: _parse_args,
);
async fn _parse_args(ctx: MessageContext) -> bot::Result {
    ctx.event
        .reply(
            &ctx.raw_ctx,
            format!("Parsed Args: `{}`", ctx.args.join("`, `")),
        )
        .await?;
    Ok(())
}

command!(
    taskqueue,
    description: "WIP",
    executor: _taskqueue,
);
async fn _taskqueue(ctx: MessageContext) -> bot::Result {
    let mut string = String::new();
    for task in ctx.state.task_scheduler.get_tasks().await {
        write!(
            string,
            "Task *{}* scheduled at *{}*.\n",
            task.name, task.next_exec
        )
        .unwrap();
    }

    ctx.event.reply(&ctx.raw_ctx, string).await?;
    Ok(())
}
