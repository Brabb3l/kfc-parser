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
        fnv_const_iter(
            input,
            index + 1,
            (hash ^ input[index] as u32).wrapping_mul(0x1000193)
        )
    }
}

const INITIAL_STATE_1: [u8; 16] = [
    0x00, 0x01, 0x02, 0x03,
    0x04, 0x05, 0x06, 0x07,
    0x08, 0x09, 0x0A, 0x0B,
    0x0C, 0x0D, 0x0E, 0x0F,
];

const INITIAL_STATE_2: [u8; 16] = [
    0x10, 0x11, 0x12, 0x13,
    0x14, 0x15, 0x16, 0x17,
    0x18, 0x19, 0x1A, 0x1B,
    0x1C, 0x1D, 0x1E, 0x1F,
];

const INITIAL_STATE_3: [u8; 16] = [
    0x20, 0x21, 0x22, 0x23,
    0x24, 0x25, 0x26, 0x27,
    0x28, 0x29, 0x2A, 0x2B,
    0x2C, 0x2D, 0x2E, 0x2F,
];

const INITIAL_STATE_4: [u8; 16] = [
    0x30, 0x31, 0x32, 0x33,
    0x34, 0x35, 0x36, 0x37,
    0x38, 0x39, 0x3A, 0x3B,
    0x3C, 0x3D, 0x3E, 0x3F,
];

#[inline]
fn aesdec(state: &mut [u8; 16], key: &[u8; 16]) {
    aes::hazmat::equiv_inv_cipher_round(state.into(), key.into());
}

#[inline]
fn get_block(data: &[u8], offset: usize) -> &[u8; 16] {
    data[offset..offset + 16].try_into().unwrap()
}

pub fn compute_blob_guid(
    data: &[u8],
    seed: u64,
) -> [u8; 16] {
    let mut state1 = INITIAL_STATE_1;
    let mut state2 = INITIAL_STATE_2;
    let mut state3 = INITIAL_STATE_3;
    let mut state4 = INITIAL_STATE_4;

    let mut offset = 0;

    // Process in blocks of 64 bytes
    while offset + 64 <= data.len() {
        aesdec(&mut state1, get_block(data, offset));
        aesdec(&mut state2, get_block(data, offset + 16));
        aesdec(&mut state3, get_block(data, offset + 32));
        aesdec(&mut state4, get_block(data, offset + 48));
        offset += 64;
    }

    // Process remaining in blocks of 16 bytes
    if offset + 16 <= data.len() {
        aesdec(&mut state1, get_block(data, offset));
        offset += 16;
    }

    if offset + 16 <= data.len() {
        aesdec(&mut state2, get_block(data, offset));
        offset += 16;
    }

    if offset + 16 <= data.len() {
        aesdec(&mut state3, get_block(data, offset));
        offset += 16;
    }

    // Process all the other bytes that are <16 bytes in a block.
    if offset < data.len() {
        let mut tmp = [0u8; 16];
        tmp[0..(data.len() - offset)].copy_from_slice(&data[offset..data.len()]);
        aesdec(&mut state4, &tmp);
    }

    // Finalize
    let mut seed_state = [0u8; 16];
    seed_state[0..8].copy_from_slice(&u64::to_le_bytes(seed.wrapping_sub(data.len() as u64)));
    seed_state[8..16].copy_from_slice(&u64::to_le_bytes(seed.wrapping_add(data.len() as u64 + 1)));

    aesdec(&mut state4, &seed_state);
    aesdec(&mut state3, &seed_state);
    aesdec(&mut state2, &seed_state);
    aesdec(&mut state1, &seed_state);
    aesdec(&mut state3, &state4);
    aesdec(&mut state1, &state2);
    aesdec(&mut state3, &seed_state);
    aesdec(&mut state1, &state3);
    aesdec(&mut state1, &seed_state);

    state1
}
