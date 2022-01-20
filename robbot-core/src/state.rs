use crate::command::CommandHandler;
use crate::config::Config;
use crate::context::Context;
use crate::permissions::PermissionHandler;
use crate::store::mysql::MysqlStore;
use crate::store::MainStore;
use crate::task::TaskScheduler;

use std::sync::{Arc, RwLock};
use std::time::Instant;

/// The global shared state.
pub struct State {
    pub config: Arc<RwLock<Config>>,
    commands: CommandHandler,
    tasks: TaskScheduler,
    store: MainStore<MysqlStore>,
    permissions: PermissionHandler,
    pub connect_time: Arc<RwLock<Option<Instant>>>,
    pub context: Arc<RwLock<Option<Context<()>>>>,
}

impl State {
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a reference to the internal [`CommandHandler`].
    pub fn commands(&self) -> &CommandHandler {
        &self.commands
    }

    /// Returns a reference to the internal [`TaskScheduler`].
    pub fn tasks(&self) -> &TaskScheduler {
        &self.tasks
    }

    /// Returns a reference to the internal [`MainStore`].
    pub fn store(&self) -> &MainStore<MysqlStore> {
        &self.store
    }

    pub fn store_mut(&mut self) -> &mut MainStore<MysqlStore> {
        &mut self.store
    }

    /// Returns a reference to the internal [`PermissionHandler`].
    pub fn permissions(&self) -> &PermissionHandler {
        &self.permissions
    }

    pub fn context(&self) -> Option<Context<()>> {
        let context = self.context.read().unwrap();
        context.clone()
    }
}

impl Default for State {
    fn default() -> Self {
        let config = Arc::default();
        let commands = CommandHandler::new();
        let tasks = TaskScheduler::new();
        let store: MainStore<MysqlStore> = MainStore::default();
        let permissions = PermissionHandler::new(store.clone());
        let connect_time = Arc::default();
        let context = Arc::default();

        Self {
            config,
            commands,
            tasks,
            store,
            permissions,
            connect_time,
            context,
        }
    }
}
