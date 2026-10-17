mod decode;
mod encode;
mod format;

pub use decode::*;
pub use encode::*;
pub use format::*;

pub fn size_of_format(
    format: PixelFormat,
    width: usize,
    height: usize,
) -> usize {
    let (block_width, block_height) = format.block_extent();
    let width = width.div_ceil(block_width);
    let height = height.div_ceil(block_height);

    width * height * format.block_size()
}

#[cfg(test)]
mod tests {
    use super::*;

    const WIDTH: usize = 3;
    const HEIGHT: usize = 2;

    /// A 3x2 RGBA8 image with distinct values in every channel,
    /// including the extremes 0x00, 0x7F, 0x80 and 0xFF.
    const IMAGE: [u8; WIDTH * HEIGHT * 4] = [
        0x00, 0x7F, 0x80, 0xFF,
        0x01, 0x23, 0x45, 0x67,
        0x89, 0xAB, 0xCD, 0xEF,
        0xFE, 0xDC, 0xBA, 0x98,
        0x76, 0x54, 0x32, 0x10,
        0x0F, 0xF0, 0x3C, 0xC3,
    ];

    /// Every possible byte value, spread across an 8x8 image.
    fn all_values() -> Vec<u8> {
        (0..=255).collect()
    }

    /// Expected RGBA8 output for a format with `channels` channels;
    /// missing color channels decode to 0 and a missing alpha to 255.
    fn expected(src: &[u8], channels: usize) -> Vec<u8> {
        src.chunks_exact(4)
            .flat_map(|texel| {
                let mut texel = [texel[0], texel[1], texel[2], texel[3]];
                texel[channels.min(3)..3].fill(0);
                if channels < 4 {
                    texel[3] = 255;
                }
                texel
            })
            .collect()
    }

    fn round_trip(format: PixelFormat, width: usize, height: usize, src: &[u8]) -> Vec<u8> {
        let mut encoded = vec![0u8; size_of_format(format, width, height)];
        encode(format, width, height, src, &mut encoded).unwrap();

        let mut decoded = vec![0u8; width * height * 4];
        decode(format, width, height, &encoded, &mut decoded).unwrap();

        decoded
    }

    fn assert_round_trip(format: PixelFormat, channels: usize) {
        assert_eq!(
            round_trip(format, WIDTH, HEIGHT, &IMAGE),
            expected(&IMAGE, channels),
            "{format:?}",
        );

        let all = all_values();
        assert_eq!(
            round_trip(format, 8, 8, &all),
            expected(&all, channels),
            "{format:?} (all values)",
        );
    }

    #[test]
    fn round_trip_8_bit() {
        assert_round_trip(PixelFormat::R8_unorm, 1);
        assert_round_trip(PixelFormat::R8G8_snorm, 2);
        assert_round_trip(PixelFormat::B8G8R8_unorm, 3);
        assert_round_trip(PixelFormat::R8G8B8A8_unorm, 4);
        assert_round_trip(PixelFormat::R8G8B8A8_sint, 4);
        assert_round_trip(PixelFormat::B8G8R8A8_unorm, 4);
        assert_round_trip(PixelFormat::A8B8G8R8_snorm_pack32, 4);
    }

    #[test]
    fn round_trip_16_bit() {
        assert_round_trip(PixelFormat::R16_unorm, 1);
        assert_round_trip(PixelFormat::R16G16_sint, 2);
        assert_round_trip(PixelFormat::R16G16B16_uint, 3);
        assert_round_trip(PixelFormat::R16G16B16A16_unorm, 4);
        assert_round_trip(PixelFormat::R16G16B16A16_snorm, 4);
        assert_round_trip(PixelFormat::R16G16B16A16_sfloat, 4);
    }

    #[test]
    fn round_trip_32_bit() {
        assert_round_trip(PixelFormat::R32_uint, 1);
        assert_round_trip(PixelFormat::R32G32_sint, 2);
        assert_round_trip(PixelFormat::R32G32B32_sfloat, 3);
        assert_round_trip(PixelFormat::R32G32B32A32_uint, 4);
        assert_round_trip(PixelFormat::R32G32B32A32_sint, 4);
        assert_round_trip(PixelFormat::R32G32B32A32_sfloat, 4);
    }

    #[test]
    fn round_trip_64_bit() {
        assert_round_trip(PixelFormat::R64_uint, 1);
        assert_round_trip(PixelFormat::R64G64_sint, 2);
        assert_round_trip(PixelFormat::R64G64B64_sfloat, 3);
        assert_round_trip(PixelFormat::R64G64B64A64_uint, 4);
        assert_round_trip(PixelFormat::R64G64B64A64_sint, 4);
        assert_round_trip(PixelFormat::R64G64B64A64_sfloat, 4);
    }

    #[test]
    fn encode_wide_unsigned_is_scaled_little_endian() {
        let mut encoded = [0u8; 8];
        encode(PixelFormat::R16G16B16A16_unorm, 1, 1, &[0x00, 0x12, 0x80, 0xFF], &mut encoded).unwrap();
        assert_eq!(encoded, [0x00, 0x00, 0x12, 0x12, 0x80, 0x80, 0xFF, 0xFF]);

        let mut encoded = [0u8; 4];
        encode(PixelFormat::R32_uint, 1, 1, &[0xAB, 0, 0, 0], &mut encoded).unwrap();
        assert_eq!(u32::from_le_bytes(encoded), 0xABAB_ABAB);
    }

    fn decode_texel(format: PixelFormat, src: &[u8]) -> [u8; 4] {
        let mut dst = [0u8; 4];
        decode(format, 1, 1, src, &mut dst).unwrap();
        dst
    }

    #[test]
    fn decode_b5g6r5_sets_opaque_alpha() {
        // b = 0x1F, g = 0x00, r = 0x00
        let src = 0xF800u16.to_le_bytes();
        assert_eq!(decode_texel(PixelFormat::B5G6R5_unorm_pack16, &src), [0, 0, 255, 255]);
    }

    #[test]
    fn decode_b10g11r11_ufloat() {
        // 1.0 is exponent 15 with a zero mantissa: R and G are 11 bits wide (6 bit mantissa),
        // B is 10 bits wide (5 bit mantissa)
        let one_11 = 15u32 << 6;
        let one_10 = 15u32 << 5;
        let half_11 = 14u32 << 6;

        let src = (one_11 | (half_11 << 11)).to_le_bytes();
        assert_eq!(decode_texel(PixelFormat::B10G11R11_ufloat_pack32, &src), [255, 127, 0, 255]);

        let src = (one_10 << 22).to_le_bytes();
        assert_eq!(decode_texel(PixelFormat::B10G11R11_ufloat_pack32, &src), [0, 0, 255, 255]);
    }

    #[test]
    fn decode_e5b9g9r9_ufloat() {
        // value = mantissa * 2^(exponent - 15 - 9), so with exponent 16 a mantissa of 256 is 1.0
        let src = (256u32 | (128 << 9) | (16 << 27)).to_le_bytes();
        assert_eq!(decode_texel(PixelFormat::E5B9G9R9_ufloat_pack32, &src), [255, 127, 0, 255]);
    }

    #[test]
    fn encode_rejects_undersized_buffers() {
        let format = PixelFormat::R16G16B16A16_unorm;
        let mut dst = vec![0u8; size_of_format(format, WIDTH, HEIGHT)];

        assert!(encode(format, WIDTH, HEIGHT, &IMAGE[..IMAGE.len() - 1], &mut dst).is_err());

        let dst_len = dst.len();
        assert!(encode(format, WIDTH, HEIGHT, &IMAGE, &mut dst[..dst_len - 1]).is_err());
    }

    #[test]
    fn decode_rejects_undersized_buffers() {
        let format = PixelFormat::R16G16B16A16_unorm;
        let src = vec![0u8; size_of_format(format, WIDTH, HEIGHT)];
        let mut dst = vec![0u8; WIDTH * HEIGHT * 4];

        assert!(decode(format, WIDTH, HEIGHT, &src[..src.len() - 1], &mut dst).is_err());

        let dst_len = dst.len();
        assert!(decode(format, WIDTH, HEIGHT, &src, &mut dst[..dst_len - 1]).is_err());
    }
}
