use robbot::command::Command;

use std::fmt::Write;

/// Returns a new global help message string.
pub(crate) fn global(commands: &[String], prefix: &str) -> String {
    // FIXME: Can pre-allocate at least the prefix and suffix of the string.
    let mut string = String::new();

    let _ = writeln!(string, "__**Commands:**__");

    for command in commands {
        let _ = writeln!(string, "- {}", command);
    }

    let _ = writeln!(
        string,
        "\n**Use `{}help`*`command`* to get more details about a command.**",
        prefix
    );

    string
}

/// Return a new help message for a specific command.
///
/// The given `path` and `prefix` values are used to correctly construct the "Usage"
/// and "Example" fields.
pub(crate) fn command<T>(command: &T, path: &str, prefix: &str) -> String
where
    T: Command,
{
    let mut string = String::new();

    let _ = writeln!(string, "**Name**: {}", command.name());
    let _ = writeln!(string, "**Description**: {}", command.description());

    if command.executor().is_some() {
        let _ = writeln!(string, "**Usage**: {}{} {}", prefix, path, command.usage());
        let _ = writeln!(
            string,
            "**Example**: {}{} {}",
            prefix,
            path,
            command.example()
        );
    }

    if !command.sub_commands().is_empty() {
        writeln!(string, "**Sub-Commands**:").unwrap();

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
