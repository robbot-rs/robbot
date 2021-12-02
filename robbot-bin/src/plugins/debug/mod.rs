//! # Debug plugin
//! Adds a few info commands under the debug module
//! in debug build mode.
use robbot::{builder::CreateMessage, command, Context, Result};
use robbot_core::{context::MessageContext, state::State};
use std::{fmt::Write, sync::Arc};

pub fn init(state: Arc<State>) {
    state.commands().load_command(debug(), None).unwrap();
    state
        .commands()
        .load_command(parse_args(), Some("debug"))
        .unwrap();
    state
        .commands()
        .load_command(taskqueue(), Some("debug"))
        .unwrap();
}

crate::command!(
    debug,
    description: "Debugging and core info command. Unavaliable in release build."
);

#[command(description = "Print out all parsed arguments.", usage = "[Args...]")]
async fn parse_args(ctx: MessageContext) -> Result {
    let args: Vec<&str> = ctx.args.iter().map(|s| s.as_str()).collect();

    ctx.respond(format!("Parsed Args: `{}`", args.join("`, `")))
        .await?;
    Ok(())
}

#[command(description = "List upcoming scheduled tasks.")]
async fn taskqueue(ctx: MessageContext) -> Result {
    let mut description = String::new();

    let tasks = ctx.state.tasks().get_tasks().await;
    match tasks.len() {
        0 => description.push_str("No tasks scheduled."),
        _ => {
            for (task, execution_time) in ctx.state.tasks().get_tasks().await {
                let _ = writeln!(
                    description,
                    "Task `{}` scheduled at `{}`.",
                    task.name, execution_time
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
