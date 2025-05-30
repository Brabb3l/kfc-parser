#[inline]
pub fn prefix_pattern<const N: usize, const M: usize>(
    pattern: [u8; N],
    value: u8
) -> [u8; M] {
    let mut new_pattern = [0; M];
    new_pattern[0] = value;
    new_pattern[1..M].copy_from_slice(&pattern);
    new_pattern
}
