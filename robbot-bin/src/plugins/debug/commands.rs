use robbot::builder::CreateMessage;
use robbot::{command, Context, Result};
use robbot_core::context::MessageContext;

use std::fmt::Write;

#[command(
    description = "Display all parsed arguments in the same way `parse_args` does.",
    usage = "[Args...]"
)]
async fn parseargs(ctx: MessageContext) -> Result {
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

#[command(description = "List all loaded hooks.")]
async fn hooks(ctx: MessageContext) -> Result {
    let mut description = String::new();

    let hooks = ctx.state.hooks().list_hooks().await;
    match hooks.len() {
        0 => description.push_str("No Hooks loaded."),
        _ => {
            for hook in hooks {
                let _ = writeln!(
                    description,
                    "Hook `{}` enabled for event `{}`",
                    hook.name, hook.on_event
                );
            }
        }
    }

    ctx.respond(CreateMessage::new(|m| {
        m.embed(|e| {
            e.title("__Hooks__");
            e.description(description);
        });
    }))
    .await?;

    Ok(())
}

#[command(description = "List all enabled modules.")]
async fn modules(ctx: MessageContext) -> Result {
    let modules = ctx.state.modules().list_modules();
    let description = match modules.len() {
        0 => String::from("No modules loaded."),
        _ => {
            let mut string = String::new();

            for module in modules {
                let _ = writeln!(string, "`{}` (ID `{}`)", module.name, module.id.0);
            }

            string
        }
    };

    ctx.respond(CreateMessage::new(|m| {
        m.embed(|e| {
            e.title("__Modules__");
            e.description(description);
        });
    }))
    .await?;
    Ok(())
}
