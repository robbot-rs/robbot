use super::router::{find_command, parse_args};
use crate::core::{
    command::Command,
    hook::{Hook, HookController},
    store::Store,
    task::{Task, TaskScheduler},
};
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
    pub(crate) commands: Arc<RwLock<HashSet<Command>>>,
    pub(crate) task_scheduler: TaskScheduler,
    pub(crate) hook_controller: HookController,
    // pub(crate) hooks: Arc<RwLock<HashMap<hook::Event, Vec<Hook>>>>,
    pub store: Store,
}

impl State {
    pub fn new() -> Self {
        Self {
            commands: Arc::default(),
            task_scheduler: TaskScheduler::new(),
            hook_controller: HookController::new(),
            store: Store { pool: None },
        }
    }

    /// Register a text command.
    pub fn add_command(&self, command: Command, path: Option<&str>) -> Result<(), LoadError> {
        let commands = self.commands.write().unwrap();

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
            let root_set: &mut HashSet<Command> = std::mem::transmute(root_set);
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

    pub fn update_task_context(&self, ctx: Option<crate::bot::Context<()>>) {
        let tx = self.task_scheduler.clone();
        tokio::task::spawn(async move {
            tx.update_context(ctx).await;
        });
    }
}
