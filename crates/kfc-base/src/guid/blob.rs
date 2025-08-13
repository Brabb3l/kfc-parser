use std::fmt::{Debug, Display};
use std::io::{Read, Write};
use std::str::FromStr;

use serde::Deserialize;

use crate::hash::{compute_blob_guid, fnv_bytes};
use crate::{container::StaticHash, Hash32};

use super::DescriptorGuid;

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct BlobGuid {
    data: [u8; 16],
}

impl BlobGuid {

    pub const NONE: Self = Self {
        data: [0; 16],
    };

    #[inline]
    #[must_use]
    pub const fn new(data: [u8; 16]) -> Self {
        Self {
            data
        }
    }

    /// Create a new BlobGuid from a string with following format:
    /// `XXXXXXXX-XXXX-XXXX-XXXX-XXXXXXXXXXXX`
    /// where `X` is a hexadecimal digit.
    ///
    /// If the string is not in the correct format, `None` is returned.
    #[inline]
    #[must_use]
    pub const fn parse(s: &str) -> Option<Self> {
        let data = match str_to_guid(s) {
            Some(data) => data,
            None => return None,
        };

        Some(Self { data })
    }

    #[inline]
    #[must_use]
    pub const fn from_parts(
        size: u32,
        hash0: u32,
        hash1: u32,
        hash2: u32,
    ) -> Self {
        let size = size.to_le_bytes();
        let hash0 = hash0.to_le_bytes();
        let hash1 = hash1.to_le_bytes();
        let hash2 = hash2.to_le_bytes();

        Self {
            data: [
                size[0], size[1], size[2], size[3],
                hash0[0], hash0[1], hash0[2], hash0[3],
                hash1[0], hash1[1], hash1[2], hash1[3],
                hash2[0], hash2[1], hash2[2], hash2[3],
            ]
        }
    }

    /// Generates a new `BlobGuid` from the given data.
    ///
    /// # Panics
    /// If the data is larger than 4294967295 bytes, it will panic.
    #[inline]
    #[must_use]
    pub fn from_data(data: &[u8]) -> Self {
        let data_size = u32::try_from(data.len())
            .expect("data may not be larger than 4294967295 bytes");
        let guid = compute_blob_guid(data, 0);

        Self::from_parts(
            data_size,
            u32::from_le_bytes(guid[4..8].try_into().unwrap()),
            u32::from_le_bytes(guid[8..12].try_into().unwrap()),
            u32::from_le_bytes(guid[12..16].try_into().unwrap()),
        )
    }

    #[inline]
    pub const fn data(&self) -> &[u8; 16] {
        &self.data
    }

    #[inline]
    pub const fn into_data(self) -> [u8; 16] {
        self.data
    }

    #[inline]
    pub const fn into_parts(self) -> (u32, u32, u32, u32) {
        (
            self.size(),
            u32::from_le_bytes([self.data[4], self.data[5], self.data[6], self.data[7]]),
            u32::from_le_bytes([self.data[8], self.data[9], self.data[10], self.data[11]]),
            u32::from_le_bytes([self.data[12], self.data[13], self.data[14], self.data[15]]),
        )
    }

    #[inline]
    pub const fn size(&self) -> u32 {
        u32::from_le_bytes([
            self.data[0],
            self.data[1],
            self.data[2],
            self.data[3],
        ])
    }

    #[inline]
    pub fn is_none(&self) -> bool {
        self.data == [0; 16]
    }

    #[inline]
    #[must_use]
    pub const fn hash32(&self) -> Hash32 {
        fnv_bytes(&self.data)
    }

    #[inline]
    #[must_use]
    pub const fn as_descriptor_guid(&self, type_hash: Hash32, part_number: u32) -> DescriptorGuid {
        DescriptorGuid {
            data: self.data,
            type_hash,
            part_number,
        }
    }

}

impl StaticHash for BlobGuid {

    #[inline]
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
        Ok(Self {
            data: str_to_guid(s)
                .ok_or_else(|| format!("invalid guid string: {s}"))?,
        })
    }
}

impl Debug for BlobGuid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", guid_to_string(&self.data))
    }
}

impl Display for BlobGuid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", guid_to_string(&self.data))
    }
}

impl<'de> Deserialize<'de> for BlobGuid {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Self::parse(&s).ok_or_else(|| {
            serde::de::Error::custom(format!("invalid guid string: {s}"))
        })
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

    #[inline]
    pub fn read<R: Read>(reader: &mut R) -> std::io::Result<Self> {
        let mut data = [0; 16];
        reader.read_exact(&mut data)?;

        Ok(Self {
            data
        })
    }

    #[inline]
    pub fn write<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        writer.write_all(&self.data)
    }

}

const fn str_to_guid(input: &str) -> Option<[u8; 16]> {
    if input.len() < 36 {
        return None;
    }

    let input = input.as_bytes();

    if input[8] != b'-' || input[13] != b'-' || input[18] != b'-' || input[23] != b'-' {
        return None;
    }

    macro_rules! hex_to_bytes {
        ($start:expr, $len:expr) => {
            match hex_to_bytes::<$len>(input, $start) {
                Some(bytes) => bytes,
                None => return None,
            }
        };
    }

    let a = hex_to_bytes!(0, 4);
    let b = hex_to_bytes!(9, 2);
    let c = hex_to_bytes!(14, 2);
    let d = hex_to_bytes!(19, 2);
    let e = hex_to_bytes!(24, 6);

    let mut bytes = [0u8; 16];

    bytes[0] = a[3];
    bytes[1] = a[2];
    bytes[2] = a[1];
    bytes[3] = a[0];

    bytes[4] = b[1];
    bytes[5] = b[0];

    bytes[6] = c[1];
    bytes[7] = c[0];

    bytes[8] = d[0];
    bytes[9] = d[1];

    bytes[10] = e[0];
    bytes[11] = e[1];
    bytes[12] = e[2];
    bytes[13] = e[3];
    bytes[14] = e[4];
    bytes[15] = e[5];

    Some(bytes)
}

pub(super) const fn hex_to_bytes<const N: usize>(input: &[u8], start: usize) -> Option<[u8; N]> {
    let mut bytes = [0u8; N];
    let end = start + N * 2;
    let mut i = start;

    while i < end {
        let high = match (input[i] as char).to_digit(16) {
            Some(d) => d,
            None => return None,
        };

        let low = match (input[i + 1] as char).to_digit(16) {
            Some(d) => d,
            None => return None,
        };

        let value = high << 4 | low;

        bytes[(i - start) >> 1] = value as u8;
        i += 2;
    }

    Some(bytes)
}

fn guid_to_string(data: &[u8; 16]) -> String {
    let a = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
    let b = u16::from_le_bytes([data[4], data[5]]);
    let c = u16::from_le_bytes([data[6], data[7]]);
    let d = u16::from_be_bytes([data[8], data[9]]);
    let e = u32::from_be_bytes([data[10], data[11], data[12], data[13]]);
    let f = u16::from_be_bytes([data[14], data[15]]);

    format!("{a:0>8x}-{b:0>4x}-{c:0>4x}-{d:0>4x}-{e:0>8x}{f:0>4x}")
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_GUID_BYTES: [u8; 16] = [
        0x33, 0x22, 0x11, 0x00,
        0x55, 0x44,
        0x77, 0x66,
        0x88, 0x99,
        0xAA, 0xBB, 0xCC, 0xDD,
        0xEE, 0xFF
    ];

    const TEST_GUID_STR_LOWER: &str = "00112233-4455-6677-8899-aabbccddeeff";
    const TEST_GUID_STR_UPPER: &str = "00112233-4455-6677-8899-AABBCCDDEEFF";

    #[test]
    fn test_string_to_guid() {
        let guid = str_to_guid(TEST_GUID_STR_LOWER)
            .expect("Failed to convert string to GUID");

        assert_eq!(guid, TEST_GUID_BYTES);
    }

    #[test]
    fn test_guid_to_guid_upper() {
        let guid = str_to_guid(TEST_GUID_STR_UPPER)
            .expect("Failed to convert string to GUID");

        assert_eq!(guid, TEST_GUID_BYTES);
    }

    #[test]
    fn test_valid_string_to_guid_trailing() {
        assert!(str_to_guid("12345678-1234-1234-1234-1234567890thisshouldbeignored").is_none());
    }

    #[test]
    fn test_invalid_string_to_guid() {
        assert!(str_to_guid("invalid-guid").is_none());
        assert!(str_to_guid("00112233445566778899aabbccddeeff").is_none());
    }

    #[test]
    fn test_guid_to_string() {
        let guid = [
            0x33, 0x22, 0x11, 0x00,
            0x55, 0x44,
            0x77, 0x66,
            0x88, 0x99,
            0xAA, 0xBB, 0xCC, 0xDD,
            0xEE, 0xFF
        ];
        let guid_str = guid_to_string(&guid);

        assert_eq!(guid_str, TEST_GUID_STR_LOWER);
    }

}
