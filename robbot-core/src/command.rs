use crate::{
    context::MessageContext,
    executor::Executor,
    router::{find_command, parse_args},
};
use robbot::{arguments::ArgumentsExt, command::Command as CommandExt};
use std::{
    borrow::Borrow,
    collections::HashSet,
    hash::{Hash, Hasher},
    sync::{Arc, RwLock},
};

use thiserror::Error;

#[derive(Clone)]
pub struct Command {
    pub name: String,
    pub description: String,
    pub usage: String,
    pub example: String,
    pub guild_only: bool,
    /// A list of permissions required to run the command.
    /// Setting this on a non-guild-only command has no effect.
    pub permissions: Vec<String>,
    pub sub_commands: HashSet<Self>,
    pub executor: Option<Executor<MessageContext>>,
}

impl Command {
    pub fn new(name: impl ToString) -> Self {
        Self {
            name: name.to_string(),
            description: String::new(),
            usage: String::new(),
            example: String::new(),
            guild_only: false,
            executor: None,
            sub_commands: HashSet::new(),
            permissions: Vec::new(),
        }
    }

    pub fn set_name<T>(&mut self, name: T)
    where
        T: ToString,
    {
        self.name = name.to_string();
    }

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

    pub fn set_guild_only(&mut self, guild_only: bool) {
        self.guild_only = guild_only;
    }

    pub fn set_permissions<I, T>(&mut self, permissions: I)
    where
        I: IntoIterator<Item = T>,
        T: ToString,
    {
        self.permissions = permissions.into_iter().map(|n| n.to_string()).collect();
    }

    pub fn executor(&mut self, executor: Option<Executor<MessageContext>>) {
        self.executor = executor;
    }
}

impl Borrow<str> for Command {
    fn borrow(&self) -> &str {
        &self.name
    }
}

impl PartialEq for Command {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for Command {}

impl Hash for Command {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        self.name.hash(state);
    }
}

impl CommandExt for Command {
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

    fn permissions(&self) -> &[String] {
        &self.permissions
    }

    fn executor(&self) -> Option<&Self::Executor> {
        self.executor.as_ref()
    }

    fn sub_commands(&self) -> &HashSet<Self> {
        &self.sub_commands
    }
}

#[derive(Clone)]
pub struct LoadedCommand {
    pub name: String,
    pub description: String,
    pub usage: String,
    pub example: String,
    pub guild_only: bool,
    pub sub_commands: HashSet<Self>,
    pub executor: Option<Executor<MessageContext>>,
    pub permissions: Vec<String>,
}

impl From<Command> for LoadedCommand {
    fn from(command: Command) -> Self {
        Self {
            name: command.name,
            description: command.description,
            usage: command.usage,
            example: command.example,
            guild_only: command.guild_only,
            sub_commands: command
                .sub_commands
                .into_iter()
                .map(LoadedCommand::from)
                .collect(),
            executor: command.executor,
            permissions: command.permissions,
        }
    }
}

impl Borrow<str> for LoadedCommand {
    fn borrow(&self) -> &str {
        &self.name
    }
}

impl PartialEq for LoadedCommand {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for LoadedCommand {}

impl Hash for LoadedCommand {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        self.name.hash(state);
    }
}

impl CommandExt for LoadedCommand {
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

    fn permissions(&self) -> &[String] {
        &self.permissions
    }

    fn executor(&self) -> Option<&Self::Executor> {
        self.executor.as_ref()
    }
}

#[derive(Clone, Default)]
pub struct CommandHandler {
    inner: Arc<RwLock<HashSet<LoadedCommand>>>,
}

impl CommandHandler {
    /// Creates a new `CommandHandler` with no commands
    /// loaded.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the command matching `args`. If no matching command can
    /// be found `None` is returned.
    pub fn get_command<A>(&self, args: &mut A) -> Option<LoadedCommand>
    where
        A: ArgumentsExt,
    {
        let cmds = self.inner.read().unwrap();
        let command = find_command(&cmds, args)?;
        Some(command.clone())
    }

    /// Returns a list all command's names in the command root.
    pub fn list_root_commands(&self) -> Vec<String> {
        let cmds = self.inner.read().unwrap();
        cmds.iter().map(|c| c.name.clone()).collect()
    }

    /// Loads a single command. If `path` is `None`, the command
    /// will be loaded in this scope.
    pub fn load_command(&self, command: Command, path: Option<&str>) -> Result<(), Error> {
        let cmds = self.inner.write().unwrap();

        let command = LoadedCommand::from(command);

        let root_set = match path {
            Some(path) => {
                let cmd = find_command(&cmds, &mut parse_args(path).as_args())
                    .ok_or(Error::InvalidPath)?;
                &cmd.sub_commands
            }
            None => &cmds,
        };

        if root_set.contains(&command) {
            return Err(Error::DuplicateName);
        }

        // Convert &HashSet into &mut HashSet. This is a safe operation
        // as `self.commands` is write locked and changing `Command.sub_commands`
        // doesn't change it's hash.
        unsafe {
            #[allow(mutable_transmutes)]
            let root_set: &mut HashSet<LoadedCommand> = std::mem::transmute(root_set);
            root_set.insert(command);
        }

        Ok(())
    }

    /// Removes the command with the given `ident`. If a path is provided,
    /// the path will be used to find the parent command.
    pub fn remove_command(&self, ident: &str, path: Option<&str>) -> Result<(), Error> {
        let mut commands = self.inner.write().unwrap();

        let root = match path {
            Some(path) => {
                let cmd = match find_command(&commands, &mut parse_args(path).as_args()) {
                    Some(cmd) => cmd,
                    None => return Err(Error::InvalidPath),
                };

                &cmd.sub_commands
            }
            None => &mut commands,
        };

        unsafe {
            #[allow(mutable_transmutes)]
            let root: &mut HashSet<LoadedCommand> = std::mem::transmute(root);

            if !root.remove(ident) {
                return Err(Error::InvalidPath);
            }
        }

        Ok(())
    }
}

#[derive(Copy, Clone, Debug, Error)]
pub enum Error {
    #[error("duplicate name")]
    DuplicateName,
    #[error("invalid path")]
    InvalidPath,
}
