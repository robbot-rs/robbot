//! # Debug plugin
//! Adds a few info commands under the debug module
//! in debug build mode.
mod commands;

use robbot::module;

module! {
    name: "debug",
    cmds: {
        "debug": {
            commands::parseargs,
            commands::taskqueue,
        },
    },
}
