use crate as robbot;
use crate::{Decode, Encode};

use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Encode, Decode)]
pub struct Color(pub u32);

impl From<serenity::utils::Color> for Color {
    fn from(c: serenity::utils::Colour) -> Self {
        Self(c.0)
    }
}
