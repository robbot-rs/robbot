//! # Debug plugin
//! Adds a few info commands under the debug module
//! in debug build mode.
use robbot::builder::CreateMessage;
use robbot::{command, Context, Result};
use robbot_core::context::MessageContext;
use robbot_core::state::State;

use std::fmt::Write;

pub async fn init(state: &State) -> Result {
    state.commands().load_command(debug(), None)?;
    state.commands().load_command(parse_args(), Some("debug"))?;
    state.commands().load_command(taskqueue(), Some("debug"))?;

    Ok(())
}

crate::command!(
    debug,
    description: "Debugging and core info command. Unavaliable in release build."
);

#[command(description = "Print out all parsed arguments.", usage = "[Args...]")]
async fn parse_args(ctx: MessageContext) -> Result {
    ctx.respond(format!("Parsed Args: `{}`", ctx.args.as_ref().join("`, `")))
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
