use crate::help;

use robbot::arguments::{ArgumentsExt, CommandArguments};
use robbot::builder::CreateMessage;
use robbot::{command, Context, Result};
use robbot_core::command::Command;
use robbot_core::context::MessageContext;
use robbot_core::state::State;

use serenity::utils::Color;

/// The color of the embed used by all builtin commands.
pub const EMBED_COLOR: Color = Color::from_rgb(0xFF, 0xA6, 0x00);

/// Loads all builtin functions into the [`State`]. If state
/// is new or has no commands loaded, `init` will never fail.
pub fn init(state: &State) -> Result {
    const COMMANDS: &[fn() -> Command] = &[help, uptime, version];

    for f in COMMANDS {
        state.commands().load_command(f(), None)?;
    }

    Ok(())
}

/// The `help` command displays a list of all commands or details about
/// a specific command. Shows a list of all commands if no arguments are
/// given or the arguments point to a command without an executor. Shows
/// more details about a command otherwise.
#[command(
    description = "Show the global help message or a help message for a command.",
    usage = "[Path to Command]"
)]
async fn help(ctx: MessageContext) -> Result {
    // FIXME: Calling `Arguments::to_owned` clones all strings which isn't necessary here.
    let mut args = CommandArguments::from(ctx.args.as_args().to_owned());

    let command = ctx.state.commands().get_command(&mut args);
    let path = args.as_parsed_args().join(" ");

    let description = match ctx.args.is_empty() {
        // Try to show command help.
        false => match command {
            Some(command) => help::command(&command, &path, &ctx.state.config.prefix),
            // Cannot find command, show global help instead.
            None => help::global(
                &ctx.state.commands().list_root_commands(),
                &ctx.state.config.prefix,
            ),
        },
        // Show global help.
        true => help::global(
            &ctx.state.commands().list_root_commands(),
            &ctx.state.config.prefix,
        ),
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

/// The `uptime` command displays the time since the bot last connected
/// to the discord gateway. This usually is the same as the time since
/// the bot was started.
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

/// The `version` command displays the current git version of the bot.
/// The version string is loaded using the Makefile, it is not displayed
/// in the debug version.
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
