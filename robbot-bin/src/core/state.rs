use super::router::{find_command, parse_args};
use crate::core::{
    command::{Command, LoadedCommand},
    hook::HookController,
    module::{LoadedModule, Module, ModuleHandle},
    store::MainStore,
    task::{Task, TaskScheduler},
};
use robbot::hook::Hook;
use std::{
    collections::HashSet,
    error,
    fmt::{self, Display, Formatter},
    sync::{Arc, RwLock},
};

/// An error returned when loading a command,
/// task or hook.
#[derive(Clone, Debug)]
pub enum LoadError {
    /// The provided insertion path for the command
    /// doesn't exist.
    InvalidPath,
    /// A command, task or hook with the same name
    /// already exists.
    DuplicateName,
}

impl Display for LoadError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::InvalidPath => "invalid path",
                Self::DuplicateName => "duplicate name",
            }
        )
    }
}

impl error::Error for LoadError {}

#[derive(Default)]
pub struct State {
    pub(crate) config: Arc<RwLock<crate::config::Config>>,
    pub(crate) modules: Arc<RwLock<Vec<LoadedModule>>>,
    pub(crate) commands: Arc<RwLock<HashSet<LoadedCommand>>>,
    pub(crate) task_scheduler: TaskScheduler,
    pub(crate) hook_controller: HookController,
    pub store: Option<MainStore<crate::core::store::mysql::MysqlStore>>,
    pub(crate) gateway_connect_time: Arc<RwLock<Option<std::time::Instant>>>,
    /// TODO: Move hook_id handling into hook logic.
    pub(crate) hook_id: Arc<std::sync::atomic::AtomicUsize>,
}

impl State {
    pub fn new() -> Self {
        Self {
            config: Arc::default(),
            modules: Arc::default(),
            commands: Arc::default(),
            task_scheduler: TaskScheduler::new(),
            hook_controller: HookController::new(),
            store: None,
            gateway_connect_time: Arc::default(),
            hook_id: Arc::default(),
        }
    }

    pub fn load_module(&self, module: Module) -> Result<ModuleHandle, LoadError> {
        let mut modules = self.modules.write().unwrap();
        let mut commands = self.commands.write().unwrap();

        let handle = ModuleHandle::new();

        if modules.iter().any(|m| m.name == module.name) {
            return Err(LoadError::DuplicateName);
        }

        if let Some(cmds) = &module.commands {
            for cmd in cmds {
                if commands.contains(cmd.name.as_str()) {
                    return Err(LoadError::DuplicateName);
                }
            }

            for cmd in cmds {
                commands.insert(LoadedCommand::new(cmd.clone(), Some(handle)));
            }
        }

        if let Some(tasks) = &module.tasks {
            for task in tasks {
                self.add_task(task.clone());
            }
        }

        modules.push(module.into());

        Ok(handle)
    }

    /// Unload the module with the given handle. Removes all commands
    /// that are loaded under the module.
    pub fn unload_module(&self, handle: ModuleHandle) -> Result<(), LoadError> {
        let mut modules = self.modules.write().unwrap();
        let mut commands = self.commands.write().unwrap();

        // Remove all top-level commands with the same handle
        // as the module.
        commands.retain(|cmd| {
            if let Some(cmd_handle) = cmd.module_handle {
                if cmd_handle == handle {
                    return false;
                }
            }

            true
        });

        // Remove the loaded module with the same handle.
        modules.retain(|m| m.handle != handle);

        Ok(())
    }

    /// Register a text command.
    pub fn add_command(&self, command: Command, path: Option<&str>) -> Result<(), LoadError> {
        let commands = self.commands.write().unwrap();

        let command = LoadedCommand::new(command, None);

        let root_set = match path {
            Some(path) => {
                let cmd = match find_command(&commands, &mut parse_args(path)) {
                    Some(cmd) => cmd,
                    None => return Err(LoadError::InvalidPath),
                };

                &cmd.sub_commands
            }
            None => &commands,
        };

        // Check for command with same name.
        if root_set.contains(&command) {
            return Err(LoadError::DuplicateName);
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

    pub fn add_task(&self, task: Task) {
        let tx = self.task_scheduler.clone();
        tokio::task::spawn(async move {
            tx.add_task(task).await;
        });
    }

    pub async fn add_hook(
        &self,
        hook: Hook,
    ) -> tokio::sync::broadcast::Receiver<super::hook::Event> {
        self.hook_controller.add_hook(hook).await
    }

    pub async fn remove_hook(&self, name: &str) {
        self.hook_controller.remove_hook(name).await;
    }

    pub fn update_task_context(&self, ctx: Option<crate::bot::Context<()>>) {
        let tx = self.task_scheduler.clone();
        tokio::task::spawn(async move {
            tx.update_context(ctx).await;
        });
    }
}
