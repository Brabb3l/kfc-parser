use block_compression::{encode::compress_rgba8, BC6HSettings, BC7Settings, CompressionVariant};
use half::f16;

use crate::image::{size_of_format, PixelFormat};

// #[inline(always)]
// const fn map8to2(value: u8) -> u8 {
//     value >> 6
// }

#[inline(always)]
const fn map8to4(value: u8) -> u8 {
    value >> 4
}

#[inline(always)]
const fn map8to5(value: u8) -> u8 {
    let value = value as u16 * 31;
    ((value + 128 + (value >> 8)) >> 8) as u8
}

#[inline(always)]
const fn map8to6(value: u8) -> u8 {
    let value = value as u16 * 63;
    ((value + 128 + (value >> 8)) >> 8) as u8
}

// #[inline(always)]
// const fn map8to10(value: u8) -> u16 {
//     let value = value as u32 * 1023;
//     ((value + 128 + (value >> 8)) >> 8) as u16
// }

macro_rules! encode_component {
    (U8, $src:expr, $dst:expr, $index:expr) => {{
        $dst[$index] = $src;
    }};
    (S8, $src:expr, $dst:expr, $index:expr) => {{
        $dst[$index] = $src.wrapping_add(1 << 7);
    }};
    (U16, $src:expr, $dst:expr, $index:expr) => {{
        $dst[$index..$index + 2].fill($src);
    }};
    (S16, $src:expr, $dst:expr, $index:expr) => {{
        let src = ($src as u16).wrapping_add(1 << 15);
        $dst.copy_from_slice(&src.to_le_bytes());
    }};
    (U32, $src:expr, $dst:expr, $index:expr) => {{
        $dst[$index..$index + 4].fill($src);
    }};
    (S32, $src:expr, $dst:expr, $index:expr) => {{
        let src = ($src as u32).wrapping_add(1 << 31);
        $dst.copy_from_slice(&src.to_le_bytes());
    }};
    (U64, $src:expr, $dst:expr, $index:expr) => {{
        $dst[$index..$index + 8].fill($src);
    }};
    (S64, $src:expr, $dst:expr, $index:expr) => {{
        let src = ($src as u64).wrapping_add(1 << 63);
        $dst.copy_from_slice(&src.to_le_bytes());
    }};
    (F16, $src:expr, $dst:expr, $index:expr) => {{
        let f16_bytes: [u8; 2] = f16::from_f32(($src as f32) / 255.0).to_le_bytes();
        $dst.copy_from_slice(&f16_bytes);
    }};
    (F32, $src:expr, $dst:expr, $index:expr) => {{
        let f32_bytes: [u8; 4] = (($src as f32) / 255.0).to_le_bytes();
        $dst.copy_from_slice(&f32_bytes);
    }};
    (F64, $src:expr, $dst:expr, $index:expr) => {{
        let f64_bytes: [u8; 8] = (($src as f64) / 255.0).to_le_bytes();
        $dst.copy_from_slice(&f64_bytes);
    }};
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

macro_rules! texel_size {
    (R, $ty:ident) => { component_size!($ty) };
    (RG, $ty:ident) => { component_size!($ty) * 2 };
    (RGB, $ty:ident) => { component_size!($ty) * 3 };
    (BGR, $ty:ident) => { component_size!($ty) * 3 };
    (RGBA, $ty:ident) => { component_size!($ty) * 4 };
    (BGRA, $ty:ident) => { component_size!($ty) * 4 };
    (ABGR, $ty:ident) => { component_size!($ty) * 4 };
    (R4G4) => { 1 };
    (R4G4B4A4) => { 2 };
    (B4G4R4A4) => { 2 };
    (R5G6B5) => { 2 };
    (B5G6R5) => { 2 };
    (R5G5B5A1) => { 2 };
    (B5G5R5A1) => { 2 };
    (A1R5G5B5) => { 2 };
    (B10G11R11F) => { 4 };
    (E5B9G9R9F) => { 4 };
}

macro_rules! encode_texel {
    ($src:expr, $dst:expr, R, $ty:ident) => {{
        encode_component!($ty, $src[0], $dst, 0);
    }};
    ($src:expr, $dst:expr, RG, $ty:ident) => {{
        encode_component!($ty, $src[0], $dst, 0);
        encode_component!($ty, $src[1], $dst, 1);
    }};
    ($src:expr, $dst:expr, RGB, $ty:ident) => {{
        encode_component!($ty, $src[0], $dst, 0);
        encode_component!($ty, $src[1], $dst, 1);
        encode_component!($ty, $src[2], $dst, 2);
    }};
    ($src:expr, $dst:expr, BGR, $ty:ident) => {{
        encode_component!($ty, $src[2], $dst, 0);
        encode_component!($ty, $src[1], $dst, 1);
        encode_component!($ty, $src[0], $dst, 2);
    }};
    ($src:expr, $dst:expr, RGBA, $ty:ident) => {{
        encode_component!($ty, $src[0], $dst, 0);
        encode_component!($ty, $src[1], $dst, 1);
        encode_component!($ty, $src[2], $dst, 2);
        encode_component!($ty, $src[3], $dst, 3);
    }};
    ($src:expr, $dst:expr, BGRA, $ty:ident) => {{
        encode_component!($ty, $src[2], $dst, 0);
        encode_component!($ty, $src[1], $dst, 1);
        encode_component!($ty, $src[0], $dst, 2);
        encode_component!($ty, $src[3], $dst, 3);
    }};
    ($src:expr, $dst:expr, ABGR, $ty:ident) => {{
        encode_component!($ty, $src[3], $dst, 0);
        encode_component!($ty, $src[2], $dst, 1);
        encode_component!($ty, $src[1], $dst, 2);
        encode_component!($ty, $src[0], $dst, 3);
    }};
    ($src:expr, $dst:expr, R4G4) => {{
        $dst[0] = (map8to4($src[0]) << 4) | map8to4($src[1]);
    }};
    ($src:expr, $dst:expr, R4G4B4A4) => {{
        $dst[1] = (map8to4($src[0]) << 4) | map8to4($src[1]);
        $dst[0] = (map8to4($src[2]) << 4) | map8to4($src[3]);
    }};
    ($src:expr, $dst:expr, B4G4R4A4) => {{
        $dst[1] = (map8to4($src[2]) << 4) | map8to4($src[1]);
        $dst[0] = (map8to4($src[0]) << 4) | map8to4($src[3]);
    }};
    ($src:expr, $dst:expr, R5G6B5) => {{
        $dst[1] = (map8to5($src[0]) << 3) | (map8to6($src[1]) >> 3);
        $dst[0] = ((map8to6($src[1]) & 0x07) << 5) | map8to5($src[2]);
    }};
    ($src:expr, $dst:expr, B5G6R5) => {{
        $dst[1] = (map8to5($src[2]) << 3) | (map8to6($src[1]) >> 3);
        $dst[0] = ((map8to6($src[1]) & 0x07) << 5) | map8to5($src[0]);
    }};
    ($src:expr, $dst:expr, R5G5B5A1) => {{
        $dst[1] = (map8to5($src[0]) << 3) | (map8to5($src[1]) >> 2);
        $dst[0] = ((map8to5($src[1]) & 0x03) << 6) | (map8to5($src[2]) << 1) | ($src[3] >> 7);
    }};
    ($src:expr, $dst:expr, B5G5R5A1) => {{
        $dst[1] = (map8to5($src[2]) << 3) | (map8to5($src[1]) >> 2);
        $dst[0] = ((map8to5($src[1]) & 0x03) << 6) | (map8to5($src[0]) << 1) | ($src[3] >> 7);
    }};
    ($src:expr, $dst:expr, A1R5G5B5) => {{
        $dst[1] = ($src[3] >> 7) << 7 | (map8to5($src[0]) << 2) | (map8to5($src[1]) >> 3);
        $dst[0] = ((map8to5($src[1]) & 0x07) << 5) | map8to5($src[2]);
    }};
    ($src:expr, $dst:expr, B10G11R11F) => {{
        // TODO: implement
    }};
    ($src:expr, $dst:expr, E5B9G9R9F) => {{
        // TODO: implement
    }};
}

macro_rules! encode {
    (
        $width:expr, $height:expr,
        $src:expr, $dst: expr,
        $format:ident$(, $numeric_type:ident)?$(,)?
    ) => {{
        let mut src = $src;
        let mut dst = $dst;

        for _ in 0..($width * $height) {
            encode_texel!(
                src,
                dst,
                $format$(, $numeric_type)?
            );
            src = &src[4..];
            dst = &mut dst[texel_size!($format$(, $numeric_type)?)..];
        }
    }};
}

pub fn encode(
    format: PixelFormat,
    width: usize,
    height: usize,
    src: &[u8],
    dst: &mut [u8],
) -> anyhow::Result<()> {
    let src = &src[..width * height * 4];
    let dst = &mut dst[..size_of_format(format, width, height)];

    match format {
        PixelFormat::None => anyhow::bail!("Unsupported pixel format: None"),
        PixelFormat::R4G4_unorm_pack8 => encode!(width, height, src, dst, R4G4),
        PixelFormat::R4G4B4A4_unorm_pack16 => encode!(width, height, src, dst, R4G4B4A4),
        PixelFormat::B4G4R4A4_unorm_pack16 => encode!(width, height, src, dst, B4G4R4A4),
        PixelFormat::R5G6B5_unorm_pack16 => encode!(width, height, src, dst, R5G6B5),
        PixelFormat::B5G6R5_unorm_pack16 => encode!(width, height, src, dst, B5G6R5),
        PixelFormat::R5G5B5A1_unorm_pack16 => encode!(width, height, src, dst, R5G5B5A1),
        PixelFormat::B5G5R5A1_unorm_pack16 => encode!(width, height, src, dst, B5G5R5A1),
        PixelFormat::A1R5G5B5_unorm_pack16 => encode!(width, height, src, dst, A1R5G5B5),
        PixelFormat::R8_unorm => encode!(width, height, src, dst, R, U8),
        PixelFormat::R8_snorm => encode!(width, height, src, dst, R, S8),
        PixelFormat::R8_uscaled => encode!(width, height, src, dst, R, U8),
        PixelFormat::R8_sscaled => encode!(width, height, src, dst, R, S8),
        PixelFormat::R8_uint => encode!(width, height, src, dst, R, U8),
        PixelFormat::R8_sint => encode!(width, height, src, dst, R, S8),
        PixelFormat::R8_srgb => anyhow::bail!("R8_srgb encoding not implemented"),
        PixelFormat::R8G8_unorm => encode!(width, height, src, dst, RG, U8),
        PixelFormat::R8G8_snorm => encode!(width, height, src, dst, RG, S8),
        PixelFormat::R8G8_uscaled => encode!(width, height, src, dst, RG, U8),
        PixelFormat::R8G8_sscaled => encode!(width, height, src, dst, RG, S8),
        PixelFormat::R8G8_uint => encode!(width, height, src, dst, RG, U8),
        PixelFormat::R8G8_sint => encode!(width, height, src, dst, RG, S8),
        PixelFormat::R8G8_srgb => anyhow::bail!("R8G8_srgb encoding not implemented"),
        PixelFormat::R8G8B8_unorm => encode!(width, height, src, dst, RGB, U8),
        PixelFormat::R8G8B8_snorm => encode!(width, height, src, dst, RGB, S8),
        PixelFormat::R8G8B8_uscaled => encode!(width, height, src, dst, RGB, U8),
        PixelFormat::R8G8B8_sscaled => encode!(width, height, src, dst, RGB, S8),
        PixelFormat::R8G8B8_uint => encode!(width, height, src, dst, RGB, U8),
        PixelFormat::R8G8B8_sint => encode!(width, height, src, dst, RGB, S8),
        PixelFormat::R8G8B8_srgb => anyhow::bail!("R8G8B8_srgb encoding not implemented"),
        PixelFormat::B8G8R8_unorm => encode!(width, height, src, dst, BGR, U8),
        PixelFormat::B8G8R8_snorm => encode!(width, height, src, dst, BGR, S8),
        PixelFormat::B8G8R8_uscaled => encode!(width, height, src, dst, BGR, U8),
        PixelFormat::B8G8R8_sscaled => encode!(width, height, src, dst, BGR, S8),
        PixelFormat::B8G8R8_uint => encode!(width, height, src, dst, BGR, U8),
        PixelFormat::B8G8R8_sint => encode!(width, height, src, dst, BGR, S8),
        PixelFormat::B8G8R8_srgb => anyhow::bail!("B8G8R8_srgb encoding not implemented"),
        PixelFormat::R8G8B8A8_unorm => encode!(width, height, src, dst, RGBA, U8),
        PixelFormat::R8G8B8A8_snorm => encode!(width, height, src, dst, RGBA, S8),
        PixelFormat::R8G8B8A8_uscaled => encode!(width, height, src, dst, RGBA, U8),
        PixelFormat::R8G8B8A8_sscaled => encode!(width, height, src, dst, RGBA, S8),
        PixelFormat::R8G8B8A8_uint => encode!(width, height, src, dst, RGBA, U8),
        PixelFormat::R8G8B8A8_sint => encode!(width, height, src, dst, RGBA, S8),
        PixelFormat::R8G8B8A8_srgb => anyhow::bail!("R8G8B8A8_srgb encoding not implemented"),
        PixelFormat::B8G8R8A8_unorm => encode!(width, height, src, dst, BGRA, U8),
        PixelFormat::B8G8R8A8_snorm => encode!(width, height, src, dst, BGRA, S8),
        PixelFormat::B8G8R8A8_uscaled => encode!(width, height, src, dst, BGRA, U8),
        PixelFormat::B8G8R8A8_sscaled => encode!(width, height, src, dst, BGRA, S8),
        PixelFormat::B8G8R8A8_uint => encode!(width, height, src, dst, BGRA, U8),
        PixelFormat::B8G8R8A8_sint => encode!(width, height, src, dst, BGRA, S8),
        PixelFormat::B8G8R8A8_srgb => anyhow::bail!("B8G8R8A8_srgb encoding not implemented"),
        PixelFormat::A8B8G8R8_unorm_pack32 => encode!(width, height, src, dst, ABGR, U8),
        PixelFormat::A8B8G8R8_snorm_pack32 => encode!(width, height, src, dst, ABGR, S8),
        PixelFormat::A8B8G8R8_uscaled_pack32 => encode!(width, height, src, dst, ABGR, U8),
        PixelFormat::A8B8G8R8_sscaled_pack32 => encode!(width, height, src, dst, ABGR, S8),
        PixelFormat::A8B8G8R8_uint_pack32 => encode!(width, height, src, dst, ABGR, U8),
        PixelFormat::A8B8G8R8_sint_pack32 => encode!(width, height, src, dst, ABGR, S8),
        PixelFormat::A8B8G8R8_srgb_pack32 => anyhow::bail!("A8B8G8R8_srgb_pack32 encoding not implemented"),
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
        PixelFormat::A2B10G10R10_sint_pack32 => anyhow::bail!("A2B10G10R10_*_pack32 encoding not implemented"),
        PixelFormat::R16_unorm => encode!(width, height, src, dst, R, U16),
        PixelFormat::R16_snorm => encode!(width, height, src, dst, R, S16),
        PixelFormat::R16_uscaled => encode!(width, height, src, dst, R, U16),
        PixelFormat::R16_sscaled => encode!(width, height, src, dst, R, S16),
        PixelFormat::R16_uint => encode!(width, height, src, dst, R, U16),
        PixelFormat::R16_sint => encode!(width, height, src, dst, R, S16),
        PixelFormat::R16_sfloat => encode!(width, height, src, dst, R, F16),
        PixelFormat::R16G16_unorm => encode!(width, height, src, dst, RG, U16),
        PixelFormat::R16G16_snorm => encode!(width, height, src, dst, RG, S16),
        PixelFormat::R16G16_uscaled => encode!(width, height, src, dst, RG, U16),
        PixelFormat::R16G16_sscaled => encode!(width, height, src, dst, RG, S16),
        PixelFormat::R16G16_uint => encode!(width, height, src, dst, RG, U16),
        PixelFormat::R16G16_sint => encode!(width, height, src, dst, RG, S16),
        PixelFormat::R16G16_sfloat => encode!(width, height, src, dst, RG, F16),
        PixelFormat::R16G16B16_unorm => encode!(width, height, src, dst, RGB, U16),
        PixelFormat::R16G16B16_snorm => encode!(width, height, src, dst, RGB, S16),
        PixelFormat::R16G16B16_uscaled => encode!(width, height, src, dst, RGB, U16),
        PixelFormat::R16G16B16_sscaled => encode!(width, height, src, dst, RGB, S16),
        PixelFormat::R16G16B16_uint => encode!(width, height, src, dst, RGB, U16),
        PixelFormat::R16G16B16_sint => encode!(width, height, src, dst, RGB, S16),
        PixelFormat::R16G16B16_sfloat => encode!(width, height, src, dst, RGB, F16),
        PixelFormat::R16G16B16A16_unorm => encode!(width, height, src, dst, RGBA, U16),
        PixelFormat::R16G16B16A16_snorm => encode!(width, height, src, dst, RGBA, S16),
        PixelFormat::R16G16B16A16_uscaled => encode!(width, height, src, dst, RGBA, U16),
        PixelFormat::R16G16B16A16_sscaled => encode!(width, height, src, dst, RGBA, S16),
        PixelFormat::R16G16B16A16_uint => encode!(width, height, src, dst, RGBA, U16),
        PixelFormat::R16G16B16A16_sint => encode!(width, height, src, dst, RGBA, S16),
        PixelFormat::R16G16B16A16_sfloat => encode!(width, height, src, dst, RGBA, F16),
        PixelFormat::R32_uint => encode!(width, height, src, dst, R, U32),
        PixelFormat::R32_sint => encode!(width, height, src, dst, R, S32),
        PixelFormat::R32_sfloat => encode!(width, height, src, dst, R, F32),
        PixelFormat::R32G32_uint => encode!(width, height, src, dst, RG, U32),
        PixelFormat::R32G32_sint => encode!(width, height, src, dst, RG, S32),
        PixelFormat::R32G32_sfloat => encode!(width, height, src, dst, RG, F32),
        PixelFormat::R32G32B32_uint => encode!(width, height, src, dst, RGB, U32),
        PixelFormat::R32G32B32_sint => encode!(width, height, src, dst, RGB, S32),
        PixelFormat::R32G32B32_sfloat => encode!(width, height, src, dst, RGB, F32),
        PixelFormat::R32G32B32A32_uint => encode!(width, height, src, dst, RGBA, U32),
        PixelFormat::R32G32B32A32_sint => encode!(width, height, src, dst, RGBA, S32),
        PixelFormat::R32G32B32A32_sfloat => encode!(width, height, src, dst, RGBA, F32),
        PixelFormat::R64_uint => encode!(width, height, src, dst, R, U64),
        PixelFormat::R64_sint => encode!(width, height, src, dst, R, S64),
        PixelFormat::R64_sfloat => encode!(width, height, src, dst, R, F64),
        PixelFormat::R64G64_uint => encode!(width, height, src, dst, RG, U64),
        PixelFormat::R64G64_sint => encode!(width, height, src, dst, RG, S64),
        PixelFormat::R64G64_sfloat => encode!(width, height, src, dst, RG, F64),
        PixelFormat::R64G64B64_uint => encode!(width, height, src, dst, RGB, U64),
        PixelFormat::R64G64B64_sint => encode!(width, height, src, dst, RGB, S64),
        PixelFormat::R64G64B64_sfloat => encode!(width, height, src, dst, RGB, F64),
        PixelFormat::R64G64B64A64_uint => encode!(width, height, src, dst, RGBA, U64),
        PixelFormat::R64G64B64A64_sint => encode!(width, height, src, dst, RGBA, S64),
        PixelFormat::R64G64B64A64_sfloat => encode!(width, height, src, dst, RGBA, F64),
        PixelFormat::B10G11R11_ufloat_pack32 => // encode!(width, height, src, dst, B10G11R11F),
            anyhow::bail!("E5B9G9R9_ufloat_pack32 encoding not implemented"),
        PixelFormat::E5B9G9R9_ufloat_pack32 => // encode!(width, height, src, dst, E5B9G9R9F),
            anyhow::bail!("E5B9G9R9_ufloat_pack32 encoding not implemented"),

        // TODO: handle srgb, currently treated as unorm
        PixelFormat::BC1_RGB_unorm_block => encode_bc1(width, height, src, dst),
        PixelFormat::BC1_RGB_srgb_block => encode_bc1(width, height, src, dst),
        PixelFormat::BC1_RGBA_unorm_block => encode_bc1(width, height, src, dst),
        PixelFormat::BC1_RGBA_srgb_block => encode_bc1(width, height, src, dst),
        PixelFormat::BC2_unorm_block => encode_bc2(width, height, src, dst),
        PixelFormat::BC2_srgb_block => encode_bc2(width, height, src, dst),
        PixelFormat::BC3_unorm_block => encode_bc3(width, height, src, dst),
        PixelFormat::BC3_srgb_block => encode_bc3(width, height, src, dst),
        PixelFormat::BC4_unorm_block => encode_bc4(width, height, src, dst),
        PixelFormat::BC4_snorm_block => encode_bc4(width, height, src, dst),
        PixelFormat::BC5_unorm_block => encode_bc5(width, height, src, dst),
        PixelFormat::BC5_snorm_block => encode_bc5(width, height, src, dst),
        PixelFormat::BC6H_ufloat_block => encode_bc6h(width, height, src, dst),
        PixelFormat::BC6H_sfloat_block => return Err(anyhow::anyhow!("BC6H_sfloat_block encoding not implemented")),
        PixelFormat::BC7_unorm_block => encode_bc7(width, height, src, dst),
        PixelFormat::BC7_srgb_block => encode_bc7(width, height, src, dst),

        // opaque
        PixelFormat::D16_unorm |
        PixelFormat::X8_D24_unorm_pack32 |
        PixelFormat::D32_sfloat |
        PixelFormat::S8_uint |
        PixelFormat::D16_unorm_S8_uint |
        PixelFormat::D24_unorm_S8_uint |
        PixelFormat::D32_sfloat_S8_uint => anyhow::bail!("Opaque pixel format: {:?}", format),
    }

    Ok(())
}

#[inline(always)]
fn encode_bc1(
    width: usize,
    height: usize,
    src: &[u8],
    dst: &mut [u8],
) {
    compress_rgba8(
        CompressionVariant::BC1,
        src,
        dst,
        width as u32,
        height as u32,
        width as u32 * 4,
    );
}

#[inline(always)]
fn encode_bc2(
    width: usize,
    height: usize,
    src: &[u8],
    dst: &mut [u8],
) {
    compress_rgba8(
        CompressionVariant::BC2,
        src,
        dst,
        width as u32,
        height as u32,
        width as u32 * 4,
    );
}

#[inline(always)]
fn encode_bc3(
    width: usize,
    height: usize,
    src: &[u8],
    dst: &mut [u8],
) {
    compress_rgba8(
        CompressionVariant::BC3,
        src,
        dst,
        width as u32,
        height as u32,
        width as u32 * 4,
    );
}

#[inline(always)]
fn encode_bc4(
    width: usize,
    height: usize,
    src: &[u8],
    dst: &mut [u8],
) {
    compress_rgba8(
        CompressionVariant::BC4,
        src,
        dst,
        width as u32,
        height as u32,
        width as u32 * 4,
    );
}

#[inline(always)]
fn encode_bc5(
    width: usize,
    height: usize,
    src: &[u8],
    dst: &mut [u8],
) {
    compress_rgba8(
        CompressionVariant::BC5,
        src,
        dst,
        width as u32,
        height as u32,
        width as u32 * 4,
    );
}

#[inline(always)]
fn encode_bc6h(
    width: usize,
    height: usize,
    src: &[u8],
    dst: &mut [u8],
) {
    compress_rgba8(
        CompressionVariant::BC6H(BC6HSettings::basic()),
        src,
        dst,
        width as u32,
        height as u32,
        width as u32 * 4,
    );
}

#[inline(always)]
fn encode_bc7(
    width: usize,
    height: usize,
    src: &[u8],
    dst: &mut [u8],
) {
    compress_rgba8(
        CompressionVariant::BC7(BC7Settings::alpha_basic()),
        src,
        dst,
        width as u32,
        height as u32,
        width as u32 * 4,
    );
}
