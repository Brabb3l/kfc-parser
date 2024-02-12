use std::io::{Read, Seek, Write};
use hound::{SampleFormat, WavSpec, WavWriter};
use image::RgbaImage;
use texture2ddecoder::{decode_bc1, decode_bc3, decode_bc4, decode_bc5, decode_bc6, decode_bc7};
use shared::io::Reader;
use crate::error::CrpfNodeParseError;
use crate::file::enums::PixelFormat;
use crate::file::structs::{ContentHash, RenderMaterialImage, SoundResource, UiTextureResource};
use crate::types::Guid;

impl ContentHash {
    pub fn hash(&self) -> u64 {
        let mut data = [0; 16];

        let size = self.size.to_le_bytes();
        let hash0 = self.hash0.to_le_bytes();
        let hash1 = self.hash1.to_le_bytes();
        let hash2 = self.hash2.to_le_bytes();

        data[0..4].copy_from_slice(&size);
        data[4..8].copy_from_slice(&hash0);
        data[8..12].copy_from_slice(&hash1);
        data[12..16].copy_from_slice(&hash2);

        Guid::new(data).hash()
    }
}

impl SoundResource {
    pub fn export_content<R: Read, W: Write + Seek>(
        &self,
        reader: R,
        writer: W
    ) -> anyhow::Result<()> {
        let mut reader = Reader::new(reader);

        let channels = match self.channel_configuration.value_name.as_str() {
            "Mono" => 1,
            "Stereo" => 2,
            "Quadrophonic" => 4,
            "FiveDotOne" => 6,
            "SevenDotOne" => 8,
            _ => return Err(CrpfNodeParseError::InvalidChannelConfig(
                self.channel_configuration.value_name.clone()).into()
            ),
        };

        let spec = WavSpec {
            channels,
            sample_rate: self.frames_per_second as u32,
            bits_per_sample: 16,
            sample_format: SampleFormat::Int
        };

        let mut wav_writer = WavWriter::new(writer, spec)?;
        let sample_count = self.frame_count * channels as u32;

        for _ in 0..sample_count {
            let sample = reader.read_i16()?;
            wav_writer.write_sample(sample)?;
        }

        Ok(())
    }
}

impl RenderMaterialImage {
    pub fn export_content(
        &self,
        reader: &[u8]
    ) -> anyhow::Result<RgbaImage> {
        export_image(
            self.format,
            self.width,
            self.height,
            reader
        )
    }
}

impl UiTextureResource {
    pub fn export_content(
        &self,
        reader: &[u8]
    ) -> anyhow::Result<RgbaImage> {
        export_image(
            self.format,
            self.size.x as u16,
            self.size.y as u16,
            reader
        )
    }
}

fn export_image(
    format: PixelFormat,
    width: u16,
    height: u16,
    reader: &[u8],
) -> anyhow::Result<RgbaImage> {
    let width = width as usize;
    let height = height as usize;

    let mut image = RgbaImage::new(width as u32, height as u32);
    let mut pixels = vec![0; width * height];

    match format {
        PixelFormat::R8_unorm |
        PixelFormat::R8_snorm |
        PixelFormat::R8_uscaled |
        PixelFormat::R8_sscaled |
        PixelFormat::R8_uint |
        PixelFormat::R8_sint |
        PixelFormat::R8_srgb => {
            let reader = &reader[..width * height];
            for (i, &pixel) in reader.iter().enumerate() {
                pixels[i] = pixel as u32;
            }
        }

        PixelFormat::R8G8_unorm |
        PixelFormat::R8G8_snorm |
        PixelFormat::R8G8_uscaled |
        PixelFormat::R8G8_sscaled |
        PixelFormat::R8G8_uint |
        PixelFormat::R8G8_sint |
        PixelFormat::R8G8_srgb => {
            let reader = &reader[..width * height * 2];
            for (i, pixel) in reader.chunks(2).enumerate() {
                let r = pixel[0];
                let g = pixel[1];

                pixels[i] = (g as u32) << 8 | r as u32;
            }
        }

        PixelFormat::R8G8B8_unorm |
        PixelFormat::R8G8B8_snorm |
        PixelFormat::R8G8B8_uscaled |
        PixelFormat::R8G8B8_sscaled |
        PixelFormat::R8G8B8_uint |
        PixelFormat::R8G8B8_sint |
        PixelFormat::R8G8B8_srgb => {
            let reader = &reader[..width * height * 3];
            for (i, pixel) in reader.chunks(3).enumerate() {
                let r = pixel[0];
                let g = pixel[1];
                let b = pixel[2];

                pixels[i] = (b as u32) << 16 | (g as u32) << 8 | r as u32;
            }
        }

        PixelFormat::B8G8R8_unorm |
        PixelFormat::B8G8R8_snorm |
        PixelFormat::B8G8R8_uscaled |
        PixelFormat::B8G8R8_sscaled |
        PixelFormat::B8G8R8_uint |
        PixelFormat::B8G8R8_sint |
        PixelFormat::B8G8R8_srgb => {
            let reader = &reader[..width * height * 3];
            for (i, pixel) in reader.chunks(3).enumerate() {
                let r = pixel[2];
                let g = pixel[1];
                let b = pixel[0];

                pixels[i] = (b as u32) << 16 | (g as u32) << 8 | r as u32;
            }
        }

        PixelFormat::R8G8B8A8_unorm |
        PixelFormat::R8G8B8A8_snorm |
        PixelFormat::R8G8B8A8_uscaled |
        PixelFormat::R8G8B8A8_sscaled |
        PixelFormat::R8G8B8A8_uint |
        PixelFormat::R8G8B8A8_sint |
        PixelFormat::R8G8B8A8_srgb => {
            let reader = &reader[..width * height * 4];
            for (i, pixel) in reader.chunks(4).enumerate() {
                let r = pixel[0];
                let g = pixel[1];
                let b = pixel[2];
                let a = pixel[3];

                pixels[i] = (a as u32) << 24 | (b as u32) << 16 | (g as u32) << 8 | r as u32;
            }
        }

        PixelFormat::B8G8R8A8_unorm |
        PixelFormat::B8G8R8A8_snorm |
        PixelFormat::B8G8R8A8_uscaled |
        PixelFormat::B8G8R8A8_sscaled |
        PixelFormat::B8G8R8A8_uint |
        PixelFormat::B8G8R8A8_sint |
        PixelFormat::B8G8R8A8_srgb => {
            let reader = &reader[..width * height * 4];
            for (i, pixel) in reader.chunks(4).enumerate() {
                let r = pixel[2];
                let g = pixel[1];
                let b = pixel[0];
                let a = pixel[3];

                pixels[i] = (a as u32) << 24 | (b as u32) << 16 | (g as u32) << 8 | r as u32;
            }
        }

        PixelFormat::A8B8G8R8_unorm_pack32 |
        PixelFormat::A8B8G8R8_snorm_pack32 |
        PixelFormat::A8B8G8R8_uscaled_pack32 |
        PixelFormat::A8B8G8R8_sscaled_pack32 |
        PixelFormat::A8B8G8R8_uint_pack32 |
        PixelFormat::A8B8G8R8_sint_pack32 |
        PixelFormat::A8B8G8R8_srgb_pack32 => {
            let reader = &reader[..width * height * 4];
            for (i, pixel) in reader.chunks(4).enumerate() {
                let r = pixel[3];
                let g = pixel[2];
                let b = pixel[1];
                let a = pixel[0];

                pixels[i] = (a as u32) << 24 | (b as u32) << 16 | (g as u32) << 8 | r as u32;
            }
        }

        PixelFormat::A2R10G10B10_unorm_pack32 |
        PixelFormat::A2R10G10B10_snorm_pack32 |
        PixelFormat::A2R10G10B10_uscaled_pack32 |
        PixelFormat::A2R10G10B10_sscaled_pack32 |
        PixelFormat::A2R10G10B10_uint_pack32 |
        PixelFormat::A2R10G10B10_sint_pack32 => {
            let reader = &reader[..width * height * 4];
            for (i, pixel) in reader.chunks(4).enumerate() {
                let data = u32::from_be_bytes(pixel.try_into()?);

                let a = (data >> 30) & 0x3;
                let b = (data >> 20) & 0x3FF;
                let g = (data >> 10) & 0x3FF;
                let r = data & 0x3FF;

                let a = (a * 0xFF) / 0x3;
                let b = (b * 0xFF) / 0x3FF;
                let g = (g * 0xFF) / 0x3FF;
                let r = (r * 0xFF) / 0x3FF;

                pixels[i] = a << 24 | b << 16 | g << 8 | r;
            }
        }

        PixelFormat::A2B10G10R10_unorm_pack32 |
        PixelFormat::A2B10G10R10_snorm_pack32 |
        PixelFormat::A2B10G10R10_uscaled_pack32 |
        PixelFormat::A2B10G10R10_sscaled_pack32 |
        PixelFormat::A2B10G10R10_uint_pack32 |
        PixelFormat::A2B10G10R10_sint_pack32 => {
            let reader = &reader[..width * height * 4];
            for (i, pixel) in reader.chunks(4).enumerate() {
                let data = u32::from_be_bytes(pixel.try_into()?);

                let a = (data >> 30) & 0x3;
                let r = (data >> 20) & 0x3FF;
                let g = (data >> 10) & 0x3FF;
                let b = data & 0x3FF;

                let a = (a * 0xFF) / 0x3;
                let r = (r * 0xFF) / 0x3FF;
                let g = (g * 0xFF) / 0x3FF;
                let b = (b * 0xFF) / 0x3FF;

                pixels[i] = a << 24 | b << 16 | g << 8 | r;
            }
        }

        // TODO: R16, R16G16, R16G16B16, R16G16B16A16, R32, R32G32, R32G32B32, R32G32B32A32, R64, R64G64, R64G64B64, R64G64B64A64

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
        PixelFormat::BC6H_ufloat_block => decode_bc6(reader, width, height, &mut pixels, false).unwrap(),
        PixelFormat::BC6H_sfloat_block => decode_bc6(reader, width, height, &mut pixels, true).unwrap(),
        PixelFormat::BC7_unorm_block => decode_bc7(reader, width, height, &mut pixels).unwrap(),
        PixelFormat::BC7_srgb_block => decode_bc7(reader, width, height, &mut pixels).unwrap(),
        _ => return Err(anyhow::anyhow!("Unsupported format: {:?}", format))
    };

    let mut x = 0;
    let mut y = 0;

    for pixel in pixels {
        let r = (pixel & 0xFF) as u8;
        let g = ((pixel >> 8) & 0xFF) as u8;
        let b = ((pixel >> 16) & 0xFF) as u8;
        let a = ((pixel >> 24) & 0xFF) as u8;

        image.put_pixel(x, y, image::Rgba([b, g, r, a]));

        x += 1;

        if x == width as u32 {
            x = 0;
            y += 1;
        }
    }

    Ok(image)
}