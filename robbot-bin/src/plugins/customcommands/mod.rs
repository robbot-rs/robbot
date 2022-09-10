mod commands;

use commands::{create, delete, list};

use robbot::arguments::ArgumentsExt;
use robbot::model::id::GuildId;
use robbot::{hook, module, Context, Result, StoreData};
use robbot_core::hook::MessageContext;

module! {
    name: "customcommands",
    cmds: {
        create,
        delete,
        list,
    },
    store: [CustomCommand,],
    tasks: [],
    hooks: [on_message,],
}

/// A guild-wide custom command.
#[derive(Clone, Debug, Default, StoreData)]
struct CustomCommand {
    guild_id: GuildId,
    /// The name of the command.
    name: String,
    /// The text message of the custom command. Display it exactly as it
    /// is saved as it contains formatting information.
    content: String,
}

impl CustomCommand {
    /// Creates a new [`CustomCommand`].
    fn new(guild_id: GuildId, name: String, content: String) -> Self {
        Self {
            guild_id,
            name,
            content,
        }
    }
}

#[hook]
async fn on_message(mut ctx: MessageContext) -> Result {
    let guild_id = match ctx.event.guild_id {
        Some(guild_id) => guild_id,
        None => return Ok(()),
    };

    let name = ctx.args.pop().unwrap();

    let query = CustomCommand::query().guild_id(guild_id).name(name);
    let command = ctx.state.store().get_one(query).await?;

    if let Some(command) = command {
        let _ = ctx.respond(command.content).await;
    }

    Ok(())
}
