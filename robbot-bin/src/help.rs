use robbot::command::Command;
use std::fmt::Write;

pub(crate) fn global(commands: &[String]) -> String {
    let mut string = String::new();
    for command in commands {
        let _ = writeln!(string, "- {}", command);
    }

    string
}

/// Return Command help
pub(crate) fn command<T>(command: &T) -> String
where
    T: Command,
{
    let mut string = String::new();

    let _ = writeln!(string, "**Name**: {}", command.name());
    let _ = writeln!(string, "**Description**: {}", command.description());

    if command.executor().is_some() {
        let _ = writeln!(string, "**Usage**: {}", command.usage());
        let _ = writeln!(string, "**Example**: {}", command.example());
    }

    if !command.sub_commands().is_empty() {
        writeln!(string, "**Sub Commands**:").unwrap();

        for command in command.sub_commands() {
            writeln!(string, "- {}", command.name()).unwrap();
        }
    }

    if !command.permissions().is_empty() {
        let _ = writeln!(
            string,
            "**Required Permissions**: `{}`",
            command.permissions().join("`,`")
        );
    }

    string
}
