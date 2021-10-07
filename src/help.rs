use {
    crate::core::command::Command,
    std::{collections::HashSet, fmt::Write},
};

pub(crate) fn global(commands: &HashSet<Command>) -> String {
    let mut string = String::new();
    for command in commands {
        write!(string, "- {}\n", command.name);
    }

    string
}

/// Return Command help
pub(crate) fn command(command: &Command) -> String {
    let mut string = String::new();
    write!(string, "**Name**: {}\n", command.name);
    write!(string, "**Description**: {}\n", command.description);

    if !command.sub_commands.is_empty() {
        write!(string, "**Sub Commands**:\n");

        for command in &command.sub_commands {
            write!(string, "- {}\n", command.name);
        }
    }

    string
}
