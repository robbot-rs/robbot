//!
use crate::core::{command::Command, task::Task};
use std::{
    borrow::Borrow,
    collections::HashSet,
    hash::{Hash, Hasher},
    sync::atomic::{AtomicU64, Ordering},
};

static HANDLE_SEQ: AtomicU64 = AtomicU64::new(0);

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(transparent)]
pub struct ModuleHandle(u64);

impl ModuleHandle {
    pub(crate) fn new() -> Self {
        Self(HANDLE_SEQ.fetch_add(1, Ordering::SeqCst))
    }
}

// /// A `Module` is a collection of [`Command`]s, [`Task`]s and
// /// [`Hook`]s organised under a unique name.
#[derive(Clone)]
pub struct Module {
    /// Name and unique identifier for the module. Loading
    /// a second module with the same name will fail.
    pub name: String,
    /// A list of commands associated with a module. When
    /// loading a module all commands will be loaded. If
    /// a single command fails to register the module fails
    /// to load.
    pub commands: Option<HashSet<Command>>,
    /// A list of Tasks associated with a module.
    pub tasks: Option<Vec<Task>>,
}

impl Hash for Module {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        self.name.hash(state);
    }
}

impl PartialEq for Module {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for Module {}

impl Borrow<str> for Module {
    fn borrow(&self) -> &str {
        &self.name
    }
}

#[derive(Clone, Debug)]
pub struct LoadedModule {
    pub name: String,

    pub handle: ModuleHandle,
}

impl Hash for LoadedModule {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        self.name.hash(state);
    }
}

impl PartialEq for LoadedModule {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl PartialEq<str> for LoadedModule {
    fn eq(&self, other: &str) -> bool {
        self.name == *other
    }
}

impl Borrow<str> for LoadedModule {
    fn borrow(&self) -> &str {
        &self.name
    }
}

impl Borrow<ModuleHandle> for LoadedModule {
    fn borrow(&self) -> &ModuleHandle {
        &self.handle
    }
}

impl From<Module> for LoadedModule {
    fn from(src: Module) -> Self {
        Self {
            name: src.name,
            handle: ModuleHandle::new(),
        }
    }
}
