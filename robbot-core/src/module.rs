use crate::command::{AddOptions, Command, CommandHandler, RemoveOptions};

use robbot::module::ModuleId;

use std::borrow::Borrow;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

use parking_lot::RwLock;

use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Debug, Error)]
pub enum Error {
    #[error("DuplicateIdent: a module with the same ident already exists")]
    DuplicateIdent,
    #[error("MaxAmountReached: reached the maximum amount of modules")]
    MaxAmountReached,
}

#[derive(Clone)]
pub struct Module {
    pub name: String,
    pub commands: HashSet<Command>,
}

impl Borrow<str> for Module {
    fn borrow(&self) -> &str {
        &self.name
    }
}

impl PartialEq for Module {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for Module {}

impl Hash for Module {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        self.name.hash(state);
    }
}

#[derive(Clone, Debug)]
pub struct LoadedModule {
    pub name: String,
    pub id: ModuleId,
}

impl LoadedModule {
    fn new(name: String, id: ModuleId) -> Self {
        Self { name, id }
    }
}

impl Borrow<str> for LoadedModule {
    fn borrow(&self) -> &str {
        &self.name
    }
}

impl PartialEq for LoadedModule {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for LoadedModule {}

impl Hash for LoadedModule {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        self.name.hash(state);
    }
}

#[derive(Debug)]
struct InnerModuleHandler {
    map: RwLock<HashSet<LoadedModule>>,
    counter: AtomicU32,
    command_handler: CommandHandler,
}

impl InnerModuleHandler {
    fn new(command_handler: CommandHandler) -> Self {
        Self {
            map: RwLock::default(),
            counter: AtomicU32::new(0),
            command_handler,
        }
    }
}

#[derive(Clone, Debug)]
pub struct ModuleHandler {
    inner: Arc<InnerModuleHandler>,
}

impl ModuleHandler {
    pub fn new(command_handler: CommandHandler) -> Self {
        Self {
            inner: Arc::new(InnerModuleHandler::new(command_handler)),
        }
    }

    /// Returns a single module.
    pub fn get_module<T>(&self, ident: &T) -> Option<LoadedModule>
    where
        T: AsRef<str>,
    {
        let modules = self.inner.map.read();

        modules.get(ident.as_ref()).cloned()
    }

    /// Returns a list of all modules.
    pub fn list_modules(&self) -> Vec<LoadedModule> {
        let modules = self.inner.map.read();

        modules.iter().cloned().collect()
    }

    /// Adds a new module to the handler. If the module has commands those will be
    /// associated under the same module id. Removing the module causes all associated
    /// commands to be removed.
    ///
    /// To avoid removal of commands when the module is removed manually, add the commands
    /// using [`CommandHandler::add_commands`].
    pub fn add_module(&self, module: Module) -> Result<ModuleId> {
        let mut modules = self.inner.map.write();

        if modules.contains(module.name.as_str()) {
            return Err(Error::DuplicateIdent);
        }

        let id = self.generate_id()?;

        let options = AddOptions::new().module_id(id);

        if !module.commands.is_empty() {
            self.inner
                .command_handler
                .inner
                .add_commands(module.commands, options)
                .unwrap();
        }

        let module = LoadedModule::new(module.name, id);

        modules.insert(module);

        Ok(id)
    }

    /// Removes a module from the handler. If the module has commands associated with
    /// the same module, those will be removed.
    pub fn remove_module(&self, name: &str) -> Result<()> {
        let mut modules = self.inner.map.write();

        let module = match modules.take(name) {
            Some(module) => module,
            None => return Ok(()),
        };

        let options = RemoveOptions::new().module_id(module.id);

        // Remove all commands associated with the module.
        // We can safely unwrap here as we haven't provided any failable
        // `RemoveOptions`.
        self.inner
            .command_handler
            .inner
            .remove_commands(options)
            .unwrap();

        Ok(())
    }

    /// Creates a new unique `ModuleId`.
    fn generate_id(&self) -> Result<ModuleId> {
        let val = self.inner.counter.fetch_add(1, Ordering::SeqCst);

        let val = match val.checked_add(1) {
            Some(val) => val,
            None => return Err(Error::MaxAmountReached),
        };

        Ok(ModuleId(val))
    }
}
