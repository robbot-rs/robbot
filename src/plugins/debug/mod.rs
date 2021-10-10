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
    state.add_command(debug(), None).unwrap();
    state.add_command(parse_args(), Some("debug")).unwrap();
    state.add_command(taskqueue(), Some("debug")).unwrap();
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

    let tasks = ctx.state.task_scheduler.get_tasks().await;
    match tasks.len() {
        0 => string.push_str("No tasks scheduled."),
        _ => {
            for task in ctx.state.task_scheduler.get_tasks().await {
                writeln!(
                    string,
                    "Task *{}* scheduled at *{}*.",
                    task.name, task.next_exec
                )
                .unwrap();
            }
        }
    }

    ctx.event.reply(&ctx.raw_ctx, string).await?;
    Ok(())
}
