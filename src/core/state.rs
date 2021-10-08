use super::router::{find_command, parse_args};
use crate::core::{
    command::Command,
    store::Store,
    task::{Task, TaskScheduler},
};
use std::{
    collections::HashSet,
    sync::{Arc, RwLock},
};

#[derive(Default)]
pub struct State {
    pub(crate) commands: Arc<RwLock<HashSet<Command>>>,
    pub(crate) task_scheduler: TaskScheduler,
    // pub(crate) hooks: Arc<RwLock<HashMap<hook::Event, Vec<Hook>>>>,
    pub store: Store,
}

impl State {
    pub fn new() -> Self {
        Self {
            commands: Arc::default(),
            task_scheduler: TaskScheduler::new(),
            // hooks: Arc::default(),
            store: Store { pool: None },
        }
    }

    /// Register a text command.
    pub fn add_command(&self, command: Command, path: Option<&str>) {
        let commands = self.commands.write().unwrap();

        let root_set = match path {
            Some(path) => {
                &find_command(&commands, &mut parse_args(path))
                    .unwrap()
                    .sub_commands
            }
            None => &commands,
        };

        // Convert &HashSet into &mut HashSet. This is a safe operation
        // as `self.commands` is write locked and changing `Command.sub_commands`
        // doesn't change it's hash.
        unsafe {
            #[allow(mutable_transmutes)]
            let root_set: &mut HashSet<Command> = std::mem::transmute(root_set);
            root_set.insert(command);
        }
    }

    pub fn add_task(&self, task: Task) {
        let tx = self.task_scheduler.clone();
        tokio::task::spawn(async move {
            tx.add_task(task).await;
        });
    }

    pub fn update_task_context(&self, ctx: Option<crate::bot::Context<()>>) {
        let tx = self.task_scheduler.clone();
        tokio::task::spawn(async move {
            tx.update_context(ctx).await;
        });
    }
}
