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

