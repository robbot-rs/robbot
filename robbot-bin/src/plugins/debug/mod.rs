//! # Debug plugin
//! Adds a few info commands under the debug module
//! in debug build mode.
use crate::{
    bot::{self, Error::InvalidCommandUsage, MessageContext},
    core::state::State,
};
use robbot::{builder::CreateMessage, command, hook::EventKind, Context};
use std::{convert::TryFrom, fmt::Write, sync::Arc};

pub fn init(state: Arc<State>) {
    state.add_command(debug(), None).unwrap();
    state.add_command(parse_args(), Some("debug")).unwrap();
    state.add_command(taskqueue(), Some("debug")).unwrap();
    state.add_command(await_hook(), Some("debug")).unwrap();
}

crate::command!(
    debug,
    description: "Debugging and core info command. Unavaliable in release build."
);

#[command(description = "Print out all parsed arguments.")]
async fn parse_args(ctx: MessageContext) -> bot::Result {
    ctx.respond(format!("Parsed Args: `{}`", ctx.args.join("`, `")))
        .await?;
    Ok(())
}

#[command(description = "List upcoming scheduled tasks.")]
async fn taskqueue(ctx: MessageContext) -> bot::Result {
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

    ctx.respond(CreateMessage::new(|m| {
        m.embed(|e| {
            e.title("__Taskqueue__");
            e.description(description);
        });
    }))
    .await?;

    Ok(())
}

#[command(description = "Await a single hook of a specific type.")]
async fn await_hook(mut ctx: MessageContext) -> bot::Result {
    if ctx.args.len() != 1 {
        return Err(InvalidCommandUsage);
    }

    let event_kind = match EventKind::try_from(ctx.args.remove(0).as_str()) {
        Ok(event_kind) => event_kind,
        Err(_) => {
            let _ = ctx.respond(":x: Invalid event type.").await;
            return Ok(());
        }
    };

    let mut rx = ctx.state.hook_controller.get_receiver(event_kind).await;

    let _ = rx.recv().await.unwrap();

    let _ = ctx.respond("Got Event Data").await;

    Ok(())
}
