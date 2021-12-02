use crate::{command::CommandHandler, config::Config, store::MainStore, task::TaskScheduler};
use std::{
    sync::{Arc, RwLock},
    time::Instant,
};

use crate::store::mysql::MysqlStore;

/// The global shared state.
#[derive(Default)]
pub struct State {
    pub config: Arc<RwLock<Config>>,
    commands: CommandHandler,
    tasks: TaskScheduler,
    store: MainStore<MysqlStore>,
    pub connect_time: Arc<RwLock<Option<Instant>>>,
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
}
