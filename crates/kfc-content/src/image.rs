use bcdec_rs::bc6h_float;
use half::f16;
use serde::{Deserialize, Serialize};
use texture2ddecoder::{decode_bc1, decode_bc3, decode_bc4, decode_bc5, decode_bc7};

#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PixelFormat {
    None,
    R4G4_unorm_pack8,
    R4G4B4A4_unorm_pack16,
    B4G4R4A4_unorm_pack16,
    R5G6B5_unorm_pack16,
    B5G6R5_unorm_pack16,
    R5G5B5A1_unorm_pack16,
    B5G5R5A1_unorm_pack16,
    A1R5G5B5_unorm_pack16,
    R8_unorm,
    R8_snorm,
    R8_uscaled,
    R8_sscaled,
    R8_uint,
    R8_sint,
    R8_srgb,
    R8G8_unorm,
    R8G8_snorm,
    R8G8_uscaled,
    R8G8_sscaled,
    R8G8_uint,
    R8G8_sint,
    R8G8_srgb,
    R8G8B8_unorm,
    R8G8B8_snorm,
    R8G8B8_uscaled,
    R8G8B8_sscaled,
    R8G8B8_uint,
    R8G8B8_sint,
    R8G8B8_srgb,
    B8G8R8_unorm,
    B8G8R8_snorm,
    B8G8R8_uscaled,
    B8G8R8_sscaled,
    B8G8R8_uint,
    B8G8R8_sint,
    B8G8R8_srgb,
    R8G8B8A8_unorm,
    R8G8B8A8_snorm,
    R8G8B8A8_uscaled,
    R8G8B8A8_sscaled,
    R8G8B8A8_uint,
    R8G8B8A8_sint,
    R8G8B8A8_srgb,
    B8G8R8A8_unorm,
    B8G8R8A8_snorm,
    B8G8R8A8_uscaled,
    B8G8R8A8_sscaled,
    B8G8R8A8_uint,
    B8G8R8A8_sint,
    B8G8R8A8_srgb,
    A8B8G8R8_unorm_pack32,
    A8B8G8R8_snorm_pack32,
    A8B8G8R8_uscaled_pack32,
    A8B8G8R8_sscaled_pack32,
    A8B8G8R8_uint_pack32,
    A8B8G8R8_sint_pack32,
    A8B8G8R8_srgb_pack32,
    A2R10G10B10_unorm_pack32,
    A2R10G10B10_snorm_pack32,
    A2R10G10B10_uscaled_pack32,
    A2R10G10B10_sscaled_pack32,
    A2R10G10B10_uint_pack32,
    A2R10G10B10_sint_pack32,
    A2B10G10R10_unorm_pack32,
    A2B10G10R10_snorm_pack32,
    A2B10G10R10_uscaled_pack32,
    A2B10G10R10_sscaled_pack32,
    A2B10G10R10_uint_pack32,
    A2B10G10R10_sint_pack32,
    R16_unorm,
    R16_snorm,
    R16_uscaled,
    R16_sscaled,
    R16_uint,
    R16_sint,
    R16_sfloat,
    R16G16_unorm,
    R16G16_snorm,
    R16G16_uscaled,
    R16G16_sscaled,
    R16G16_uint,
    R16G16_sint,
    R16G16_sfloat,
    R16G16B16_unorm,
    R16G16B16_snorm,
    R16G16B16_uscaled,
    R16G16B16_sscaled,
    R16G16B16_uint,
    R16G16B16_sint,
    R16G16B16_sfloat,
    R16G16B16A16_unorm,
    R16G16B16A16_snorm,
    R16G16B16A16_uscaled,
    R16G16B16A16_sscaled,
    R16G16B16A16_uint,
    R16G16B16A16_sint,
    R16G16B16A16_sfloat,
    R32_uint,
    R32_sint,
    R32_sfloat,
    R32G32_uint,
    R32G32_sint,
    R32G32_sfloat,
    R32G32B32_uint,
    R32G32B32_sint,
    R32G32B32_sfloat,
    R32G32B32A32_uint,
    R32G32B32A32_sint,
    R32G32B32A32_sfloat,
    R64_uint,
    R64_sint,
    R64_sfloat,
    R64G64_uint,
    R64G64_sint,
    R64G64_sfloat,
    R64G64B64_uint,
    R64G64B64_sint,
    R64G64B64_sfloat,
    R64G64B64A64_uint,
    R64G64B64A64_sint,
    R64G64B64A64_sfloat,
    B10G11R11_ufloat_pack32,
    E5B9G9R9_ufloat_pack32,
    D16_unorm,
    X8_D24_unorm_pack32,
    D32_sfloat,
    S8_uint,
    D16_unorm_S8_uint,
    D24_unorm_S8_uint,
    D32_sfloat_S8_uint,
    BC1_RGB_unorm_block,
    BC1_RGB_srgb_block,
    BC1_RGBA_unorm_block,
    BC1_RGBA_srgb_block,
    BC2_unorm_block,
    BC2_srgb_block,
    BC3_unorm_block,
    BC3_srgb_block,
    BC4_unorm_block,
    BC4_snorm_block,
    BC5_unorm_block,
    BC5_snorm_block,
    BC6H_ufloat_block,
    BC6H_sfloat_block,
    BC7_unorm_block,
    BC7_srgb_block,
}

macro_rules! map_component {
    (U8, $value:expr, $index:expr) => { $value[$index] };
    (S8, $value:expr, $index:expr) => { ($value[$index] & 0x7F) | (!$value[$index] & 0x80) };
    (U16, $value:expr, $index:expr) => { map_component!(U8, $value, $index + 1) };
    (S16, $value:expr, $index:expr) => { map_component!(S8, $value, $index + 1) };
    (U32, $value:expr, $index:expr) => { map_component!(U8, $value, $index + 3) };
    (S32, $value:expr, $index:expr) => { map_component!(S8, $value, $index + 3) };
    (U64, $value:expr, $index:expr) => { map_component!(U8, $value, $index + 7) };
    (S64, $value:expr, $index:expr) => { map_component!(S8, $value, $index + 7) };
    (F16, $value:expr, $index:expr) => {
        (f16::from_le_bytes($value[$index..$index + 2].try_into()?).to_f32()) * 255.0
    };
    (F32, $value:expr, $index:expr) => {
        {
            let data = u32::from_le_bytes($value[$index..$index + 4].try_into()?);
            f32::from_bits(data) * (1 << 8)
        }
    };
    (F64, $value:expr, $index:expr) => {
        {
            let data = u64::from_le_bytes($value[$index..$index + 8].try_into()?);
            f64::from_bits(data) * (1 << 8)
        }
    };
}

macro_rules! convert_pixel {
    (R, $ty:ident, $pixel:ident, $size:expr) => {
        map_component!($ty, $pixel, 0) as u32
    };
    (RG, $ty:ident, $pixel:ident, $size:expr) => {
        {
            let step = $size / 2;
            let r = map_component!($ty, $pixel, step * 0) as u32;
            let g = map_component!($ty, $pixel, step * 1) as u32;

            g << 8 | r
        }
    };
    (RGB, $ty:ident, $pixel:ident, $size:expr) => {
        {
            let step = $size / 3;
            let r = map_component!($ty, $pixel, step * 0) as u32;
            let g = map_component!($ty, $pixel, step * 1) as u32;
            let b = map_component!($ty, $pixel, step * 2) as u32;

            b << 16 | g << 8 | r
        }
    };
    (BGR, $ty:ident, $pixel:ident, $size:expr) => {
        {
            let step = $size / 3;
            let r = map_component!($ty, $pixel, step * 2) as u32;
            let g = map_component!($ty, $pixel, step * 1) as u32;
            let b = map_component!($ty, $pixel, step * 0) as u32;

            b << 16 | g << 8 | r
        }
    };
    (RGBA, $ty:ident, $pixel:ident, $size:expr) => {
        {
            let step = $size / 4;
            let r = map_component!($ty, $pixel, step * 0) as u32;
            let g = map_component!($ty, $pixel, step * 1) as u32;
            let b = map_component!($ty, $pixel, step * 2) as u32;
            let a = map_component!($ty, $pixel, step * 3) as u32;

            a << 24 | b << 16 | g << 8 | r
        }
    };
    (BGRA, $ty:ident, $pixel:ident, $size:expr) => {
        {
            let step = $size / 4;
            let r = map_component!($ty, $pixel, step * 2) as u32;
            let g = map_component!($ty, $pixel, step * 1) as u32;
            let b = map_component!($ty, $pixel, step * 0) as u32;
            let a = map_component!($ty, $pixel, step * 3) as u32;

            a << 24 | b << 16 | g << 8 | r
        }
    };
    (ABGR, $ty:ident, $pixel:ident, $size:expr) => {
        {
            let step = $size / 4;
            let r = map_component!($ty, $pixel, step * 3) as u32;
            let g = map_component!($ty, $pixel, step * 2) as u32;
            let b = map_component!($ty, $pixel, step * 1) as u32;
            let a = map_component!($ty, $pixel, step * 0) as u32;

            a << 24 | b << 16 | g << 8 | r
        }
    };
}

macro_rules! convert_image {
    ($reader:expr, $width:expr, $height:expr, $pixels:expr, $format:ident, $ty:ident, $size:expr) => {
        for (i, pixel) in (&$reader[..$width * $height * $size]).chunks($size).enumerate() {
            $pixels[i] = convert_pixel!($format, $ty, pixel, $size);
        }
    };
}

pub fn deserialize_image(
    format: PixelFormat,
    width: usize,
    height: usize,
    reader: &[u8],
) -> anyhow::Result<Vec<u32>> {
    let mut pixels = vec![0; width * height];

    match format {
        PixelFormat::R8_unorm |
        PixelFormat::R8_uscaled |
        PixelFormat::R8_uint |
        PixelFormat::R8_srgb => convert_image!(reader, width, height, pixels, R, U8, 1),
        PixelFormat::R8_snorm |
        PixelFormat::R8_sscaled |
        PixelFormat::R8_sint => convert_image!(reader, width, height, pixels, R, S8, 1),

        PixelFormat::R8G8_unorm |
        PixelFormat::R8G8_uscaled |
        PixelFormat::R8G8_uint |
        PixelFormat::R8G8_srgb => convert_image!(reader, width, height, pixels, RG, U8, 2),
        PixelFormat::R8G8_snorm |
        PixelFormat::R8G8_sscaled |
        PixelFormat::R8G8_sint => convert_image!(reader, width, height, pixels, RG, S8, 2),

        PixelFormat::R8G8B8_unorm |
        PixelFormat::R8G8B8_uscaled |
        PixelFormat::R8G8B8_uint |
        PixelFormat::R8G8B8_srgb => convert_image!(reader, width, height, pixels, RGB, U8, 3),
        PixelFormat::R8G8B8_snorm |
        PixelFormat::R8G8B8_sscaled |
        PixelFormat::R8G8B8_sint => convert_image!(reader, width, height, pixels, RGB, S8, 3),

        PixelFormat::B8G8R8_unorm |
        PixelFormat::B8G8R8_uscaled |
        PixelFormat::B8G8R8_uint |
        PixelFormat::B8G8R8_srgb => convert_image!(reader, width, height, pixels, BGR, U8, 3),
        PixelFormat::B8G8R8_snorm |
        PixelFormat::B8G8R8_sscaled |
        PixelFormat::B8G8R8_sint => convert_image!(reader, width, height, pixels, BGR, S8, 3),

        PixelFormat::R8G8B8A8_unorm |
        PixelFormat::R8G8B8A8_uscaled |
        PixelFormat::R8G8B8A8_uint |
        PixelFormat::R8G8B8A8_srgb => convert_image!(reader, width, height, pixels, RGBA, U8, 4),
        PixelFormat::R8G8B8A8_snorm |
        PixelFormat::R8G8B8A8_sscaled |
        PixelFormat::R8G8B8A8_sint => convert_image!(reader, width, height, pixels, RGBA, S8, 4),

        PixelFormat::B8G8R8A8_unorm |
        PixelFormat::B8G8R8A8_uscaled |
        PixelFormat::B8G8R8A8_uint |
        PixelFormat::B8G8R8A8_srgb => convert_image!(reader, width, height, pixels, BGRA, U8, 4),
        PixelFormat::B8G8R8A8_snorm |
        PixelFormat::B8G8R8A8_sscaled |
        PixelFormat::B8G8R8A8_sint => convert_image!(reader, width, height, pixels, BGRA, S8, 4),

        PixelFormat::A8B8G8R8_unorm_pack32 |
        PixelFormat::A8B8G8R8_uscaled_pack32 |
        PixelFormat::A8B8G8R8_uint_pack32 |
        PixelFormat::A8B8G8R8_srgb_pack32 => convert_image!(reader, width, height, pixels, ABGR, U8, 4),
        PixelFormat::A8B8G8R8_snorm_pack32 |
        PixelFormat::A8B8G8R8_sscaled_pack32 |
        PixelFormat::A8B8G8R8_sint_pack32 => convert_image!(reader, width, height, pixels, ABGR, S8, 4),

        // PixelFormat::A2R10G10B10_unorm_pack32 |
        // PixelFormat::A2R10G10B10_snorm_pack32 |
        // PixelFormat::A2R10G10B10_uscaled_pack32 |
        // PixelFormat::A2R10G10B10_sscaled_pack32 |
        // PixelFormat::A2R10G10B10_uint_pack32 |
        // PixelFormat::A2R10G10B10_sint_pack32 => {
        //     let reader = &reader[..width * height * 4];
        //     for (i, pixel) in reader.chunks(4).enumerate() {
        //         let data = u32::from_be_bytes(pixel.try_into()?);
        //
        //         let a = (data >> 30) & 0x3;
        //         let b = (data >> 20) & 0x3FF;
        //         let g = (data >> 10) & 0x3FF;
        //         let r = data & 0x3FF;
        //
        //         let a = (a * 0xFF) / 0x3;
        //         let b = (b * 0xFF) / 0x3FF;
        //         let g = (g * 0xFF) / 0x3FF;
        //         let r = (r * 0xFF) / 0x3FF;
        //
        //         pixels[i] = a << 24 | b << 16 | g << 8 | r;
        //     }
        // }

        // PixelFormat::A2B10G10R10_unorm_pack32 |
        // PixelFormat::A2B10G10R10_snorm_pack32 |
        // PixelFormat::A2B10G10R10_uscaled_pack32 |
        // PixelFormat::A2B10G10R10_sscaled_pack32 |
        // PixelFormat::A2B10G10R10_uint_pack32 |
        // PixelFormat::A2B10G10R10_sint_pack32 => {
        //     let reader = &reader[..width * height * 4];
        //     for (i, pixel) in reader.chunks(4).enumerate() {
        //         let data = u32::from_be_bytes(pixel.try_into()?);
        //
        //         let a = (data >> 30) & 0x3;
        //         let r = (data >> 20) & 0x3FF;
        //         let g = (data >> 10) & 0x3FF;
        //         let b = data & 0x3FF;
        //
        //         let a = (a * 0xFF) / 0x3;
        //         let r = (r * 0xFF) / 0x3FF;
        //         let g = (g * 0xFF) / 0x3FF;
        //         let b = (b * 0xFF) / 0x3FF;
        //
        //         pixels[i] = a << 24 | b << 16 | g << 8 | r;
        //     }
        // }

        PixelFormat::R16_unorm |
        PixelFormat::R16_uscaled |
        PixelFormat::R16_uint => convert_image!(reader, width, height, pixels, R, U16, 2),
        PixelFormat::R16_snorm |
        PixelFormat::R16_sscaled |
        PixelFormat::R16_sint => convert_image!(reader, width, height, pixels, R, S16, 2),
        PixelFormat::R16_sfloat => convert_image!(reader, width, height, pixels, R, F16, 2),

        PixelFormat::R16G16_unorm |
        PixelFormat::R16G16_uscaled |
        PixelFormat::R16G16_uint => convert_image!(reader, width, height, pixels, RG, U16, 4),
        PixelFormat::R16G16_snorm |
        PixelFormat::R16G16_sscaled |
        PixelFormat::R16G16_sint => convert_image!(reader, width, height, pixels, RG, S16, 4),
        PixelFormat::R16G16_sfloat => convert_image!(reader, width, height, pixels, RG, F16, 4),

        PixelFormat::R16G16B16_unorm |
        PixelFormat::R16G16B16_uscaled |
        PixelFormat::R16G16B16_uint => convert_image!(reader, width, height, pixels, RGB, U16, 6),
        PixelFormat::R16G16B16_snorm |
        PixelFormat::R16G16B16_sscaled |
        PixelFormat::R16G16B16_sint => convert_image!(reader, width, height, pixels, RGB, S16, 6),
        PixelFormat::R16G16B16_sfloat => convert_image!(reader, width, height, pixels, RGB, F16, 6),

        PixelFormat::R16G16B16A16_unorm |
        PixelFormat::R16G16B16A16_uscaled |
        PixelFormat::R16G16B16A16_uint => convert_image!(reader, width, height, pixels, RGBA, U16, 8),
        PixelFormat::R16G16B16A16_snorm |
        PixelFormat::R16G16B16A16_sscaled |
        PixelFormat::R16G16B16A16_sint => convert_image!(reader, width, height, pixels, RGBA, S16, 8),
        PixelFormat::R16G16B16A16_sfloat => convert_image!(reader, width, height, pixels, RGBA, F16, 8),

        // PixelFormat::R32_uint |
        // PixelFormat::R32_sint => {
        //     let reader = &reader[..width * height * 4];
        //     for (i, pixel) in reader.chunks(4).enumerate() {
        //         let data = u32::from_le_bytes(pixel.try_into()?) >> 24;
        //         pixels[i] = data;
        //     }
        // }
        //
        // PixelFormat::R32G32_uint |
        // PixelFormat::R32G32_sint => {
        //     let reader = &reader[..width * height * 8];
        //     for (i, pixel) in reader.chunks(8).enumerate() {
        //         let r = u32::from_le_bytes(pixel[0..4].try_into()?) >> 24;
        //         let g = u32::from_le_bytes(pixel[4..8].try_into()?) >> 24;
        //
        //         pixels[i] = g << 8 | r;
        //     }
        // }
        //
        // PixelFormat::R32G32B32_uint |
        // PixelFormat::R32G32B32_sint => {
        //     let reader = &reader[..width * height * 12];
        //     for (i, pixel) in reader.chunks(12).enumerate() {
        //         let r = u32::from_le_bytes(pixel[0..4].try_into()?) >> 24;
        //         let g = u32::from_le_bytes(pixel[4..8].try_into()?) >> 24;
        //         let b = u32::from_le_bytes(pixel[8..12].try_into()?) >> 24;
        //
        //         pixels[i] = b << 16 | g << 8 | r;
        //     }
        // }
        //
        // PixelFormat::R32G32B32A32_uint |
        // PixelFormat::R32G32B32A32_sint => {
        //     let reader = &reader[..width * height * 16];
        //     for (i, pixel) in reader.chunks(16).enumerate() {
        //         let r = u32::from_le_bytes(pixel[0..4].try_into()?) >> 24;
        //         let g = u32::from_le_bytes(pixel[4..8].try_into()?) >> 24;
        //         let b = u32::from_le_bytes(pixel[8..12].try_into()?) >> 24;
        //         let a = u32::from_le_bytes(pixel[12..16].try_into()?) >> 24;
        //
        //         pixels[i] = a << 24 | b << 16 | g << 8 | r;
        //     }
        // }
        //
        // PixelFormat::R64_uint |
        // PixelFormat::R64_sint => {
        //     let reader = &reader[..width * height * 8];
        //     for (i, pixel) in reader.chunks(8).enumerate() {
        //         let data = u64::from_le_bytes(pixel.try_into()?) >> 56;
        //         pixels[i] = data as u32;
        //     }
        // }
        //
        // PixelFormat::R64G64_uint |
        // PixelFormat::R64G64_sint => {
        //     let reader = &reader[..width * height * 16];
        //     for (i, pixel) in reader.chunks(16).enumerate() {
        //         let r = u64::from_le_bytes(pixel[0..8].try_into()?) >> 56;
        //         let g = u64::from_le_bytes(pixel[8..16].try_into()?) >> 56;
        //
        //         pixels[i] = (g << 8 | r) as u32;
        //     }
        // }
        //
        // PixelFormat::R64G64B64_uint |
        // PixelFormat::R64G64B64_sint => {
        //     let reader = &reader[..width * height * 24];
        //     for (i, pixel) in reader.chunks(24).enumerate() {
        //         let r = u64::from_le_bytes(pixel[0..8].try_into()?) >> 56;
        //         let g = u64::from_le_bytes(pixel[8..16].try_into()?) >> 56;
        //         let b = u64::from_le_bytes(pixel[16..24].try_into()?) >> 56;
        //
        //         pixels[i] = (b << 16 | g << 8 | r) as u32;
        //     }
        // }
        //
        // PixelFormat::R64G64B64A64_uint |
        // PixelFormat::R64G64B64A64_sint => {
        //     let reader = &reader[..width * height * 32];
        //     for (i, pixel) in reader.chunks(32).enumerate() {
        //         let r = u64::from_le_bytes(pixel[0..8].try_into()?) >> 56;
        //         let g = u64::from_le_bytes(pixel[8..16].try_into()?) >> 56;
        //         let b = u64::from_le_bytes(pixel[16..24].try_into()?) >> 56;
        //         let a = u64::from_le_bytes(pixel[24..32].try_into()?) >> 56;
        //
        //         pixels[i] = (a << 24 | b << 16 | g << 8 | r) as u32;
        //     }
        // }

        // TODO: sfloat of R16, R16G16, R16G16B16, R16G16B16A16, R32, R32G32, R32G32B32, R32G32B32A32, R64, R64G64, R64G64B64, R64G64B64A64

        PixelFormat::BC1_RGB_unorm_block => decode_bc1(reader, width, height, &mut pixels).unwrap(),
        PixelFormat::BC1_RGB_srgb_block => decode_bc1(reader, width, height, &mut pixels).unwrap(),
        PixelFormat::BC1_RGBA_unorm_block => decode_bc1(reader, width, height, &mut pixels).unwrap(),
        PixelFormat::BC1_RGBA_srgb_block => decode_bc1(reader, width, height, &mut pixels).unwrap(),
        PixelFormat::BC2_unorm_block => panic!("BC2_unorm_block is not supported"),
        PixelFormat::BC2_srgb_block => panic!("BC2_srgb_block is not supported"),
        PixelFormat::BC3_unorm_block => decode_bc3(reader, width, height, &mut pixels).unwrap(),
        PixelFormat::BC3_srgb_block => decode_bc3(reader, width, height, &mut pixels).unwrap(),
        PixelFormat::BC4_unorm_block => decode_bc4(reader, width, height, &mut pixels).unwrap(),
        PixelFormat::BC4_snorm_block => decode_bc4(reader, width, height, &mut pixels).unwrap(),
        PixelFormat::BC5_unorm_block => decode_bc5(reader, width, height, &mut pixels).unwrap(),
        PixelFormat::BC5_snorm_block => decode_bc5(reader, width, height, &mut pixels).unwrap(),
        PixelFormat::BC6H_ufloat_block => decode_bc6h(reader, width, height, &mut pixels, false).unwrap(),
        PixelFormat::BC6H_sfloat_block => decode_bc6h(reader, width, height, &mut pixels, true).unwrap(),
        PixelFormat::BC7_unorm_block => decode_bc7(reader, width, height, &mut pixels).unwrap(),
        PixelFormat::BC7_srgb_block => decode_bc7(reader, width, height, &mut pixels).unwrap(),
        _ => return Err(anyhow::anyhow!("Unsupported format: {:?}", format))
    };

    let _bgr = matches!(
        format,
        PixelFormat::BC1_RGB_unorm_block |
        PixelFormat::BC1_RGB_srgb_block |
        PixelFormat::BC1_RGBA_unorm_block |
        PixelFormat::BC1_RGBA_srgb_block |
        PixelFormat::BC2_unorm_block |
        PixelFormat::BC2_srgb_block |
        PixelFormat::BC3_unorm_block |
        PixelFormat::BC3_srgb_block |
        PixelFormat::BC4_unorm_block |
        PixelFormat::BC4_snorm_block |
        PixelFormat::BC5_unorm_block |
        PixelFormat::BC5_snorm_block |
        PixelFormat::BC6H_ufloat_block |
        PixelFormat::BC6H_sfloat_block |
        PixelFormat::BC7_unorm_block |
        PixelFormat::BC7_srgb_block
    );

    // let mut x = 0;
    // let mut y = 0;

    // for pixel in pixels {
    //     let r = (pixel & 0xFF) as u8;
    //     let g = ((pixel >> 8) & 0xFF) as u8;
    //     let b = ((pixel >> 16) & 0xFF) as u8;
    //     let a = ((pixel >> 24) & 0xFF) as u8;

    //     if bgr {
    //         image.put_pixel(x, y, image::Rgba([b, g, r, a]));
    //     } else {
    //         image.put_pixel(x, y, image::Rgba([r, g, b, a]));
    //     }

    //     x += 1;

    //     if x == width as u32 {
    //         x = 0;
    //         y += 1;
    //     }
    // }

    Ok(pixels)
}

fn decode_bc6h(
    data: &[u8],
    width: usize,
    height: usize,
    image: &mut [u32],
    signed: bool,
) -> Result<(), &'static str> {
    const BLOCK_WIDTH: usize = 4;
    const BLOCK_HEIGHT: usize = 4;
    const BLOCK_SIZE: usize = BLOCK_WIDTH * BLOCK_HEIGHT;
    let num_blocks_x: usize = width.div_ceil(BLOCK_WIDTH);
    let num_blocks_y: usize = (height + BLOCK_WIDTH - 1) / BLOCK_HEIGHT;

    if data.len() < num_blocks_x * num_blocks_y * 16 {
        return Err("Not enough data to decode image!");
    }

    if image.len() < width * height {
        return Err("Image buffer is too small!");
    }

    let mut block_buffer = [0f32; BLOCK_SIZE * 3];
    let mut data_offset = 0;

    for by in 0..num_blocks_y {
        for bx in 0..num_blocks_x {
            bc6h_float(&data[data_offset..], &mut block_buffer, 4 * 3, signed);
            copy_block_buffer(
                bx,
                by,
                width,
                height,
                &block_buffer,
                image,
            );
            data_offset += 16;
        }
    }
    Ok(())
}

#[inline]
fn pack_rgb(rgb: &[f32]) -> u32 {
    let r = (rgb[0] * 255.0) as u32;
    let g = (rgb[1] * 255.0) as u32;
    let b = (rgb[2] * 255.0) as u32;

    b << 16 | g << 8 | r
}

#[inline]
fn copy_block_buffer(
    bx: usize,
    by: usize,
    w: usize,
    h: usize,
    buffer: &[f32],
    image: &mut [u32],
) {
    let x: usize = 4 * bx;
    let copy_width: usize = if 4 * (bx + 1) > w { w - 4 * bx } else { 4 };

    let y_0 = by * 4;
    let copy_height: usize = if 4 * (by + 1) > h { h - y_0 } else { 4 };
    let mut buffer_offset = 0;

    for y in y_0..y_0 + copy_height {
        let image_offset = y * w + x;

        for x in 0..copy_width {
            image[image_offset + x] = pack_rgb(&buffer[buffer_offset..buffer_offset + 3]);
            buffer_offset += 3;
        }

        // image[image_offset..image_offset + copy_width]
        //     .copy_from_slice(&buffer[buffer_offset..buffer_offset + copy_width]);
    }
}
