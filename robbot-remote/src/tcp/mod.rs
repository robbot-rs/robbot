mod client;
mod event;
mod raw;
mod server;

use crate::proto::Message;

pub use client::Client;
pub use server::Server;

use thiserror::Error;

use std::io;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error("no response")]
    NoResponse,
    #[error("unexpected message: {0:?}")]
    UnexpectedMessage(Message),
    #[error("unimplemented")]
    Unimplemented,
    #[error("unknown error")]
    Unknown,
}
