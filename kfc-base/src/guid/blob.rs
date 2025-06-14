use std::fmt::{Debug, Display};
use std::io::{Read, Write};
use std::str::FromStr;

use serde::Deserialize;

use crate::hash::compute_blob_guid;
use crate::{container::StaticHash, hash::fnv, Hash32};

use super::DescriptorGuid;

#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct BlobGuid {
    pub data: [u8; 16],
}

impl BlobGuid {

    pub const NONE: BlobGuid = BlobGuid {
        data: [0; 16],
    };

    pub fn from_bytes(data: [u8; 16]) -> Self {
        Self {
            data
        }
    }

    pub fn from_data(data: &[u8]) -> Self {
        let data_size = data.len();
        let guid = compute_blob_guid(data, 0);

        Self::from_parts(
            data_size as u32,
            u32::from_le_bytes(guid[4..8].try_into().unwrap()),
            u32::from_le_bytes(guid[8..12].try_into().unwrap()),
            u32::from_le_bytes(guid[12..16].try_into().unwrap()),
        )
    }

    pub fn from_parts(
        size: u32,
        hash0: u32,
        hash1: u32,
        hash2: u32,
    ) -> Self {
        let mut data = [0; 16];

        data[0..4].copy_from_slice(&size.to_le_bytes());
        data[4..8].copy_from_slice(&hash0.to_le_bytes());
        data[8..12].copy_from_slice(&hash1.to_le_bytes());
        data[12..16].copy_from_slice(&hash2.to_le_bytes());

        Self {
            data
        }
    }

    pub fn hash32(&self) -> Hash32 {
        fnv(self.data)
    }

    pub fn size(&self) -> u32 {
        let data: [u8; 4] = self.data[0..4].try_into().unwrap();
        u32::from_le_bytes(data)
    }

    pub fn is_none(&self) -> bool {
        self.data == [0; 16]
    }

    pub fn as_descriptor_guid(&self, type_hash: u32, part_number: u32) -> DescriptorGuid {
        DescriptorGuid {
            data: self.data,
            type_hash,
            part_number,
        }
    }

}

impl StaticHash for BlobGuid {
    fn static_hash(&self) -> u32 {
        u32::from_le_bytes([
            self.data[4],
            self.data[5],
            self.data[6],
            self.data[7],
        ])
    }
}

impl FromStr for BlobGuid {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(BlobGuid {
            data: super::string_to_guid(s)
                .ok_or_else(|| format!("Invalid BlobGuid string: {}", s))?,
        })
    }
}

impl Debug for BlobGuid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "\"{}\" ({})", self, self.hash32())
    }
}

impl Display for BlobGuid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", super::guid_to_string(&self.data))
    }
}

impl<'de> Deserialize<'de> for BlobGuid {
    fn deserialize<D>(deserializer: D) -> Result<BlobGuid, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        BlobGuid::from_str(&s).map_err(serde::de::Error::custom)
    }
}

impl serde::Serialize for BlobGuid {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.to_string().serialize(serializer)
    }
}

impl BlobGuid {

    pub fn read<R: Read>(reader: &mut R) -> std::io::Result<Self> {
        let mut data = [0; 16];
        reader.read_exact(&mut data)?;

        Ok(Self {
            data
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        writer.write_all(&self.data)
    }

}
