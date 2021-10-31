use serde::{Deserialize, Serialize};
use std::{
    borrow::Borrow,
    collections::HashSet,
    hash::{Hash, Hasher},
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Command {
    pub name: String,
    pub description: String,
    /// Defines the usage field in the help message.
    /// Required arguments should be enclosed by `[]`,
    /// optional arguments should be enclosed by `<>`.
    pub usage: String,
    pub example_usage: String,
    pub guild_only: bool,
    pub sub_commands: HashSet<Self>,
}

impl Command {
    /// Create a new [`Command`] with a name and defaulted
    /// fields.
    pub fn new<T>(name: T) -> Self
    where
        T: ToString,
    {
        Self {
            name: name.to_string(),
            description: String::new(),
            usage: String::new(),
            example_usage: String::new(),
            guild_only: false,
            sub_commands: HashSet::new(),
        }
    }

    /// Set the `description` field of the command.
    pub fn description<T>(&mut self, description: T)
    where
        T: ToString,
    {
        self.description = description.to_string();
    }

    /// Set the `usage` field of the command.
    pub fn usage<T>(&mut self, usage: T)
    where
        T: ToString,
    {
        self.usage = usage.to_string();
    }

    /// Set the `example_usage` field of the command.
    pub fn example_usage<T>(&mut self, example_usage: T)
    where
        T: ToString,
    {
        self.example_usage = example_usage.to_string();
    }

    /// Set the `guild_only` field of the command.
    pub fn guild_only<T>(&mut self, guild_only: bool) {
        self.guild_only = guild_only;
    }
}

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
