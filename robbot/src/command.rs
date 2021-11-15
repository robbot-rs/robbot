use std::{borrow::Borrow, collections::HashSet, hash::Hash};

pub trait Command: Sized + Hash + Eq + Borrow<str> {
    type Executor;

    fn name(&self) -> &str;
    /// Description of the command. Shown to the user in
    /// the help command.
    fn description(&self) -> &str;
    /// Usage of the command. Shown to the user in the help
    /// command. Use `<>` for required arguments and `[]` for
    /// optional commands. Only write the arguments to the usage
    /// field, **do not** write the full command path.
    /// Example: `<User> [Message]`
    fn usage(&self) -> &str;
    /// Example usage shown to the user in the help command.
    /// The example should match defined usage.
    fn example(&self) -> &str;
    /// Whether the command should only be usable inside
    /// guilds. Note that if the command is guild-only all
    /// subcommands will infer the guild-only property.
    fn guild_only(&self) -> bool;
    fn sub_commands(&self) -> &HashSet<Self>;
    fn executor(&self) -> Option<&Self::Executor>;
}
