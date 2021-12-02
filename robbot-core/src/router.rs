use robbot::command::Command;
use std::collections::HashSet;

pub fn parse_args(input: &str) -> Vec<&str> {
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

    args.push(&input[start..]);

    args.into_iter().filter(|arg| !arg.is_empty()).collect()
}

pub fn find_command<'life0, T>(
    commands: &'life0 HashSet<T>,
    args: &mut Vec<&str>,
) -> Option<&'life0 T>
where
    T: Command,
{
    if args.is_empty() {
        return None;
    }

    let mut command = commands.get(args.remove(0))?;

    while let Some(arg) = args.get(0) {
        match command.sub_commands().get(*arg) {
            Some(cmd) => {
                args.remove(0);
                command = cmd;
            }
            None => break,
        }
    }

    Some(command)
}
