use serde::{Deserialize, Serialize};

use crate::Hash32;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HashKey32 {
    pub value: Hash32,
}

impl From<u32> for HashKey32 {
    fn from(value: u32) -> Self {
        Self { value }
    }
}
