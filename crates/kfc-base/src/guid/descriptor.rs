use std::fmt::{Debug, Display};
use std::io::{Read, Write};
use std::str::FromStr;

use serde::Deserialize;

use crate::{container::StaticHash, hash::fnv_with_seed, io::{ReadExt, WriteExt}, Hash32};

use super::BlobGuid;

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct DescriptorGuid {
    pub data: [u8; 16],
    pub type_hash: Hash32,
    pub part_number: u32,
    // pub reserved: u64,
}

impl DescriptorGuid {

    pub const NONE: DescriptorGuid = DescriptorGuid {
        data: [0; 16],
        type_hash: 0,
        part_number: 0,
    };

    pub fn from_bytes(data: [u8; 16], type_hash: Hash32, part_number: u32) -> Self {
        Self {
            data,
            type_hash,
            part_number,
        }
    }

    /// Create a new DescriptorGuid from a string with following format:
    /// `XXXXXXXX-XXXX-XXXX-XXXX-XXXXXXXXXXXX`
    /// where `X` is a hexadecimal digit.
    ///
    /// If the string is not in the correct format, `None` is returned.
    pub fn from_str(s: &str, type_hash: Hash32, part_number: u32) -> Option<Self> {
        Some(Self {
            data: super::string_to_guid(s)?,
            type_hash,
            part_number,
        })
    }

    /// Create a new DescriptorGuid from a string with following format:
    /// `XXXXXXXX-XXXX-XXXX-XXXX-XXXXXXXXXXXX_XXXXXXXX_N`
    /// where `X` is a hexadecimal digit.
    ///
    /// If the string is not in the correct format, `None` is returned.
    pub fn from_qualified_str(s: &str) -> Option<Self> {
        if s.len() < 47 {
            return None;
        }

        if !super::is_section_separator(s[36..].chars().next().unwrap()) ||
            !super::is_hex_slice(&s[37..45]) ||
            !super::is_section_separator(s[45..].chars().next().unwrap())
        {
            return None;
        }

        let type_hash = u32::from_str_radix(&s[37..45], 16).ok()?;
        let part_number = u32::from_str(&s[46..]).ok()?;

        Self::from_str(&s[0..36], type_hash, part_number)
    }

    /// Convert the DescriptorGuid to a string with following format:
    /// `XXXXXXXX-XXXX-XXXX-XXXX-XXXXXXXXXXXX`
    /// where `X` is a hexadecimal digit.
    ///
    /// Type hash and part number are not included in the string.
    #[allow(clippy::inherent_to_string_shadow_display)] // this is intentional
    pub fn to_string(&self) -> String {
        super::guid_to_string(&self.data)
    }

    /// Convert the DescriptorGuid to a qualified string with following format:
    /// `XXXXXXXX-XXXX-XXXX-XXXX-XXXXXXXXXXXX_XXXXXXXX_N`
    /// where `X` is a hexadecimal digit and `N` is a 32-bit decimal integer.
    pub fn to_qualified_string(&self) -> String {
        format!("{}_{:0>8x}_{}", self.to_string(), self.type_hash, self.part_number)
    }

    pub fn hash32(&self) -> Hash32 {
        self.as_blob_guid().hash32()
    }

    pub fn is_none(&self) -> bool {
        self.data == [0; 16]
    }

    pub fn as_blob_guid(&self) -> BlobGuid {
        BlobGuid {
            data: self.data,
        }
    }

    pub fn with_type_hash(&self, type_hash: Hash32) -> Self {
        Self {
            data: self.data,
            type_hash,
            part_number: self.part_number,
        }
    }

    pub fn with_part_number(&self, part_number: u32) -> Self {
        Self {
            data: self.data,
            type_hash: self.type_hash,
            part_number,
        }
    }

}

impl StaticHash for DescriptorGuid {
    fn static_hash(&self) -> u32 {
        let seed = u32::from_le_bytes(self.data[0..4].try_into().unwrap());
        let mut rest = [0u8; 8];
        rest[0..4].copy_from_slice(self.type_hash.to_le_bytes().as_ref());
        rest[4..8].copy_from_slice(self.part_number.to_le_bytes().as_ref());

        fnv_with_seed(rest, seed)
    }
}

impl Display for DescriptorGuid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl DescriptorGuid {

    pub fn read<R: Read>(reader: &mut R) -> std::io::Result<Self> {
        let mut data = [0; 16];
        reader.read_exact(&mut data)?;

        let type_hash = reader.read_u32()?;
        let part_number = reader.read_u32()?;
        reader.padding(8)?;

        Ok(Self {
            data,
            type_hash,
            part_number,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        writer.write_all(&self.data)?;
        writer.write_u32(self.type_hash)?;
        writer.write_u32(self.part_number)?;
        writer.padding(8)?;

        Ok(())
    }

}

impl<'de> Deserialize<'de> for DescriptorGuid {
    fn deserialize<D>(deserializer: D) -> Result<DescriptorGuid, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        DescriptorGuid::from_qualified_str(&s)
            .ok_or_else(|| serde::de::Error::custom("invalid DescriptorGuid"))
    }
}

impl serde::Serialize for DescriptorGuid {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.to_qualified_string().serialize(serializer)
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test_guid_qualified_string() {
        const GUID: &str = "40e6ba42-a397-5790-a5c9-a4151fffe1c5_647628d6_420";

        let guid = super::DescriptorGuid::from_qualified_str(GUID).unwrap();

        assert_eq!(guid.to_qualified_string(), GUID);
    }

    #[test]
    fn test_guid_string() {
        const GUID: &str = "40e6ba42-a397-5790-a5c9-a4151fffe1c5";

        let guid = super::DescriptorGuid::from_str(GUID, 0, 0).unwrap();

        assert_eq!(guid.to_string(), GUID);
    }
}
