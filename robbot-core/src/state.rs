use robbot::store::lazy::LazyStore;

use crate::command::CommandHandler;
use crate::config::Config;
use crate::context::Context;
use crate::hook::HookController;
use crate::module::ModuleHandler;
use crate::store::mysql::MysqlStore;
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
    modules: ModuleHandler,
    store: LazyStore<MysqlStore>,
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

        let modules = ModuleHandler::new(commands.clone());

        let store: LazyStore<MysqlStore> = LazyStore::new(&config.database.connect_string());

        #[cfg(feature = "permissions")]
        let permissions = PermissionHandler::new(store.clone());

        let connect_time = Arc::default();
        let config = Arc::new(config);

        Self {
            config,
            commands,
            tasks,
            hooks,
            modules,
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

    /// Returns a reference to the internal [`ModuleHandler`].
    pub fn modules(&self) -> &ModuleHandler {
        &self.modules
    }

    /// Returns a reference to the internal [`LazyStore`].
    pub fn store(&self) -> &LazyStore<MysqlStore> {
        &self.store
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
