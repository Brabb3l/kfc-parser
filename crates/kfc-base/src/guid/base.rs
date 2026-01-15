use std::fmt::{Debug, Display};
use std::io::{Read, Write};
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::hash::fnv_bytes;
use crate::Hash32;

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
#[repr(align(4))]
pub struct Guid([u8; 16]);

impl Guid {

    pub const NONE: Self = Self([0; 16]);

    #[inline]
    #[must_use]
    pub const fn new(data: [u8; 16]) -> Self {
        Self(data)
    }

    /// Create a new `Guid` from a string with following format:
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

        Some(Self(data))
    }

    #[inline]
    pub const fn data(&self) -> &[u8; 16] {
        &self.0
    }

    #[inline]
    pub fn is_none(&self) -> bool {
        self.0 == [0; 16]
    }

    #[inline]
    #[must_use]
    pub const fn hash32(&self) -> Hash32 {
        fnv_bytes(&self.0)
    }

}

impl FromStr for Guid {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(str_to_guid(s)
            .ok_or_else(|| format!("invalid guid string: {s}"))?))
    }
}

impl Debug for Guid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", guid_to_string(&self.0))
    }
}

impl Display for Guid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", guid_to_string(&self.0))
    }
}

impl<'de> Deserialize<'de> for Guid {
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

impl Serialize for Guid {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.to_string().serialize(serializer)
    }
}

impl Guid {

    #[inline]
    pub fn read<R: Read>(reader: &mut R) -> std::io::Result<Self> {
        let mut data = [0; 16];
        reader.read_exact(&mut data)?;

        Ok(Self(data))
    }

    #[inline]
    pub fn write<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        writer.write_all(&self.0)
    }

}

impl From<[u8; 16]> for Guid {
    fn from(data: [u8; 16]) -> Self {
        Self::new(data)
    }
}

impl From<&[u8; 16]> for Guid {
    fn from(data: &[u8; 16]) -> Self {
        Self::new(*data)
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
