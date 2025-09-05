use block_compression::{decode::decompress_blocks_as_rgba8, BC6HSettings, BC7Settings, CompressionVariant};
use half::f16;

use crate::image::{size_of_format, PixelFormat};

// #[inline(always)]
// const fn map2to8(value: u8) -> u8 {
//     (value << 6) | (value << 4) | (value << 2) | value
// }

#[inline(always)]
const fn map4to8(value: u8) -> u8 {
    (value << 4) | value
}

#[inline(always)]
const fn map5to8(value: u8) -> u8 {
    (value << 3) | (value >> 2)
}

#[inline(always)]
const fn map6to8(value: u8) -> u8 {
    (value << 2) | (value >> 4)
}

// #[inline(always)]
// const fn map10to8(value: u16) -> u8 {
//     let value = value as u32 * 255;
//     ((value + 512 + (value >> 10)) >> 10) as u8
// }

#[inline(always)]
fn ufloat(
    exponent: u32,
    mantissa: u32,
    exponent_bits: u32,
    mantissa_bits: u32
) -> f32 {
    if exponent == 0 && mantissa == 0 {
        return 0.0;
    }

    let mantissa_upper = (1 << mantissa_bits) as f32;
    let bias = (1 << (exponent_bits - 1)) - 1;

    if exponent == 0 /* && mantissa != 0 */ {
        (2.0f32).powi(1 - bias) * (mantissa as f32 / mantissa_upper)
    } else if exponent == 31 {
        if mantissa == 0 {
            f32::INFINITY
        } else {
            f32::NAN
        }
    } else {
        (2.0f32).powi(exponent as i32 - bias) * (1.0 + (mantissa as f32 / mantissa_upper))
    }
}

#[inline(always)]
fn ufloat_10(value: u16) -> f32 {
    let exponent = (value >> 5) & 0x1F;
    let mantissa = value & 0x1F;

    ufloat(exponent as u32, mantissa as u32, 5, 5)
}

#[inline(always)]
fn ufloat_11(value: u16) -> f32 {
    let exponent = (value >> 6) & 0x1F;
    let mantissa = value & 0x3F;

    ufloat(exponent as u32, mantissa as u32, 5, 6)
}

macro_rules! decode_component {
    (U8, $value:expr, $index:expr) => { $value[$index] };
    (S8, $value:expr, $index:expr) => { ($value[$index] & 0x7F) | (!$value[$index] & 0x80) };
    (U16, $value:expr, $index:expr) => { decode_component!(U8, $value, $index * 2 + 1) };
    (S16, $value:expr, $index:expr) => { decode_component!(S8, $value, $index * 2 + 1) };
    (U32, $value:expr, $index:expr) => { decode_component!(U16, $value, $index * 4 + 3) };
    (S32, $value:expr, $index:expr) => { decode_component!(S16, $value, $index * 4 + 3) };
    (U64, $value:expr, $index:expr) => { decode_component!(U32, $value, $index * 8 + 7) };
    (S64, $value:expr, $index:expr) => { decode_component!(S32, $value, $index * 8 + 7) };
    (F16, $value:expr, $index:expr) => {
        ((f16::from_le_bytes($value[$index * 2..$index * 2 + 2].try_into().unwrap()).to_f32()) * 255.0) as u8
    };
    (F32, $value:expr, $index:expr) => {
        ((f32::from_le_bytes($value[$index * 4..$index * 4 + 4].try_into().unwrap())) * 255.0) as u8
    };
    (F64, $value:expr, $index:expr) => {
        ((f64::from_le_bytes($value[$index * 8..$index * 8 + 8].try_into().unwrap())) * 255.0) as u8
    };
}

macro_rules! component_size {
    (U8) => { 1 };
    (S8) => { 1 };
    (U16) => { 2 };
    (S16) => { 2 };
    (U32) => { 4 };
    (S32) => { 4 };
    (U64) => { 8 };
    (S64) => { 8 };
    (F16) => { 2 };
    (F32) => { 4 };
    (F64) => { 8 };
}

macro_rules! decode_texel {
    ($src:expr, $dst:expr, R, $ty:ident) => {{
        $dst[0] = decode_component!($ty, $src, 0);
        $dst[1] = 0;
        $dst[2] = 0;
        $dst[3] = 255;
        component_size!($ty)
    }};
    ($src:expr, $dst:expr, RG, $ty:ident) => {{
        $dst[0] = decode_component!($ty, $src, 0);
        $dst[1] = decode_component!($ty, $src, 1);
        $dst[2] = 0;
        $dst[3] = 255;
        component_size!($ty) * 2
    }};
    ($src:expr, $dst:expr, RGB, $ty:ident) => {{
        $dst[0] = decode_component!($ty, $src, 0);
        $dst[1] = decode_component!($ty, $src, 1);
        $dst[2] = decode_component!($ty, $src, 2);
        $dst[3] = 255;
        component_size!($ty) * 3
    }};
    ($src:expr, $dst:expr, BGR, $ty:ident) => {{
        $dst[2] = decode_component!($ty, $src, 0);
        $dst[1] = decode_component!($ty, $src, 1);
        $dst[0] = decode_component!($ty, $src, 2);
        $dst[3] = 255;
        component_size!($ty) * 3
    }};
    ($src:expr, $dst:expr, RGBA, $ty:ident) => {{
        $dst[0] = decode_component!($ty, $src, 0);
        $dst[1] = decode_component!($ty, $src, 1);
        $dst[2] = decode_component!($ty, $src, 2);
        $dst[3] = decode_component!($ty, $src, 3);
        component_size!($ty) * 4
    }};
    ($src:expr, $dst:expr, BGRA, $ty:ident) => {{
        $dst[2] = decode_component!($ty, $src, 0);
        $dst[1] = decode_component!($ty, $src, 1);
        $dst[0] = decode_component!($ty, $src, 2);
        $dst[3] = decode_component!($ty, $src, 3);
        component_size!($ty) * 4
    }};
    ($src:expr, $dst:expr, ABGR, $ty:ident) => {{
        $dst[3] = decode_component!($ty, $src, 0);
        $dst[2] = decode_component!($ty, $src, 1);
        $dst[1] = decode_component!($ty, $src, 2);
        $dst[0] = decode_component!($ty, $src, 3);
        component_size!($ty) * 4
    }};
    ($src:expr, $dst:expr, R4G4) => {{
        let value = $src[0];
        let r = value >> 4;
        let g = value & 0x0F;

        $dst[0] = map4to8(r);
        $dst[1] = map4to8(g);
        $dst[2] = 0;
        $dst[3] = 255;

        1
    }};
    ($src:expr, $dst:expr, R4G4B4A4) => {{
        let value = u16::from_le_bytes([$src[0], $src[1]]);
        let r = ((value >> 12) & 0x0F) as u8;
        let g = ((value >> 8) & 0x0F) as u8;
        let b = ((value >> 4) & 0x0F) as u8;
        let a = (value & 0x0F) as u8;

        $dst[0] = map4to8(r);
        $dst[1] = map4to8(g);
        $dst[2] = map4to8(b);
        $dst[3] = map4to8(a);

        2
    }};
    ($src:expr, $dst:expr, B4G4R4A4) => {{
        let value = u16::from_le_bytes([$src[0], $src[1]]);
        let b = ((value >> 12) & 0x0F) as u8;
        let g = ((value >> 8) & 0x0F) as u8;
        let r = ((value >> 4) & 0x0F) as u8;
        let a = (value & 0x0F) as u8;

        $dst[0] = map4to8(r);
        $dst[1] = map4to8(g);
        $dst[2] = map4to8(b);
        $dst[3] = map4to8(a);

        2
    }};
    ($src:expr, $dst:expr, R5G6B5) => {{
        let value = u16::from_le_bytes([$src[0], $src[1]]);
        let r = ((value >> 11) & 0x1F) as u8;
        let g = ((value >> 5) & 0x3F) as u8;
        let b = (value & 0x1F) as u8;

        $dst[0] = map5to8(r);
        $dst[1] = map6to8(g);
        $dst[2] = map5to8(b);
        $dst[3] = 255;

        2
    }};
    ($src:expr, $dst:expr, B5G6R5) => {{
        let value = u16::from_le_bytes([$src[0], $src[1]]);
        let b = ((value >> 11) & 0x1F) as u8;
        let g = ((value >> 5) & 0x3F) as u8;
        let r = (value & 0x1F) as u8;

        $dst[0] = map5to8(r);
        $dst[1] = map6to8(g);
        $dst[2] = map5to8(b);

        2
    }};
    ($src:expr, $dst:expr, R5G5B5A1) => {{
        let value = u16::from_le_bytes([$src[0], $src[1]]);
        let r = ((value >> 11) & 0x1F) as u8;
        let g = ((value >> 6) & 0x1F) as u8;
        let b = ((value >> 1) & 0x1F) as u8;
        let a = (value & 0x01) as u8;

        $dst[0] = map5to8(r);
        $dst[1] = map5to8(g);
        $dst[2] = map5to8(b);
        $dst[3] = a * 255;

        2
    }};
    ($src:expr, $dst:expr, B5G5R5A1) => {{
        let value = u16::from_le_bytes([$src[0], $src[1]]);
        let b = ((value >> 11) & 0x1F) as u8;
        let g = ((value >> 6) & 0x1F) as u8;
        let r = ((value >> 1) & 0x1F) as u8;
        let a = (value & 0x01) as u8;

        $dst[0] = map5to8(r);
        $dst[1] = map5to8(g);
        $dst[2] = map5to8(b);
        $dst[3] = a * 255;

        2
    }};
    ($src:expr, $dst:expr, A1R5G5B5) => {{
        let value = u16::from_le_bytes([$src[0], $src[1]]);
        let a = ((value >> 15) & 0x01) as u8;
        let r = ((value >> 10) & 0x1F) as u8;
        let g = ((value >> 5) & 0x1F) as u8;
        let b = (value & 0x1F) as u8;

        $dst[0] = map5to8(r);
        $dst[1] = map5to8(g);
        $dst[2] = map5to8(b);
        $dst[3] = a * 255;

        2
    }};
    ($src:expr, $dst:expr, B10G11R11F) => {{
        let value = u32::from_le_bytes([$src[0], $src[1], $src[2], $src[3]]);
        let b = ((value >> 22) & 0x3FF) as u16;
        let g = ((value >> 11) & 0x7FF) as u16;
        let r = (value & 0x7FF) as u16;

        $dst[0] = (ufloat_10(r) * 255.0) as u8;
        $dst[1] = (ufloat_11(g) * 255.0) as u8;
        $dst[2] = (ufloat_11(b) * 255.0) as u8;
        $dst[3] = 255;

        4
    }};
    ($src:expr, $dst:expr, E5B9G9R9F) => {{
        let value = u32::from_le_bytes([$src[0], $src[1], $src[2], $src[3]]);
        let e = ((value >> 27) & 0x1F) as u32;
        let b = ((value >> 18) & 0x1FF) as u32;
        let g = ((value >> 9) & 0x1FF) as u32;
        let r = (value & 0x1FF) as u32;

        $dst[0] = (ufloat(e, r, 5, 9) * 255.0) as u8;
        $dst[1] = (ufloat(e, g, 5, 9) * 255.0) as u8;
        $dst[2] = (ufloat(e, b, 5, 9) * 255.0) as u8;
        $dst[3] = 255;

        4
    }};
}

macro_rules! decode {
    (
        $width:expr, $height:expr,
        $src:expr, $dst: expr,
        $format:ident$(, $numeric_type:ident)?$(,)?
    ) => {{
        let mut src = $src;

        for i in 0..($width * $height) {
            let size = decode_texel!(
                src,
                &mut $dst[i * 4..],
                $format$(, $numeric_type)?
            );
            src = &src[size..];
        }
    }};
}

pub fn decode(
    format: PixelFormat,
    width: usize,
    height: usize,
    src: &[u8],
    dst: &mut [u8],
) -> anyhow::Result<()> {
    let src = &src[..size_of_format(format, width, height)];
    let dst = &mut dst[..width * height * 4];

    match format {
        PixelFormat::None => return Err(anyhow::anyhow!("Unsupported pixel format: None")),
        PixelFormat::R4G4_unorm_pack8 => decode!(width, height, src, dst, R4G4),
        PixelFormat::R4G4B4A4_unorm_pack16 => decode!(width, height, src, dst, R4G4B4A4),
        PixelFormat::B4G4R4A4_unorm_pack16 => decode!(width, height, src, dst, B4G4R4A4),
        PixelFormat::R5G6B5_unorm_pack16 => decode!(width, height, src, dst, R5G6B5),
        PixelFormat::B5G6R5_unorm_pack16 => decode!(width, height, src, dst, B5G6R5),
        PixelFormat::R5G5B5A1_unorm_pack16 => decode!(width, height, src, dst, R5G5B5A1),
        PixelFormat::B5G5R5A1_unorm_pack16 => decode!(width, height, src, dst, B5G5R5A1),
        PixelFormat::A1R5G5B5_unorm_pack16 => decode!(width, height, src, dst, A1R5G5B5),
        PixelFormat::R8_unorm => decode!(width, height, src, dst, R, U8),
        PixelFormat::R8_snorm => decode!(width, height, src, dst, R, S8),
        PixelFormat::R8_uscaled => decode!(width, height, src, dst, R, U8),
        PixelFormat::R8_sscaled => decode!(width, height, src, dst, R, S8),
        PixelFormat::R8_uint => decode!(width, height, src, dst, R, U8),
        PixelFormat::R8_sint => decode!(width, height, src, dst, R, S8),
        PixelFormat::R8_srgb => anyhow::bail!("R8_srgb decoding not implemented"),
        PixelFormat::R8G8_unorm => decode!(width, height, src, dst, RG, U8),
        PixelFormat::R8G8_snorm => decode!(width, height, src, dst, RG, S8),
        PixelFormat::R8G8_uscaled => decode!(width, height, src, dst, RG, U8),
        PixelFormat::R8G8_sscaled => decode!(width, height, src, dst, RG, S8),
        PixelFormat::R8G8_uint => decode!(width, height, src, dst, RG, U8),
        PixelFormat::R8G8_sint => decode!(width, height, src, dst, RG, S8),
        PixelFormat::R8G8_srgb => anyhow::bail!("R8G8_srgb decoding not implemented"),
        PixelFormat::R8G8B8_unorm => decode!(width, height, src, dst, RGB, U8),
        PixelFormat::R8G8B8_snorm => decode!(width, height, src, dst, RGB, S8),
        PixelFormat::R8G8B8_uscaled => decode!(width, height, src, dst, RGB, U8),
        PixelFormat::R8G8B8_sscaled => decode!(width, height, src, dst, RGB, S8),
        PixelFormat::R8G8B8_uint => decode!(width, height, src, dst, RGB, U8),
        PixelFormat::R8G8B8_sint => decode!(width, height, src, dst, RGB, S8),
        PixelFormat::R8G8B8_srgb => anyhow::bail!("R8G8B8_srgb decoding not implemented"),
        PixelFormat::B8G8R8_unorm => decode!(width, height, src, dst, BGR, U8),
        PixelFormat::B8G8R8_snorm => decode!(width, height, src, dst, BGR, S8),
        PixelFormat::B8G8R8_uscaled => decode!(width, height, src, dst, BGR, U8),
        PixelFormat::B8G8R8_sscaled => decode!(width, height, src, dst, BGR, S8),
        PixelFormat::B8G8R8_uint => decode!(width, height, src, dst, BGR, U8),
        PixelFormat::B8G8R8_sint => decode!(width, height, src, dst, BGR, S8),
        PixelFormat::B8G8R8_srgb => anyhow::bail!("B8G8R8_srgb decoding not implemented"),
        PixelFormat::R8G8B8A8_unorm => decode!(width, height, src, dst, RGBA, U8),
        PixelFormat::R8G8B8A8_snorm => decode!(width, height, src, dst, RGBA, S8),
        PixelFormat::R8G8B8A8_uscaled => decode!(width, height, src, dst, RGBA, U8),
        PixelFormat::R8G8B8A8_sscaled => decode!(width, height, src, dst, RGBA, S8),
        PixelFormat::R8G8B8A8_uint => decode!(width, height, src, dst, RGBA, U8),
        PixelFormat::R8G8B8A8_sint => decode!(width, height, src, dst, RGBA, S8),
        PixelFormat::R8G8B8A8_srgb => anyhow::bail!("R8G8B8A8_srgb decoding not implemented"),
        PixelFormat::B8G8R8A8_unorm => decode!(width, height, src, dst, BGRA, U8),
        PixelFormat::B8G8R8A8_snorm => decode!(width, height, src, dst, BGRA, S8),
        PixelFormat::B8G8R8A8_uscaled => decode!(width, height, src, dst, BGRA, U8),
        PixelFormat::B8G8R8A8_sscaled => decode!(width, height, src, dst, BGRA, S8),
        PixelFormat::B8G8R8A8_uint => decode!(width, height, src, dst, BGRA, U8),
        PixelFormat::B8G8R8A8_sint => decode!(width, height, src, dst, BGRA, S8),
        PixelFormat::B8G8R8A8_srgb => anyhow::bail!("B8G8R8A8_srgb decoding not implemented"),
        PixelFormat::A8B8G8R8_unorm_pack32 => decode!(width, height, src, dst, ABGR, U8),
        PixelFormat::A8B8G8R8_snorm_pack32 => decode!(width, height, src, dst, ABGR, S8),
        PixelFormat::A8B8G8R8_uscaled_pack32 => decode!(width, height, src, dst, ABGR, U8),
        PixelFormat::A8B8G8R8_sscaled_pack32 => decode!(width, height, src, dst, ABGR, S8),
        PixelFormat::A8B8G8R8_uint_pack32 => decode!(width, height, src, dst, ABGR, U8),
        PixelFormat::A8B8G8R8_sint_pack32 => decode!(width, height, src, dst, ABGR, S8),
        PixelFormat::A8B8G8R8_srgb_pack32 => anyhow::bail!("A8B8G8R8_srgb_pack32 decoding not implemented"),
        PixelFormat::A2R10G10B10_unorm_pack32 |
        PixelFormat::A2R10G10B10_snorm_pack32 |
        PixelFormat::A2R10G10B10_uscaled_pack32 |
        PixelFormat::A2R10G10B10_sscaled_pack32 |
        PixelFormat::A2R10G10B10_uint_pack32 |
        PixelFormat::A2R10G10B10_sint_pack32 |
        PixelFormat::A2B10G10R10_unorm_pack32 |
        PixelFormat::A2B10G10R10_snorm_pack32 |
        PixelFormat::A2B10G10R10_uscaled_pack32 |
        PixelFormat::A2B10G10R10_sscaled_pack32 |
        PixelFormat::A2B10G10R10_uint_pack32 |
        PixelFormat::A2B10G10R10_sint_pack32 => anyhow::bail!("A2B10G10R10_*_pack32 decoding not implemented"),
        PixelFormat::R16_unorm => decode!(width, height, src, dst, R, U16),
        PixelFormat::R16_snorm => decode!(width, height, src, dst, R, S16),
        PixelFormat::R16_uscaled => decode!(width, height, src, dst, R, U16),
        PixelFormat::R16_sscaled => decode!(width, height, src, dst, R, S16),
        PixelFormat::R16_uint => decode!(width, height, src, dst, R, U16),
        PixelFormat::R16_sint => decode!(width, height, src, dst, R, S16),
        PixelFormat::R16_sfloat => decode!(width, height, src, dst, R, F16),
        PixelFormat::R16G16_unorm => decode!(width, height, src, dst, RG, U16),
        PixelFormat::R16G16_snorm => decode!(width, height, src, dst, RG, S16),
        PixelFormat::R16G16_uscaled => decode!(width, height, src, dst, RG, U16),
        PixelFormat::R16G16_sscaled => decode!(width, height, src, dst, RG, S16),
        PixelFormat::R16G16_uint => decode!(width, height, src, dst, RG, U16),
        PixelFormat::R16G16_sint => decode!(width, height, src, dst, RG, S16),
        PixelFormat::R16G16_sfloat => decode!(width, height, src, dst, RG, F16),
        PixelFormat::R16G16B16_unorm => decode!(width, height, src, dst, RGB, U16),
        PixelFormat::R16G16B16_snorm => decode!(width, height, src, dst, RGB, S16),
        PixelFormat::R16G16B16_uscaled => decode!(width, height, src, dst, RGB, U16),
        PixelFormat::R16G16B16_sscaled => decode!(width, height, src, dst, RGB, S16),
        PixelFormat::R16G16B16_uint => decode!(width, height, src, dst, RGB, U16),
        PixelFormat::R16G16B16_sint => decode!(width, height, src, dst, RGB, S16),
        PixelFormat::R16G16B16_sfloat => decode!(width, height, src, dst, RGB, F16),
        PixelFormat::R16G16B16A16_unorm => decode!(width, height, src, dst, RGBA, U16),
        PixelFormat::R16G16B16A16_snorm => decode!(width, height, src, dst, RGBA, S16),
        PixelFormat::R16G16B16A16_uscaled => decode!(width, height, src, dst, RGBA, U16),
        PixelFormat::R16G16B16A16_sscaled => decode!(width, height, src, dst, RGBA, S16),
        PixelFormat::R16G16B16A16_uint => decode!(width, height, src, dst, RGBA, U16),
        PixelFormat::R16G16B16A16_sint => decode!(width, height, src, dst, RGBA, S16),
        PixelFormat::R16G16B16A16_sfloat => decode!(width, height, src, dst, RGBA, F16),
        PixelFormat::R32_uint => decode!(width, height, src, dst, R, U32),
        PixelFormat::R32_sint => decode!(width, height, src, dst, R, S32),
        PixelFormat::R32_sfloat => decode!(width, height, src, dst, R, F32),
        PixelFormat::R32G32_uint => decode!(width, height, src, dst, RG, U32),
        PixelFormat::R32G32_sint => decode!(width, height, src, dst, RG, S32),
        PixelFormat::R32G32_sfloat => decode!(width, height, src, dst, RG, F32),
        PixelFormat::R32G32B32_uint => decode!(width, height, src, dst, RGB, U32),
        PixelFormat::R32G32B32_sint => decode!(width, height, src, dst, RGB, S32),
        PixelFormat::R32G32B32_sfloat => decode!(width, height, src, dst, RGB, F32),
        PixelFormat::R32G32B32A32_uint => decode!(width, height, src, dst, RGBA, U32),
        PixelFormat::R32G32B32A32_sint => decode!(width, height, src, dst, RGBA, S32),
        PixelFormat::R32G32B32A32_sfloat => decode!(width, height, src, dst, RGBA, F32),
        PixelFormat::R64_uint => decode!(width, height, src, dst, R, U64),
        PixelFormat::R64_sint => decode!(width, height, src, dst, R, S64),
        PixelFormat::R64_sfloat => decode!(width, height, src, dst, R, F64),
        PixelFormat::R64G64_uint => decode!(width, height, src, dst, RG, U64),
        PixelFormat::R64G64_sint => decode!(width, height, src, dst, RG, S64),
        PixelFormat::R64G64_sfloat => decode!(width, height, src, dst, RG, F64),
        PixelFormat::R64G64B64_uint => decode!(width, height, src, dst, RGB, U64),
        PixelFormat::R64G64B64_sint => decode!(width, height, src, dst, RGB, S64),
        PixelFormat::R64G64B64_sfloat => decode!(width, height, src, dst, RGB, F64),
        PixelFormat::R64G64B64A64_uint => decode!(width, height, src, dst, RGBA, U64),
        PixelFormat::R64G64B64A64_sint => decode!(width, height, src, dst, RGBA, S64),
        PixelFormat::R64G64B64A64_sfloat => decode!(width, height, src, dst, RGBA, F64),
        PixelFormat::B10G11R11_ufloat_pack32 => decode!(width, height, src, dst, B10G11R11F),
        PixelFormat::E5B9G9R9_ufloat_pack32 => decode!(width, height, src, dst, E5B9G9R9F),

        // TODO: handle srgb, currently treated as unorm
        PixelFormat::BC1_RGB_unorm_block => decode_bc1(width, height, src, dst),
        PixelFormat::BC1_RGB_srgb_block => decode_bc1(width, height, src, dst),
        PixelFormat::BC1_RGBA_unorm_block => decode_bc1(width, height, src, dst),
        PixelFormat::BC1_RGBA_srgb_block => decode_bc1(width, height, src, dst),
        PixelFormat::BC2_unorm_block => decode_bc2(width, height, src, dst),
        PixelFormat::BC2_srgb_block => decode_bc2(width, height, src, dst),
        PixelFormat::BC3_unorm_block => decode_bc3(width, height, src, dst),
        PixelFormat::BC3_srgb_block => decode_bc3(width, height, src, dst),
        PixelFormat::BC4_unorm_block => decode_bc4(width, height, src, dst),
        PixelFormat::BC4_snorm_block => decode_bc4(width, height, src, dst),
        PixelFormat::BC5_unorm_block => decode_bc5(width, height, src, dst),
        PixelFormat::BC5_snorm_block => decode_bc5(width, height, src, dst),
        PixelFormat::BC6H_ufloat_block => decode_bc6h(width, height, src, dst),
        PixelFormat::BC6H_sfloat_block => return Err(anyhow::anyhow!("BC6H_sfloat_block decoding not implemented")),
        PixelFormat::BC7_unorm_block => decode_bc7(width, height, src, dst),
        PixelFormat::BC7_srgb_block => decode_bc7(width, height, src, dst),

        // opaque
        PixelFormat::D16_unorm |
        PixelFormat::X8_D24_unorm_pack32 |
        PixelFormat::D32_sfloat |
        PixelFormat::S8_uint |
        PixelFormat::D16_unorm_S8_uint |
        PixelFormat::D24_unorm_S8_uint |
        PixelFormat::D32_sfloat_S8_uint => return Err(anyhow::anyhow!("Opaque pixel format: {:?}", format)),
    }

    Ok(())
}

#[inline(always)]
fn decode_bc1(
    width: usize,
    height: usize,
    src: &[u8],
    dst: &mut [u8],
) {
    decompress_blocks_as_rgba8(
        CompressionVariant::BC1,
        width as u32,
        height as u32,
        src,
        dst
    );
}

#[inline(always)]
fn decode_bc2(
    width: usize,
    height: usize,
    src: &[u8],
    dst: &mut [u8],
) {
    decompress_blocks_as_rgba8(
        CompressionVariant::BC2,
        width as u32,
        height as u32,
        src,
        dst
    );
}

#[inline(always)]
fn decode_bc3(
    width: usize,
    height: usize,
    src: &[u8],
    dst: &mut [u8],
) {
    decompress_blocks_as_rgba8(
        CompressionVariant::BC3,
        width as u32,
        height as u32,
        src,
        dst
    );
}

#[inline(always)]
fn decode_bc4(
    width: usize,
    height: usize,
    src: &[u8],
    dst: &mut [u8],
) {
    decompress_blocks_as_rgba8(
        CompressionVariant::BC4,
        width as u32,
        height as u32,
        src,
        dst
    );
}

#[inline(always)]
fn decode_bc5(
    width: usize,
    height: usize,
    src: &[u8],
    dst: &mut [u8],
) {
    decompress_blocks_as_rgba8(
        CompressionVariant::BC5,
        width as u32,
        height as u32,
        src,
        dst
    );
}

#[inline(always)]
fn decode_bc6h(
    width: usize,
    height: usize,
    src: &[u8],
    dst: &mut [u8],
) {
    decompress_blocks_as_rgba8(
        CompressionVariant::BC6H(BC6HSettings::basic()),
        width as u32,
        height as u32,
        src,
        dst
    );
}

#[inline(always)]
fn decode_bc7(
    width: usize,
    height: usize,
    src: &[u8],
    dst: &mut [u8],
) {
    decompress_blocks_as_rgba8(
        CompressionVariant::BC7(BC7Settings::alpha_basic()),
        width as u32,
        height as u32,
        src,
        dst
    );
}
