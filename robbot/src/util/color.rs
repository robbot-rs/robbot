use crate as robbot;
use crate::{Decode, Encode};

use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Encode, Decode)]
pub struct Color(pub u32);

impl Color {
    pub const fn from_rgb(red: u8, green: u8, blue: u8) -> Self {
        Self((red as u32) << 16 | (green as u32) << 8 | blue as u32)
    }
}

impl From<serenity::utils::Color> for Color {
    fn from(c: serenity::utils::Colour) -> Self {
        Self(c.0)
    }
}
