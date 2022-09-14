use super::Error;

use crate::context::Context;
use crate::permissions;
use crate::router::parse_args;
use crate::state::State;

use robbot::arguments::CommandArguments;
use robbot::hook::MessageData;
use robbot::model::channel::Message;
use robbot::Command;

use std::sync::Arc;

pub(crate) async fn message(
    state: Arc<State>,
    event: Message,
    raw_ctx: serenity::client::Context,
) -> Result<(), Error> {
    // Dispatch the hook event.
    let hook = MessageData(event.clone());
    state.hooks().dispatch_event(hook).await;

    // Always ignore messages from bots.
    if event.author.bot {
        return Ok(());
    }

    let prefix = &state.config.prefix;

    let content = match event.content.strip_prefix(prefix) {
        Some(content) => content,
        None => return Ok(()),
    };

    let args = parse_args(content);
    let mut args = CommandArguments::from(args);

    let command = match state.commands().get_command(&mut args) {
        Some(cmd) => cmd,
        None => return Ok(()),
    };

    let args_parsed = args.as_parsed_args().to_owned();

    let ctx = Context::new_with_args(raw_ctx, state.clone(), event, args);

    // Returns if the command is guild-only and the message was not sent from within a guild.
    if command.guild_only() && ctx.event.guild_id.is_none() {
        let _ = ctx
            .respond(":x: This command can only be used in guilds.")
            .await;

        return Ok(());
    }

    // Reject and return if the caller doesn't have all required permissions.
    #[cfg(feature = "permissions")]
    if !permissions::has_permission(&ctx, command.permissions())
        .await
        .unwrap_or(false)
    {
        return Err(Error::NoPermission);
    }

    match command.executor() {
        Some(executor) => match executor.call(ctx.clone()).await {
            Ok(_) => Ok(()),
            Err(err) => match err {
                robbot::Error::InvalidCommandUsage => {
                    Err(Error::InvalidCommandUsage(command, args_parsed))
                }
                robbot::Error::Other(err) => Err(Error::Other(err)),
                _ => Err(Error::Unknown),
            },
        },
        None => Err(Error::InvalidCommandUsage(command, args_parsed)),
    }
}
