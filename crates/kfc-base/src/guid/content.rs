use std::fmt::{Debug, Display};
use std::io::{Read, Write};
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::guid::Guid;
use crate::hash::compute_content_guid;
use crate::{container::StaticHash, Hash32};

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default, Serialize, Deserialize)]
pub struct ContentHash {
    size: u32,
    hash0: u32,
    hash1: u32,
    hash2: u32,
}

impl ContentHash {

    pub const NONE: Self = Self {
        size: 0,
        hash0: 0,
        hash1: 0,
        hash2: 0,
    };

    #[inline]
    #[must_use]
    pub const fn new(
        size: u32,
        hash0: u32,
        hash1: u32,
        hash2: u32,
    ) -> Self {
        Self {
            size,
            hash0,
            hash1,
            hash2,
        }
    }

    /// Generates a new `ContentHash` from the given data.
    ///
    /// # Panics
    /// If the data is larger than 4294967295 bytes, it will panic.
    #[inline]
    #[must_use]
    pub fn from_data(data: &[u8]) -> Self {
        let data_size = u32::try_from(data.len())
            .expect("data may not be larger than 4294967295 bytes");
        let guid = compute_content_guid(data, 0);

        Self::new(
            data_size,
            u32::from_le_bytes(guid[4..8].try_into().unwrap()),
            u32::from_le_bytes(guid[8..12].try_into().unwrap()),
            u32::from_le_bytes(guid[12..16].try_into().unwrap()),
        )
    }

    /// Create a new `ContentHash` from a string with following format:
    /// `XXXXXXXX-XXXX-XXXX-XXXX-XXXXXXXXXXXX`
    /// where `X` is a hexadecimal digit.
    ///
    /// If the string is not in the correct format, `None` is returned.
    #[inline]
    #[must_use]
    pub const fn parse(s: &str) -> Option<Self> {
        match Guid::parse(s) {
            Some(guid) => Some(Self::from_guid(guid)),
            None => None,
        }
    }

    #[inline]
    #[must_use]
    pub const fn size(&self) -> u32 {
        self.size
    }

    #[inline]
    #[must_use]
    pub const fn hash0(&self) -> u32 {
        self.hash0
    }

    #[inline]
    #[must_use]
    pub const fn hash1(&self) -> u32 {
        self.hash1
    }

    #[inline]
    #[must_use]
    pub const fn hash2(&self) -> u32 {
        self.hash2
    }

    #[inline]
    pub fn is_none(&self) -> bool {
        self.size == 0 && self.hash0 == 0 && self.hash1 == 0 && self.hash2 == 0
    }

    #[inline]
    #[must_use]
    pub const fn hash32(&self) -> Hash32 {
        self.into_guid().hash32()
    }

    #[inline]
    #[must_use]
    pub const fn from_guid(guid: Guid) -> Self {
        let data = guid.data();

        let data_size = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
        let hash0 = u32::from_le_bytes([data[4], data[5], data[6], data[7]]);
        let hash1 = u32::from_le_bytes([data[8], data[9], data[10], data[11]]);
        let hash2 = u32::from_le_bytes([data[12], data[13], data[14], data[15]]);

        Self::new(data_size, hash0, hash1, hash2)
    }

    #[inline]
    #[must_use]
    pub const fn into_guid(&self) -> Guid {
        let size = self.size.to_le_bytes();
        let hash0 = self.hash0.to_le_bytes();
        let hash1 = self.hash1.to_le_bytes();
        let hash2 = self.hash2.to_le_bytes();

        Guid::new([
            size[0], size[1], size[2], size[3],
            hash0[0], hash0[1], hash0[2], hash0[3],
            hash1[0], hash1[1], hash1[2], hash1[3],
            hash2[0], hash2[1], hash2[2], hash2[3],
        ])
    }

}

impl StaticHash for ContentHash {

    #[inline]
    fn static_hash(&self) -> u32 {
        self.hash0
    }

}

impl FromStr for ContentHash {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Guid::from_str(s).map(Into::into)
    }
}

impl Debug for ContentHash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&Guid::from(*self), f)
    }
}

impl Display for ContentHash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&Guid::from(*self), f)
    }
}

impl ContentHash {

    #[inline]
    pub fn read<R: Read>(reader: &mut R) -> std::io::Result<Self> {
        Guid::read(reader).map(Into::into)
    }

    #[inline]
    pub fn write<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        Guid::from(*self).write(writer)
    }

}

impl From<[u8; 16]> for ContentHash {
    fn from(data: [u8; 16]) -> Self {
        Guid::new(data).into()
    }
}

impl From<&[u8; 16]> for ContentHash {
    fn from(data: &[u8; 16]) -> Self {
        Guid::new(*data).into()
    }
}

impl From<Guid> for ContentHash {
    fn from(guid: Guid) -> Self {
        Self::from_guid(guid)
    }
}

impl From<ContentHash> for Guid {
    fn from(guid: ContentHash) -> Self {
        guid.into_guid()
    }
}
