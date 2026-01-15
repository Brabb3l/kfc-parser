const INITIAL_STATE: u32 = 0x811c9dc5;
const PRIME: u32 = 0x1000193;

#[inline]
#[must_use]
pub const fn fnv(input: &str) -> u32 {
    fnv_bytes(input.as_bytes())
}

#[must_use]
pub const fn fnv_bytes(input: &[u8]) -> u32 {
    let mut hash = INITIAL_STATE;
    let mut i = 0;

    while i < input.len() {
        hash ^= input[i] as u32;
        hash = hash.wrapping_mul(PRIME);
        i += 1;
    }

    hash
}

#[inline]
#[must_use]
pub const fn fnv_with_seed(input: &str, seed: u32) -> u32 {
    fnv_bytes_with_seed(input.as_bytes(), seed)
}

#[must_use]
pub const fn fnv_bytes_with_seed(input: &[u8], seed: u32) -> u32 {
    let mut hash = seed;
    let mut i = 0;

    while i < input.len() {
        hash ^= input[i] as u32;
        hash = hash.wrapping_mul(PRIME);
        i += 1;
    }

    hash
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fnv() {
        assert_eq!(fnv("hello"), 0x4f9f2cab);
        assert_eq!(fnv("world"), 0x37a3e893);
    }

    #[test]
    fn test_fnv_with_seed() {
        assert_eq!(fnv_with_seed("hello", 0x12345678), 0x66ce6340);
        assert_eq!(fnv_with_seed("world", 0x12345678), 0x570c34cc);
    }

}
