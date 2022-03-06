//! robbot-remote provides an API for querying Robbot data via external sources. It also
//! provides the ability to handle bot executor callbacks (commands, tasks, etc..) from
//! an external source.
//!
//!# Proto
//! The protocol can be found in [`proto`]. All items implement [`Serialize`] and [`Deserialize`],
//! meaning that any data format that supports [`serde`] may be used.
//!
//! All items also implement [`Encode`] and [`Decode`] for efficient serialization/deserialization
//! for binary data formats.
//!
//! # Implementions
//! Implementations currently exist for:
//! - [`tcp`]
//!
//! [`Serialize`]: serde::Serialize
//! [`Deserialize`]: serde::Deserialize
//! [`Encode`]: robbot::remote::Encode
//! [`Decode`]: robbot::remote::Decode
pub mod events;
pub mod executor;
pub mod proto;
pub mod tcp;
