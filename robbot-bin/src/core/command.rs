use super::executor::Executor;
use crate::{bot::MessageContext, core::module::ModuleHandle};
use std::{
    borrow::Borrow,
    collections::HashSet,
    hash::{Hash, Hasher},
};

#[derive(Clone)]
pub struct Command {
    pub name: String,
    /// Description of the command. Shown to the user in
    /// the help command.
    pub description: String,
    /// Usage of the command. Shown to the user in the help
    /// command. Use `<>` for required arguments and `[]` for
    /// optional commands. Only write the arguments to the usage
    /// field, **do not** write the full command path.
    /// Example: `<User> [Message]`
    pub usage: String,
    /// Example usage shown to the user in the help command.
    /// The example should match defined usage.
    pub example: String,
    /// Whether the command should only be usable inside
    /// guilds. Note that if the command is guild-only all
    /// subcommands will infer the guild-only property.
    pub guild_only: bool,
    pub sub_commands: HashSet<Command>,
    pub executor: Option<Executor<MessageContext>>,
}

impl Command {
    pub fn new<T>(name: T) -> Self
    where
        T: ToString,
    {
        Self {
            name: name.to_string(),
            description: String::new(),
            usage: String::new(),
            example: String::new(),
            guild_only: false,
            sub_commands: HashSet::new(),
            executor: None,
        }
    }

    /// Set the `description` field of the command.
    pub fn set_description<T>(&mut self, description: T)
    where
        T: ToString,
    {
        self.description = description.to_string();
    }

    pub fn set_usage<T>(&mut self, usage: T)
    where
        T: ToString,
    {
        self.usage = usage.to_string();
    }

    pub fn set_example<T>(&mut self, example: T)
    where
        T: ToString,
    {
        self.example = example.to_string();
    }

    /// Set the `guild_only` field of the command.
    pub fn set_guild_only(&mut self, guild_only: bool) {
        self.guild_only = guild_only;
    }
}

impl robbot::Command for Command {
    type Executor = Executor<MessageContext>;

    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn usage(&self) -> &str {
        &self.usage
    }

    fn example(&self) -> &str {
        &self.example
    }

    fn guild_only(&self) -> bool {
        self.guild_only
    }

    fn sub_commands(&self) -> &HashSet<Self> {
        &self.sub_commands
    }

    fn executor(&self) -> Option<&Executor<MessageContext>> {
        self.executor.as_ref()
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
pub(crate) struct LoadedCommand {
    pub name: String,
    pub description: String,
    pub usage: String,
    pub example: String,
    pub guild_only: bool,
    pub sub_commands: HashSet<Self>,
    pub executor: Option<Executor<MessageContext>>,

    pub module_handle: Option<ModuleHandle>,
}

impl LoadedCommand {
    pub fn new(command: Command, handle: Option<ModuleHandle>) -> Self {
        Self {
            name: command.name,
            description: command.description,
            usage: command.usage,
            example: command.example,
            guild_only: command.guild_only,
            sub_commands: {
                let mut hashset = HashSet::with_capacity(command.sub_commands.capacity());

                for cmd in command.sub_commands {
                    hashset.insert(LoadedCommand::new(cmd, handle));
                }

                hashset
            },
            executor: command.executor,
            module_handle: handle,
        }
    }
}

impl robbot::Command for LoadedCommand {
    type Executor = Executor<MessageContext>;

    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn usage(&self) -> &str {
        &self.usage
    }

    fn example(&self) -> &str {
        &self.example
    }

    fn guild_only(&self) -> bool {
        self.guild_only
    }

    fn sub_commands(&self) -> &HashSet<Self> {
        &self.sub_commands
    }

    fn executor(&self) -> Option<&Self::Executor> {
        self.executor.as_ref()
    }
}

impl Hash for LoadedCommand {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        self.name.hash(state);
    }
}

impl PartialEq for LoadedCommand {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for LoadedCommand {}

impl Borrow<str> for LoadedCommand {
    fn borrow(&self) -> &str {
        &self.name
    }
}
