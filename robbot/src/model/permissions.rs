use crate as robbot;
use crate::{Decode, Encode};

use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, Serialize, Deserialize, Encode, Decode)]
pub struct Permissions {
    pub bits: u64,
}
