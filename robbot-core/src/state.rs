use crate::command::CommandHandler;
use crate::config::Config;
use crate::context::Context;
use crate::hook::HookController;
use crate::store::mysql::MysqlStore;
use crate::store::MainStore;
use crate::task::TaskScheduler;

#[cfg(feature = "permissions")]
use crate::permissions::PermissionHandler;

use std::sync::{Arc, RwLock};
use std::time::Instant;

/// The global shared state.
pub struct State {
    pub config: Arc<Config>,
    commands: CommandHandler,
    tasks: TaskScheduler,
    hooks: HookController,
    store: MainStore<MysqlStore>,
    #[cfg(feature = "permissions")]
    permissions: PermissionHandler,
    pub connect_time: Arc<RwLock<Option<Instant>>>,
    pub context: Arc<RwLock<Option<Context<()>>>>,
}

impl State {
    pub fn new(config: Config) -> Self {
        let context: Arc<RwLock<Option<Context<()>>>> = Arc::default();

        let commands = CommandHandler::new();
        let tasks = TaskScheduler::new();
        let hooks = HookController::new(context.clone());
        let store: MainStore<MysqlStore> = MainStore::new(&config.database.connect_string());

        #[cfg(feature = "permissions")]
        let permissions = PermissionHandler::new(store.clone());

        let connect_time = Arc::default();
        let config = Arc::new(config);

        Self {
            config,
            commands,
            tasks,
            hooks,
            store,
            #[cfg(feature = "permissions")]
            permissions,
            connect_time,
            context,
        }
    }

    /// Returns a reference to the internal [`CommandHandler`].
    pub fn commands(&self) -> &CommandHandler {
        &self.commands
    }

    /// Returns a reference to the internal [`TaskScheduler`].
    pub fn tasks(&self) -> &TaskScheduler {
        &self.tasks
    }

    /// Returns a reference to the internal [`HookController`].
    pub fn hooks(&self) -> &HookController {
        &self.hooks
    }

    /// Returns a reference to the internal [`MainStore`].
    pub fn store(&self) -> &MainStore<MysqlStore> {
        &self.store
    }

    pub fn store_mut(&mut self) -> &mut MainStore<MysqlStore> {
        &mut self.store
    }

    /// Returns a reference to the internal [`PermissionHandler`].
    #[cfg(feature = "permissions")]
    pub fn permissions(&self) -> &PermissionHandler {
        &self.permissions
    }

    pub fn context(&self) -> Option<Context<()>> {
        let context = self.context.read().unwrap();
        context.clone()
    }
}
