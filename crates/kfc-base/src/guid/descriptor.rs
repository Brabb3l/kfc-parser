use std::fmt::{Debug, Display};
use std::io::{Read, Write};

use serde::Deserialize;

use crate::hash::fnv_bytes_with_seed;
use crate::{container::StaticHash, io::{ReadExt, WriteExt}, Hash32};

use super::BlobGuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct DescriptorGuid {
    pub data: [u8; 16],
    pub type_hash: Hash32,
    pub part_number: u32,
    // pub reserved: u64,
}

impl DescriptorGuid {

    pub const NONE: Self = Self {
        data: [0; 16],
        type_hash: 0,
        part_number: 0,
    };

    #[inline]
    #[must_use]
    pub const fn new(data: [u8; 16], type_hash: Hash32, part_number: u32) -> Self {
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
    #[inline]
    #[must_use]
    pub const fn parse(s: &str, type_hash: Hash32, part_number: u32) -> Option<Self> {
        let data = match BlobGuid::parse(s) {
            Some(data) => data.into_data(),
            None => return None,
        };

        Some(Self {
            data,
            type_hash,
            part_number,
        })
    }

    /// Create a new DescriptorGuid from a string with following format:
    /// `XXXXXXXX-XXXX-XXXX-XXXX-XXXXXXXXXXXX_XXXXXXXX_N`
    /// where `X` is a hexadecimal digit.
    ///
    /// If the string is not in the correct format, `None` is returned.
    #[inline]
    #[must_use]
    #[deprecated(note = "the format is not very user-friendly, use your own format instead")]
    pub const fn parse_qualified(s: &str) -> Option<Self> {
        str_to_qualified_guid(s)
    }

    /// Convert the DescriptorGuid to a string with following format:
    /// `XXXXXXXX-XXXX-XXXX-XXXX-XXXXXXXXXXXX`
    /// where `X` is a hexadecimal digit.
    ///
    /// Type hash and part number are not included in the string.
    #[allow(clippy::inherent_to_string_shadow_display)] // this is intentional
    #[inline]
    #[must_use]
    pub fn to_string(&self) -> String {
        self.as_blob_guid().to_string()
    }

    /// Convert the DescriptorGuid to a qualified string with following format:
    /// `XXXXXXXX-XXXX-XXXX-XXXX-XXXXXXXXXXXX_XXXXXXXX_N`
    /// where `X` is a hexadecimal digit and `N` is a 32-bit decimal integer.
    #[inline]
    #[must_use]
    #[deprecated(note = "the format is not very user-friendly, use your own format instead")]
    pub fn to_qualified_string(&self) -> String {
        format!("{}_{:0>8x}_{}", self.as_blob_guid(), self.type_hash, self.part_number)
    }

    #[inline]
    pub fn is_none(&self) -> bool {
        self.data == [0; 16]
    }

    #[inline]
    #[must_use]
    pub const fn hash32(&self) -> Hash32 {
        self.as_blob_guid().hash32()
    }

    #[inline]
    #[must_use]
    pub const fn as_blob_guid(&self) -> BlobGuid {
        BlobGuid::new(self.data)
    }

    #[inline]
    #[must_use]
    pub const fn with_type_hash(&self, type_hash: Hash32) -> Self {
        Self {
            data: self.data,
            type_hash,
            part_number: self.part_number,
        }
    }

    #[inline]
    #[must_use]
    pub const fn with_part_number(&self, part_number: u32) -> Self {
        Self {
            data: self.data,
            type_hash: self.type_hash,
            part_number,
        }
    }

}

impl StaticHash for DescriptorGuid {

    #[inline]
    fn static_hash(&self) -> u32 {
        let seed = u32::from_le_bytes(self.data[0..4].try_into().unwrap());
        let mut rest = [0u8; 8];

        rest[0..4].copy_from_slice(&self.type_hash.to_le_bytes());
        rest[4..8].copy_from_slice(&self.part_number.to_le_bytes());

        fnv_bytes_with_seed(&rest, seed)
    }

}

impl Display for DescriptorGuid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl DescriptorGuid {

    #[inline]
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

    #[inline]
    pub fn write<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        writer.write_all(&self.data)?;
        writer.write_u32(self.type_hash)?;
        writer.write_u32(self.part_number)?;
        writer.padding(8)?;

        Ok(())
    }

}

impl<'de> Deserialize<'de> for DescriptorGuid {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;

        #[allow(deprecated)]
        Self::parse_qualified(&s)
            .ok_or_else(|| serde::de::Error::custom("invalid DescriptorGuid"))
    }
}

impl serde::Serialize for DescriptorGuid {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        #[allow(deprecated)]
        self.to_qualified_string().serialize(serializer)
    }
}

const fn str_to_qualified_guid(input: &str) -> Option<DescriptorGuid> {
    if input.len() < 47 {
        return None;
    }

    let guid = match BlobGuid::parse(input) {
        Some(guid) => guid.into_data(),
        None => return None,
    };

    let input = input.as_bytes();

    if input[36] != b'_' || input[45] != b'_' {
        return None;
    }

    let type_hash = match super::blob::hex_to_bytes::<4>(input, 37) {
        Some(bytes) => u32::from_be_bytes(bytes),
        None => return None,
    };

    let part_number = match dec_to_u32_end(input, 46) {
        Some(num) => num,
        None => return None,
    };

    Some(DescriptorGuid {
        data: guid,
        type_hash,
        part_number,
    })
}

const fn dec_to_u32_end(input: &[u8], start: usize) -> Option<u32> {
    let mut result = 0u32;
    let mut i = start;

    while i < input.len() {
        let x = match (input[i] as char).to_digit(10) {
            Some(digit) => digit,
            None => return None,
        };

        result = result.wrapping_mul(10);
        result = result.wrapping_add(x);
        i += 1;
    }

    Some(result)
}

#[cfg(test)]
#[allow(deprecated)]
mod test {

    #[test]
    fn test_guid_qualified_string() {
        const GUID: &str = "40e6ba42-a397-5790-a5c9-a4151fffe1c5_647628d6_420";

        let guid = super::DescriptorGuid::parse_qualified(GUID).unwrap();

        assert_eq!(guid.to_qualified_string(), GUID);
    }

    #[test]
    fn test_guid_string() {
        const GUID: &str = "40e6ba42-a397-5790-a5c9-a4151fffe1c5";

        let guid = super::DescriptorGuid::parse(GUID, 0, 0).unwrap();

        assert_eq!(guid.to_string(), GUID);
    }

}
