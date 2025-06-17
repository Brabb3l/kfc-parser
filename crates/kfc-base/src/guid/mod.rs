mod blob;
mod descriptor;

pub use blob::*;
pub use descriptor::*;

fn string_to_guid(s: &str) -> Option<[u8; 16]> {
    if s.len() != 36 {
        return None;
    }

    if !is_hex_slice(&s[0..8]) ||
        !is_separator(s[8..].chars().next().unwrap()) ||
        !is_hex_slice(&s[9..13]) ||
        !is_separator(s[13..].chars().next().unwrap()) ||
        !is_hex_slice(&s[14..18]) ||
        !is_separator(s[18..].chars().next().unwrap()) ||
        !is_hex_slice(&s[19..23]) ||
        !is_separator(s[23..].chars().next().unwrap()) ||
        !is_hex_slice(&s[24..36])
    {
        return None;
    }

    let mut data = [0; 16];

    data[0..4].copy_from_slice(&u32::from_str_radix(&s[0..8], 16).unwrap().to_le_bytes());
    data[4..6].copy_from_slice(&u16::from_str_radix(&s[9..13], 16).unwrap().to_le_bytes());
    data[6..8].copy_from_slice(&u16::from_str_radix(&s[14..18], 16).unwrap().to_le_bytes());
    data[8..10].copy_from_slice(&u16::from_str_radix(&s[19..23], 16).unwrap().to_be_bytes());
    data[10..14].copy_from_slice(&u32::from_str_radix(&s[24..32], 16).unwrap().to_be_bytes());
    data[14..16].copy_from_slice(&u16::from_str_radix(&s[32..36], 16).unwrap().to_be_bytes());

    Some(data)
}

fn guid_to_string(data: &[u8; 16]) -> String {
    let part_0 = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
    let part_1 = u16::from_le_bytes([data[4], data[5]]);
    let part_2 = u16::from_le_bytes([data[6], data[7]]);
    let part_3 = u16::from_be_bytes([data[8], data[9]]);
    let part_4 = u32::from_be_bytes([data[10], data[11], data[12], data[13]]);
    let part_5 = u16::from_be_bytes([data[14], data[15]]);

    format!("{:0>8x}-{:0>4x}-{:0>4x}-{:0>4x}-{:0>8x}{:0>4x}", part_0, part_1, part_2, part_3, part_4, part_5)
}

#[inline]
fn is_hex_slice(s: &str) -> bool {
    s.chars().all(|c| c.is_ascii_hexdigit())
}

#[inline]
fn is_separator(c: char) -> bool {
    c == '-'
}

#[inline]
fn is_section_separator(c: char) -> bool {
    c == '_'
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
        let guid = string_to_guid(TEST_GUID_STR_LOWER)
            .expect("Failed to convert string to GUID");

        assert_eq!(guid, TEST_GUID_BYTES);
    }
    #[test]
    fn test_guid_to_guid_upper() {
        let guid = string_to_guid(TEST_GUID_STR_UPPER)
            .expect("Failed to convert string to GUID");

        assert_eq!(guid, TEST_GUID_BYTES);
    }

    #[test]
    fn test_invalid_string_to_guid() {
        assert!(string_to_guid("invalid-guid").is_none());
        assert!(string_to_guid("12345678-1234-1234-1234-1234567890a").is_none());
        assert!(string_to_guid("00112233-4455-6677-8899-aabbccddeeffg").is_none());
        assert!(string_to_guid("00112233-4455-6677-8899-aabbccddeeff-").is_none());
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
