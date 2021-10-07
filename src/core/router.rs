use crate::core::command::Command;
use std::collections::HashSet;

/// Parse an input string into a list of arguments.
/// Removes trailing spaces, double spaces between arguments and
/// other unneeded tokens.
pub(crate) fn parse_args(input: &str) -> Vec<&str> {
    let mut args = Vec::new();

    let mut start = 0;
    let mut esc = false;
    for (i, b) in input.bytes().enumerate() {
        match b {
            b if b == b' ' && !esc => {
                args.push(&input[start..i]);
                start = i + 1;
            }
            b'"' => {
                if esc {
                    args.push(&input[start + 1..i]);
                    start = i + 1;
                }
                esc = !esc;
            }
            _ => (),
        }
    }

    // Push the rest of buffer.
    args.push(&input[start..]);

    // let mut args = input.split(' ').filter(|arg| !arg.is_empty()).collect();
    args.iter()
        .filter(|arg| !arg.is_empty())
        .collect::<Vec<&&str>>()
        .iter()
        .map(|arg| **arg)
        .collect()
}

pub(crate) fn find_command<'life0>(
    commands: &'life0 HashSet<Command>,
    args: &mut Vec<&str>,
) -> Option<&'life0 Command> {
    if args.len() == 0 {
        return None;
    }

    let mut command = match commands.get(args.remove(0)) {
        Some(command) => command,
        None => return None,
    };

    while let Some(arg) = args.get(0) {
        match command.sub_commands.get(*arg) {
            Some(cmd) => {
                args.remove(0);
                command = cmd;
            }
            None => break,
        }
    }

    Some(command)
}

pub(crate) fn route_command(commands: &HashSet<Command>, args: &mut Vec<&str>) -> Option<Command> {
    if args.len() == 0 {
        return None;
    }

    let mut command = match commands.get(args.remove(0)) {
        Some(command) => command,
        None => return None,
    };

    while let Some(arg) = args.get(0) {
        match command.sub_commands.get(*arg) {
            Some(cmd) => {
                args.remove(0);
                command = cmd;
            }
            None => break,
        }
    }

    Some(command.clone())
}

#[cfg(test)]
mod tests {
    use super::{parse_args, route_command};
    use crate::core::command::Command;
    use std::collections::HashSet;

    #[test]
    fn test_parse_args() {
        let input = "Hello World";
        let output = vec!["Hello", "World"];
        assert_eq!(parse_args(input), output);

        let input = "Hello World ";
        let output = vec!["Hello", "World"];
        assert_eq!(parse_args(input), output);

        let input = "Hello     World";
        let output = vec!["Hello", "World"];
        assert_eq!(parse_args(input), output);

        let input = "\"Hello World  \"";
        let output = vec!["Hello World  "];
        assert_eq!(parse_args(input), output);

        let input = "strstr \"escaped value\"";
        let output = vec!["strstr", "escaped value"];
        assert_eq!(parse_args(input), output);
    }

    #[test]
    fn test_route_command() {
        let mut commands = HashSet::new();
        commands.insert(Command::new(String::from("test1")));
        let mut hello = Command::new(String::from("hello"));
        hello
            .sub_commands
            .insert(Command::new(String::from("world")));
        commands.insert(hello);

        let mut args = vec!["empty"];
        assert_eq!(route_command(&commands, &mut args).is_none(), true);

        let mut args = vec!["test1"];
        assert_eq!(route_command(&commands, &mut args).unwrap().name, "test1");

        let mut args = vec!["hello", "world"];
        assert_eq!(route_command(&commands, &mut args).unwrap().name, "world");
    }
}
