use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HashKey32 {
    pub value: u32,
}

impl From<u32> for HashKey32 {
    fn from(value: u32) -> Self {
        Self { value }
    }
}
