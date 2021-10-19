use {
    crate::core::command::Command,
    std::{collections::HashSet, fmt::Write},
};

pub(crate) fn global(commands: &HashSet<Command>) -> String {
    let mut string = String::new();
    for command in commands {
        writeln!(string, "- {}", command.name).unwrap();
    }

    string
}

/// Return Command help
pub(crate) fn command(command: &Command) -> String {
    let mut string = String::new();
    writeln!(string, "**Name**: {}", command.name).unwrap();
    writeln!(string, "**Description**: {}", command.description).unwrap();

    if !command.sub_commands.is_empty() {
        writeln!(string, "**Sub Commands**:").unwrap();

        for command in &command.sub_commands {
            writeln!(string, "- {}", command.name).unwrap();
        }
    }

    string
}
