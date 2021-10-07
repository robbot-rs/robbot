use {
    crate::{
        bot::{self, MessageContext},
        command,
        core::{command::Command, state::State},
        help,
    },
    std::sync::Arc,
};

pub fn init(state: Arc<State>) {
    const COMMANDS: &[fn() -> Command] = &[help, uptime, version];

    for f in COMMANDS {
        state.add_command(f(), None);
    }
}

command!(help, description: "HELP", executor: _help);
async fn _help(ctx: MessageContext) -> bot::Result {
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

command!(uptime, description: "UPTIME", executor: _uptime);
async fn _uptime(ctx: MessageContext) -> bot::Result {
    ctx.event
        .channel_id
        .send_message(&ctx.raw_ctx, |m| {
            m.embed(|e| {
                e.title("Uptime");
                e
            });
            m
        })
        .await
        .unwrap();

    Ok(())
}

command!(version, description: "VERSION", executor: _version);
async fn _version(ctx: MessageContext) -> bot::Result {
    const VERSION: &str = "1.0.0";
    const BUILT: &str = "0";

    ctx.event
        .channel_id
        .send_message(&ctx.raw_ctx, |m| {
            m.embed(|e| {
                e.title("Version");
                e.description(format!("{}\n{}", VERSION, BUILT));
                e
            })
        })
        .await
        .unwrap();

    Ok(())
}
