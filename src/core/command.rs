use super::executor::Executor;
use crate::bot::Context;
use std::{
    borrow::Borrow,
    collections::HashSet,
    hash::{Hash, Hasher},
};

use serenity::model::channel::Message;

#[derive(Clone, Debug)]
pub struct Command {
    pub name: String,
    pub description: String,
    pub sub_commands: HashSet<Command>,
    pub executor: Option<*const Executor<Context<Message>>>,
}

impl Command {
    pub fn new(name: String) -> Self {
        Self {
            name,
            description: String::new(),
            sub_commands: HashSet::new(),
            executor: None,
        }
    }

    /// Return a reference to the Comamnds Executor.
    pub fn executor(&self) -> Option<&Executor<Context<Message>>> {
        self.executor
            .and_then(|executor| unsafe { executor.as_ref() })
    }
}

unsafe impl Send for Command {}
unsafe impl Sync for Command {}

impl Hash for Command {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        self.name.hash(state);
    }
}

impl PartialEq for Command {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for Command {}

impl Borrow<str> for Command {
    fn borrow(&self) -> &str {
        &self.name
    }
}
