use image::ImageFormat;

use crate::lua::{CastLua, LuaValue};

impl CastLua<'_> for ImageFormat {
    type Output = Self;

    fn try_cast_lua(
        value: &LuaValue
    ) -> mlua::Result<Option<Self::Output>> {
        match &value {
            LuaValue::String(s) => {
                let bytes = s.as_bytes();
                let s = match str::from_utf8(bytes.as_ref()) {
                    Ok(s) => s,
                    Err(_) => return Ok(None),
                };

                match s {
                    "png" => Ok(Some(Self::Png)),
                    "jpeg" | "jpg" => Ok(Some(Self::Jpeg)),
                    "gif" => Ok(Some(Self::Gif)),
                    "webp" => Ok(Some(Self::WebP)),
                    "pnm" => Ok(Some(Self::Pnm)),
                    "tiff" => Ok(Some(Self::Tiff)),
                    "tga" => Ok(Some(Self::Tga)),
                    "dds" => Ok(Some(Self::Dds)),
                    "bmp" => Ok(Some(Self::Bmp)),
                    "ico" => Ok(Some(Self::Ico)),
                    "hdr" => Ok(Some(Self::Hdr)),
                    "openexr" => Ok(Some(Self::OpenExr)),
                    "farbfeld" => Ok(Some(Self::Farbfeld)),
                    "avif" => Ok(Some(Self::Avif)),
                    "qoi" => Ok(Some(Self::Qoi)),
                    "pcx" => Ok(Some(Self::Pcx)),
                    _ => Ok(None),
                }
            },
            _ => Ok(None),
        }
    }

    fn name() -> std::borrow::Cow<'static, str> {
        "'png', 'jpeg', 'gif', 'webp', 'pnm', 'tiff', 'tga', 'dds', 'bmp', 'ico', 'hdr', 'openexr', 'farbfeld', 'avif', 'qoi' or 'pcx'".into()
    }
}
