use crate::help;
use robbot::{arguments::ArgumentsExt, builder::CreateMessage, command, Context, Result};
use robbot_core::{command::Command, context::MessageContext, router::find_command, state::State};
use serenity::utils::Color;

const EMBED_COLOR: Color = Color::from_rgb(0xFF, 0xA6, 0x00);

pub fn init(state: &State) {
    const COMMANDS: &[fn() -> Command] = &[help, uptime, version];

    for f in COMMANDS {
        state.commands().load_command(f(), None).unwrap();
    }
}

#[command(description = "Show the global help message or a help message for a command.")]
async fn help(mut ctx: MessageContext) -> Result {
    let description = {
        let commands = ctx.state.commands().get_inner();
        let commands = commands.read().unwrap();

        match ctx.args.is_empty() {
            // Try to show command help.
            false => match find_command(&commands, &mut ctx.args) {
                Some(command) => help::command(command),
                // Cannot find command, show global help instead.
                None => help::global(&commands),
            },
            // Show global help.
            true => help::global(&commands),
        }
    };

    ctx.respond(CreateMessage::new(|m| {
        m.embed(|e| {
            e.color(EMBED_COLOR);
            e.title("Help");
            e.description(description);
        });
    }))
    .await?;

    Ok(())
}

#[command(description = "Show the bot uptime.")]
async fn uptime(ctx: MessageContext) -> Result {
    let description = {
        let connect_time = ctx.state.connect_time.read().unwrap().unwrap();

        match connect_time.elapsed().as_secs() {
            secs if secs >= 3600 => format!(
                "{} hrs, {} min, {} sec",
                secs / 3600,
                (secs % 3600) / 60,
                secs % 60
            ),
            secs if secs >= 60 => format!("{} min, {} sec", secs / 60, secs % 60),
            secs => format!("{} sec", secs),
        }
    };

    ctx.respond(CreateMessage::new(|m| {
        m.embed(|e| {
            e.color(EMBED_COLOR);
            e.title("Uptime");
            e.description(description);
        });
    }))
    .await?;

    Ok(())
}

#[command(description = "Show the bot version.")]
async fn version(ctx: MessageContext) -> Result {
    #[cfg(debug_assertions)]
    const VERSION: &str = "`None`";

    #[cfg(not(debug_assertions))]
    const VERSION: &str = env!(
        "ROBBOT_VERSION",
        "ROBBOT_VERSION environment variable is undefined"
    );

    #[cfg(debug_assertions)]
    const BUILT: &str = "`None`";

    #[cfg(not(debug_assertions))]
    const BUILT: &str = env!(
        "ROBBOT_BUILT",
        "ROBBOT_BUILT environment variable is undefined"
    );

    ctx.respond(CreateMessage::new(|m| {
        m.embed(|e| {
            e.color(EMBED_COLOR);
            e.title("Version");
            e.description(format!("{}\nBuilt: {}", VERSION, BUILT));
        });
    }))
    .await?;

    Ok(())
}
