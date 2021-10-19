use robbot_derive::command;
use {
    crate::{
        bot::{self, MessageContext},
        core::{command::Command, state::State},
        help,
    },
    std::sync::Arc,
};

pub fn init(state: Arc<State>) {
    const COMMANDS: &[fn() -> Command] = &[help, uptime, version];

    for f in COMMANDS {
        state.add_command(f(), None).unwrap();
    }
}

// command!(help, description: "HELP", executor: _help);
#[command]
async fn help(ctx: MessageContext) -> bot::Result {
    let string = {
        let commands = ctx.state.commands.read().unwrap();
        help::global(&commands)
    };

    ctx.event
        .channel_id
        .send_message(&ctx.raw_ctx, |m| {
            m.embed(|e| {
                e.title("Help");
                e.description(string);
                e
            });
            m
        })
        .await
        .unwrap();

    Ok(())
}

// command!(uptime, description: "Show the bot uptime.", executor: _uptime);
#[command]
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

#[command]
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
