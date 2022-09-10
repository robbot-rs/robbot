use super::CustomCommand;

use robbot::arguments::ArgumentsExt;
use robbot::builder::CreateMessage;
use robbot::context::Context;
use robbot::{command, Result, StoreData};
use robbot_core::context::MessageContext;

use std::fmt::Write;

#[command]
async fn create(mut ctx: MessageContext) -> Result {
    let name: String = ctx.args.pop_parse()?;
    let content: String = ctx.args.join_rest()?;

    // The bot cannot send messages longer than
    // 2000 unicode characters.
    if content.chars().count() > 2000 {
        let _ = ctx
            .respond(":x: The maximum character count for a command is 2000.")
            .await;
    }

    let custom_command = CustomCommand::new(ctx.event.guild_id.unwrap(), name.clone(), content);
    ctx.state.store().insert(custom_command).await?;

    let _ = ctx
        .respond(format!(":white_check_mark: Created command `{}`.", name))
        .await;
    Ok(())
}

#[command]
async fn delete(mut ctx: MessageContext) -> Result {
    let name: String = ctx.args.join_rest()?;

    let guild_id = ctx.event.guild_id.unwrap();

    ctx.state
        .store()
        .delete(CustomCommand::query().guild_id(guild_id).name(name.clone()))
        .await?;

    let _ = ctx.respond(format!(":white_check_mark: Deleted command `{}`.", name));
    Ok(())
}

#[command]
async fn list(ctx: MessageContext) -> Result {
    let guild_id = ctx.event.guild_id.unwrap();

    let commands = ctx
        .state
        .store()
        .get(CustomCommand::query().guild_id(guild_id))
        .await?;

    let description = match commands.len() {
        0 => String::from("No commands."),
        _ => {
            let mut string = String::new();

            let mut iter = commands.into_iter();
            let _ = write!(string, "{}", iter.next().unwrap().name);
            for command in iter {
                let _ = write!(string, ", {}", command.name);
            }

            string
        }
    };

    let _ = ctx
        .respond(CreateMessage::new(|m| {
            m.embed(|e| {
                e.title("Custom Commands");
                e.description(description);
            });
        }))
        .await;
    Ok(())
}
