//!
// use crate::core::{command::Command, task::Task};
// use std::{
//     borrow::Borrow,
//     collections::HashSet,
//     hash::{Hash, Hasher},
// };

// // /// A `Module` is a collection of [`Command`]s, [`Task`]s and
// // /// [`Hook`]s organised under a unique name.
// pub struct Module {
//     /// Name and unique identifier for the module. Loading
//     /// a second module with the same name will fail.
//     pub name: String,
//     /// A list of commands associated with a module. When
//     /// loading a module all commands will be loaded. If
//     /// a single command fails to register the module fails
//     /// to load.
//     pub commands: Option<HashSet<Command>>,
//     /// A list of Tasks associated with a module.
//     pub tasks: Option<Vec<Task>>,
// }

// impl Hash for Module {
//     fn hash<H>(&self, state: &mut H)
//     where
//         H: Hasher,
//     {
//         self.name.hash(state);
//     }
// }

// impl PartialEq for Module {
//     fn eq(&self, other: &Self) -> bool {
//         self.name == other.name
//     }
// }

// impl Eq for Module {}

// impl Borrow<str> for Module {
//     fn borrow(&self) -> &str {
//         &self.name
//     }
// }
