//! # Debug plugin
//! Adds a few info commands under the debug module
//! in debug build mode.
use crate::{
    bot::{self, Error::InvalidCommandUsage, MessageContext},
    command,
    core::{hook::EventKind, state::State},
};
use std::{convert::TryFrom, fmt::Write, sync::Arc};

pub fn init(state: Arc<State>) {
    state.add_command(debug(), None).unwrap();
    state.add_command(parse_args(), Some("debug")).unwrap();
    state.add_command(taskqueue(), Some("debug")).unwrap();
    state.add_command(await_hook(), Some("debug")).unwrap();
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
    ctx.respond(format!("Parsed Args: `{}`", ctx.args.join("`, `")))
        .await?;
    Ok(())
}

command!(
    taskqueue,
    description: "WIP",
    executor: _taskqueue,
);
async fn _taskqueue(ctx: MessageContext) -> bot::Result {
    let mut description = String::new();

    let tasks = ctx.state.task_scheduler.get_tasks().await;
    match tasks.len() {
        0 => description.push_str("No tasks scheduled."),
        _ => {
            for task in ctx.state.task_scheduler.get_tasks().await {
                let _ = writeln!(
                    description,
                    "Task `{}` scheduled at `{}`.",
                    task.name, task.next_exec
                );
            }
        }
    }

    ctx.event
        .channel_id
        .send_message(&ctx.raw_ctx, |m| {
            m.embed(|e| {
                e.title("__Taskqueue__");
                e.description(description);
                e
            });
            m
        })
        .await?;

    Ok(())
}

command!(
    await_hook,
    description: "Await a single hook of a specific type, then drop the receiver.",
    executor: _await_hook,
);
async fn _await_hook(mut ctx: MessageContext) -> bot::Result {
    if ctx.args.len() != 1 {
        return Err(InvalidCommandUsage);
    }

    let event_kind = match EventKind::try_from(ctx.args.remove(0).as_str()) {
        Ok(event_kind) => event_kind,
        Err(_) => return Err(InvalidCommandUsage),
    };

    let mut rx = ctx.state.hook_controller.get_receiver(event_kind).await;

    let _ = rx.recv().await.unwrap();

    let _ = ctx.respond("Got Event Data").await;

    Ok(())
}
