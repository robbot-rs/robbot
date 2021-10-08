//!

// /// A `Module` is a collection of [`Command`]s, [`Task`]s and
// /// [`Hook`]s organised under a unique name.
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
