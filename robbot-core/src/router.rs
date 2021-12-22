use robbot::{
    arguments::{ArgumentsExt, OwnedArguments},
    command::Command,
};
use std::collections::HashSet;

pub fn parse_args<T>(input: T) -> OwnedArguments
where
    T: AsRef<str>,
{
    let input = input.as_ref();

    let mut args = OwnedArguments::new();

    let mut start = 0;
    let mut esc = false;
    for (i, b) in input.bytes().enumerate() {
        match b {
            b if b == b' ' && !esc => {
                args.push(input[start..i].to_string());
                start = i + 1;
            }
            b'"' => {
                if esc {
                    args.push(input[start + 1..i].to_string());
                    start = i + 1;
                }
                esc = !esc;
            }
            _ => (),
        }
    }

    args.push(input[start..].to_string());

    args.iter().filter(|arg| !arg.is_empty()).collect()
}

pub fn find_command<'life0, T, U>(commands: &'life0 HashSet<T>, args: &mut U) -> Option<&'life0 T>
where
    T: Command,
    U: ArgumentsExt,
{
    if args.is_empty() {
        return None;
    }

    let mut command = commands.get(&args.pop().unwrap())?;

    while let Some(arg) = args.get(0) {
        match command.sub_commands().get(arg) {
            Some(cmd) => {
                args.pop();
                command = cmd;
            }
            None => break,
        }
    }

    Some(command)
}
