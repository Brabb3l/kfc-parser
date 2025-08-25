use serde::{Deserialize, Serialize};

use kfc::Hash32;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HashKey32 {
    pub value: Hash32,
}

impl From<u32> for HashKey32 {
    fn from(value: u32) -> Self {
        Self { value }
    }
}
