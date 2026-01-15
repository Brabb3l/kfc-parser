use std::fmt::{Debug, Display};
use std::io::{Read, Write};

use serde::{Deserialize, Serialize};

use crate::guid::Guid;
use crate::hash::fnv_bytes_with_seed;
use crate::{container::StaticHash, io::{ReadExt, WriteExt}, Hash32};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct ResourceId {
    guid: Guid,
    type_hash: Hash32,
    part_index: u32,
    reserved_0: u32,
    reserved_1: u32,
}

impl ResourceId {

    pub const NONE: Self = Self {
        guid: Guid::NONE,
        type_hash: 0,
        part_index: 0,
        reserved_0: 0,
        reserved_1: 0,
    };

    #[inline]
    #[must_use]
    pub const fn new(
        guid: Guid,
        type_hash: Hash32,
        part_index: u32
    ) -> Self {
        Self {
            guid,
            type_hash,
            part_index,
            reserved_0: 0,
            reserved_1: 0,
        }
    }

    /// Create a new `ResourceId` from a string with following format:
    /// `XXXXXXXX-XXXX-XXXX-XXXX-XXXXXXXXXXXX`
    /// where `X` is a hexadecimal digit.
    ///
    /// If the string is not in the correct format, `None` is returned.
    #[inline]
    #[must_use]
    pub const fn parse(
        s: &str,
        type_hash: Hash32,
        part_index: u32
    ) -> Option<Self> {
        let guid = match Guid::parse(s) {
            Some(data) => data,
            None => return None,
        };

        Some(Self::new(
            guid,
            type_hash,
            part_index,
        ))
    }

    #[inline]
    #[must_use]
    pub const fn guid(&self) -> Guid {
        self.guid
    }

    #[inline]
    #[must_use]
    pub const fn type_hash(&self) -> Hash32 {
        self.type_hash
    }

    #[inline]
    #[must_use]
    pub const fn part_index(&self) -> u32 {
        self.part_index
    }

    #[inline]
    pub fn is_none(&self) -> bool {
        self.guid == Guid::NONE
    }

    #[inline]
    #[must_use]
    pub const fn hash32(&self) -> Hash32 {
        self.guid().hash32()
    }

    #[inline]
    #[must_use]
    pub const fn with_guid(&self, guid: Guid) -> Self {
        Self::new(
            guid,
            self.type_hash,
            self.part_index,
        )
    }

    #[inline]
    #[must_use]
    pub const fn with_type_hash(&self, type_hash: Hash32) -> Self {
        Self::new(
            self.guid,
            type_hash,
            self.part_index,
        )
    }

    #[inline]
    #[must_use]
    pub const fn with_part_index(&self, part_index: u32) -> Self {
        Self::new(
            self.guid,
            self.type_hash,
            part_index,
        )
    }

    /// Create a new `ResourceId` from a string with following format:
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

    /// Convert the `ResourceId` to a qualified string with following format:
    /// `XXXXXXXX-XXXX-XXXX-XXXX-XXXXXXXXXXXX_XXXXXXXX_N`
    /// where `X` is a hexadecimal digit and `N` is a 32-bit decimal integer.
    #[inline]
    #[must_use]
    #[deprecated(note = "the format is not very user-friendly, use your own format instead")]
    pub fn to_qualified_string(&self) -> String {
        format!("{}_{:0>8x}_{}", self.guid(), self.type_hash, self.part_index)
    }

}

impl StaticHash for ResourceId {

    #[inline]
    fn static_hash(&self) -> u32 {
        let seed = u32::from_le_bytes(self.guid.data()[0..4].try_into().unwrap());
        let mut rest = [0u8; 8];

        rest[0..4].copy_from_slice(&self.type_hash.to_le_bytes());
        rest[4..8].copy_from_slice(&self.part_index.to_le_bytes());

        fnv_bytes_with_seed(&rest, seed)
    }

}

impl Display for ResourceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.guid(), f)
    }
}

impl ResourceId {

    #[inline]
    pub fn read<R: Read>(reader: &mut R) -> std::io::Result<Self> {
        let guid = Guid::read(reader)?;
        let type_hash = reader.read_u32()?;
        let part_index = reader.read_u32()?;
        reader.padding(4)?; // reserved_0
        reader.padding(4)?; // reserved_1

        Ok(Self::new(
            guid,
            type_hash,
            part_index,
        ))
    }

    #[inline]
    pub fn write<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        self.guid.write(writer)?;
        writer.write_u32(self.type_hash)?;
        writer.write_u32(self.part_index)?;
        writer.padding(4)?; // reserved_0
        writer.padding(4)?; // reserved_1

        Ok(())
    }

}

impl<'de> Deserialize<'de> for ResourceId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;

        #[allow(deprecated)]
        Self::parse_qualified(&s)
            .ok_or_else(|| serde::de::Error::custom("invalid ResourceId"))
    }
}

impl Serialize for ResourceId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        #[allow(deprecated)]
        self.to_qualified_string().serialize(serializer)
    }
}

const fn str_to_qualified_guid(input: &str) -> Option<ResourceId> {
    if input.len() < 47 {
        return None;
    }

    let guid = match Guid::parse(input) {
        Some(guid) => guid,
        None => return None,
    };

    let input = input.as_bytes();

    if input[36] != b'_' || input[45] != b'_' {
        return None;
    }

    let type_hash = match super::base::hex_to_bytes::<4>(input, 37) {
        Some(bytes) => u32::from_be_bytes(bytes),
        None => return None,
    };

    let part_index = match dec_to_u32_end(input, 46) {
        Some(num) => num,
        None => return None,
    };

    Some(ResourceId::new(
        guid,
        type_hash,
        part_index,
    ))
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

        let guid = super::ResourceId::parse_qualified(GUID).unwrap();

        assert_eq!(guid.to_qualified_string(), GUID);
    }

    #[test]
    fn test_guid_string() {
        const GUID: &str = "40e6ba42-a397-5790-a5c9-a4151fffe1c5";

        let guid = super::ResourceId::parse(GUID, 0, 0).unwrap();

        assert_eq!(guid.to_string(), GUID);
    }

}
