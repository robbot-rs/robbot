use super::executor::Executor;
use crate::{
    bot::Context,
    model::{GuildMessage, Message},
};
use std::{
    borrow::Borrow,
    collections::HashSet,
    hash::{Hash, Hasher},
};

#[derive(Clone)]
pub struct Command {
    pub name: String,
    pub description: String,
    /// Whether the command should only be usable inside
    /// guilds. Note that if the command is guild-only all
    /// subcommands will infer the guild-only property.
    pub guild_only: bool,
    pub sub_commands: HashSet<Command>,
    pub executor: Option<CommandExecutor>,
}

impl Command {
    pub fn new<T>(name: T) -> Self
    where
        T: ToString,
    {
        Self {
            name: name.to_string(),
            description: String::new(),
            guild_only: false,
            sub_commands: HashSet::new(),
            executor: None,
        }
    }
}

impl Hash for Command {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        self.name.hash(state);
    }
}

impl PartialEq for Command {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for Command {}

impl Borrow<str> for Command {
    fn borrow(&self) -> &str {
        &self.name
    }
}

#[derive(Clone)]
pub enum CommandExecutor {
    Message(Executor<Context<Message>>),
    GuildMessage(Executor<Context<GuildMessage>>),
}
