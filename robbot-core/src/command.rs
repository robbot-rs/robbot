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
    sync::Arc,
};

use robbot::module::ModuleId;

use parking_lot::RwLock;

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
    pub module_id: ModuleId,
}

impl LoadedCommand {
    fn new(command: Command, module_id: ModuleId) -> Self {
        Self {
            name: command.name,
            description: command.description,
            usage: command.usage,
            example: command.example,
            guild_only: command.guild_only,
            sub_commands: command
                .sub_commands
                .into_iter()
                .map(|cmd| LoadedCommand::new(cmd, module_id))
                .collect(),
            executor: command.executor,
            permissions: command.permissions,
            module_id,
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

#[derive(Default)]
pub(crate) struct InnerCommandHandler {
    commands: RwLock<HashSet<LoadedCommand>>,
}

impl InnerCommandHandler {
    pub fn add_commands<I>(&self, commands: I, options: AddOptions) -> Result<(), Error>
    where
        I: IntoIterator<Item = Command>,
    {
        let commands_set = self.commands.read();

        let root_set = match options.path {
            Some(path) => {
                let cmd = match find_command(&commands_set, &mut parse_args(path).as_args()) {
                    Some(cmd) => cmd,
                    None => return Err(Error::InvalidPath),
                };

                &cmd.sub_commands
            }
            None => &commands_set,
        };

        let module_id = match options.module_id {
            Some(module_id) => module_id,
            None => ModuleId::default(),
        };

        for command in commands.into_iter() {
            let mut command = LoadedCommand::new(command, module_id);

            if let Some(module_id) = options.module_id {
                command.module_id = module_id;
            }

            unsafe {
                #[allow(mutable_transmutes)]
                let root_set: &mut HashSet<LoadedCommand> = std::mem::transmute(root_set);
                root_set.insert(command);
            }
        }

        Ok(())
    }

    pub fn add_command(&self, command: Command, options: AddOptions) -> Result<(), Error> {
        self.add_commands([command], options)
    }

    /// Removes the command with the given `ident`. If a path is provided,
    /// the path will be used to find the parent command.
    pub fn remove_commands(&self, options: RemoveOptions) -> Result<(), Error> {
        let commands = self.commands.write();

        let root = match options.path {
            Some(path) => {
                let cmd = match find_command(&commands, &mut parse_args(path).as_args()) {
                    Some(cmd) => cmd,
                    None => return Err(Error::InvalidPath),
                };

                &cmd.sub_commands
            }
            None => &commands,
        };

        // Convert &HashSet into &mut HashSet. This is a safe operation
        // as `self.commands` is write locked and changing `Command.sub_commands`
        // doesn't change it's hash.
        let root: &mut HashSet<LoadedCommand> = unsafe {
            #[allow(mutable_transmutes)]
            std::mem::transmute(root)
        };

        // When a name is given only remove a single command.
        // Otherwise remove all matching commands in `root`.
        match options.name {
            Some(name) => {
                // Get the command from the collection or return Ok
                // when it is not found.
                let cmd = match root.get(name) {
                    Some(cmd) => cmd,
                    None => return Ok(()),
                };

                if let Some(module_id) = options.module_id {
                    if cmd.module_id == module_id {
                        root.remove(name);
                    }
                }
            }
            None => {
                // Retain only elments with a different module_id.
                root.retain(|cmd| match options.module_id {
                    Some(module_id) => cmd.module_id != module_id,
                    None => false,
                });
            }
        };

        Ok(())
    }
}

#[derive(Clone, Debug, Default)]
pub struct AddOptions<'a> {
    path: Option<&'a str>,
    module_id: Option<ModuleId>,
}

impl<'a> AddOptions<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn path(mut self, path: &'a str) -> Self {
        self.path = Some(path);
        self
    }

    pub fn module_id(mut self, module_id: ModuleId) -> Self {
        self.module_id = Some(module_id);
        self
    }
}

#[derive(Clone, Debug, Default)]
pub struct RemoveOptions<'a, 'b> {
    pub name: Option<&'a str>,
    pub path: Option<&'b str>,
    pub module_id: Option<ModuleId>,
}

impl<'a, 'b> RemoveOptions<'a, 'b> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn name(mut self, name: &'a str) -> Self {
        self.name = Some(name);
        self
    }

    pub fn path(mut self, path: &'b str) -> Self {
        self.path = Some(path);
        self
    }

    pub fn module_id(mut self, module_id: ModuleId) -> Self {
        self.module_id = Some(module_id);
        self
    }
}

#[derive(Clone, Default)]
pub struct CommandHandler {
    pub(crate) inner: Arc<InnerCommandHandler>,
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
        let cmds = self.inner.commands.read();
        let command = find_command(&cmds, args)?;
        Some(command.clone())
    }

    /// Returns a list all command's names in the command root.
    pub fn list_root_commands(&self) -> Vec<String> {
        let cmds = self.inner.commands.read();
        cmds.iter().map(|c| c.name.clone()).collect()
    }

    pub fn add_commands<I>(&self, commands: I, options: AddOptions) -> Result<(), Error>
    where
        I: IntoIterator<Item = Command>,
    {
        self.inner.add_commands(commands, options)
    }

    /// Loads a single command. If `path` is `None`, the command
    /// will be loaded in this scope.
    pub fn load_command(&self, command: Command, path: Option<&str>) -> Result<(), Error> {
        let mut opts = AddOptions::new();

        if let Some(path) = path {
            opts = opts.path(path);
        }

        self.inner.add_command(command, opts)
    }

    pub fn remove_command(&self, ident: &str, path: Option<&str>) -> Result<(), Error> {
        let mut opts = RemoveOptions::default().name(ident);
        opts.path = path;

        self.inner.remove_commands(opts)
    }
}

#[derive(Copy, Clone, Debug, Error)]
pub enum Error {
    #[error("duplicate name")]
    DuplicateName,
    #[error("invalid path")]
    InvalidPath,
}
