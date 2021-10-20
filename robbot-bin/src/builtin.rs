use crate::{
    bot::{self, MessageContext},
    core::{command::Command, router::route_command, state::State},
    help,
};
use robbot_derive::command;

pub fn init(state: &State) {
    const COMMANDS: &[fn() -> Command] = &[help, uptime, version];

    for f in COMMANDS {
        state.add_command(f(), None).unwrap();
    }
}

#[command(description = "Show the global help message or a help message for a command.")]
async fn help(ctx: MessageContext) -> bot::Result {
    let description = {
        let mut args: Vec<&str> = ctx.args.iter().map(|s| s.as_str()).collect();

        let commands = ctx.state.commands.read().unwrap();

        match args.is_empty() {
            // Try to show command help.
            false => match route_command(&commands, &mut args) {
                Some(command) => help::command(&command),
                // Cannot find command, show global help instead.
                None => help::global(&commands),
            },
            // Show global help.
            true => help::global(&commands),
        }
    };

    ctx.event
        .channel_id
        .send_message(&ctx.raw_ctx, |m| {
            m.embed(|e| {
                e.title("Help");
                e.description(description);
                e
            });
            m
        })
        .await
        .unwrap();

    Ok(())
}

#[command(description = "Show the bot uptime.")]
async fn uptime(ctx: MessageContext) -> bot::Result {
    let description = {
        let connect_time = ctx.state.gateway_connect_time.read().unwrap().unwrap();

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

    ctx.event
        .channel_id
        .send_message(&ctx.raw_ctx, |m| {
            m.embed(|e| {
                e.title("Uptime");
                e.description(description);
                e
            });
            m
        })
        .await
        .unwrap();

    Ok(())
}

#[command(description = "Show the bot version.")]
async fn version(ctx: MessageContext) -> bot::Result {
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

    ctx.event
        .channel_id
        .send_message(&ctx.raw_ctx, |m| {
            m.embed(|e| {
                e.title("Version");
                e.description(format!("{}\nBuilt: {}", VERSION, BUILT));
                e
            })
        })
        .await
        .unwrap();

    Ok(())
}
