pub mod channel;
pub mod guild;
pub mod id;
pub mod permissions;
pub mod user;

mod channel_impl;
mod guild_impl;
mod permissions_impl;
mod user_impl;

use thiserror::Error;

/// An error indicating the model data exists in a state that is not considered valid. One should
/// always prefer using this error instead of panicking.
///
/// This would for example be an [`Message`] with the `member` field being `Some(..)` and
/// the `guild_id` being `None`.
///
/// [`Message`]: channel::Message
#[derive(Copy, Clone, Debug, PartialEq, Eq, Error)]
#[error("invalid model data")]
pub struct InvalidModelData;
