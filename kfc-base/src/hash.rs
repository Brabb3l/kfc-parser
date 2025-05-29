pub fn fnv<T: AsRef<[u8]>>(input: T) -> u32 {
    let mut hash = 0x811c9dc5_u32;

    for byte in input.as_ref().iter() {
        hash ^= *byte as u32;
        hash = hash.wrapping_mul(0x1000193);
    }

    hash
}

pub fn fnv_with_seed<T: AsRef<[u8]>>(input: T, seed: u32) -> u32 {
    let mut hash = seed;

    for byte in input.as_ref().iter() {
        hash ^= *byte as u32;
        hash = hash.wrapping_mul(0x1000193);
    }

    hash
}

pub const fn fnv_const(input: &str) -> u32 {
    fnv_const_iter(input.as_bytes(), 0, 0x811c9dc5)
}

const fn fnv_const_iter(input: &[u8], index: usize, hash: u32) -> u32 {
    if index == input.len() {
        hash
    } else {
        fnv_const_iter(input, index + 1, (hash ^ input[index] as u32).wrapping_mul(0x1000193))
    }
}

pub fn crc64<T: AsRef<[u8]>>(input: T) -> u64 {
    let mut output = crc64fast::Digest::new();
    output.write(input.as_ref());
    output.sum64()
}
