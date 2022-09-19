# Concepts

## Commands

A command is a script invoked by a chat message. Commands are the basic building block for user interaction. Commands need to be registered and have a hierarchical order. The name of a command is the key which is used to identify a command from a chat message.

### Permission handling

Sometimes it is required to limit command execution to specific users or roles (e.g. admins). Robbot provides a builtin `permissions` command and module for this purpose. When registering a command you can either set a list of permissions which are always required by the author calling the command or manually request a users permissions from the `permissions` module. If possible you should always prefer the first approach as it can provide better help messages and performance.

### Builtin commands

The default version of Robbot has a limited amount of commands builtin. Some commands 

| Command   | Description |
| --------- | ----------- |
| `help`    | Displays a help generic help message if called without arguments and a help message about a specific command if specified. |
| `version` | Shows the version of the compiled bot. |
| `uptime`  | Shows the uptime of the bot. |
| `permissions` | A top-level command that provides commands for server-based permission handling. **Note that this command only exists if Robbot was compiled using the `permissions` feature.** |
| `debug` | A top-level command that provides commands to query internal systems. **Note that command only exists if Robbot was compiled using the `debug` feature.** |

## Tasks

Tasks are used to run background tasks without requiring user interaction. Tasks can be scheduled to run at after specific time intervals, or run at exact times.

## Hooks

## Modules

