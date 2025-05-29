use serde::{Deserialize, Serialize};

use kfc::{guid::BlobGuid, Hash32};

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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentHash {
    pub hash0: u32,
    pub hash1: u32,
    pub hash2: u32,
    pub size: u32,
}

impl ContentHash {

    pub fn as_blob_guid(&self) -> BlobGuid {
        BlobGuid::from_parts(self.hash0, self.hash1, self.hash2, self.size)
    }

    pub fn into_blob_guid(self) -> BlobGuid {
        BlobGuid::from_parts(self.hash0, self.hash1, self.hash2, self.size)
    }

}
