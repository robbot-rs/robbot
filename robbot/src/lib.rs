pub mod bot;
pub mod builder;
pub mod context;
pub mod executor;
pub mod hook;
pub mod model;

pub use context::Context;
pub use robbot_derive::command;

use std::collections::HashSet;

#[derive(Clone, Debug)]
pub struct Command {
    pub name: String,
    pub description: String,
    pub guild_only: bool,
    pub sub_commands: HashSet<Self>,
    // pub executor: Option<CommandExecutor>,
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
