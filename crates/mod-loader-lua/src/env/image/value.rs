use image::RgbaImage;
use mlua::{IntoLuaMulti, UserData};

use crate::lua::{LuaError, LuaValue, MethodArgs};

pub struct Image {
    inner: RgbaImage,
}

impl Image {

    #[inline]
    pub fn new(
        width: u32,
        height: u32,
    ) -> Self {
        Self {
            inner: RgbaImage::new(width, height)
        }
    }

    pub fn from_vec(
        vec: Vec<u8>,
        width: u32,
        height: u32,
    ) -> mlua::Result<Self> {
        RgbaImage::from_vec(width, height, vec)
            .map(|inner| Self { inner })
            .ok_or_else(|| LuaError::generic("failed to create image"))
    }

    #[inline]
    pub fn width(&self) -> u32 {
        self.inner.width()
    }

    #[inline]
    pub fn height(&self) -> u32 {
        self.inner.height()
    }

    #[inline]
    pub(super) fn inner(&self) -> &RgbaImage {
        &self.inner
    }

}

impl From<RgbaImage> for Image {
    fn from(value: RgbaImage) -> Self {
        Self { inner: value }
    }
}

impl UserData for Image {

    fn add_fields<F: mlua::UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("width", |_, this| {
            Ok(this.inner.width())
        });
        fields.add_field_method_get("height", |_, this| {
            Ok(this.inner.height())
        });
    }

    fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
        methods.add_function("get_pixel", lua_get_pixel);
        methods.add_function("set_pixel", lua_set_pixel);
        methods.add_function("get_pixel_packed", lua_get_pixel_packed);
        methods.add_function("set_pixel_packed", lua_set_pixel_packed);
    }

}

fn lua_get_pixel(
    lua: &mlua::Lua,
    args: MethodArgs,
) -> mlua::Result<mlua::MultiValue> {
    let this = args.this::<&Image>()?;
    let x = args.get::<u32>(0)?;
    let y = args.get::<u32>(1)?;

    if x >= this.inner.width() || y >= this.inner.height() {
        return LuaValue::Nil.into_lua_multi(lua);
    }

    let pixel = this.inner.get_pixel(x, y);
    let r = pixel[0];
    let g = pixel[1];
    let b = pixel[2];
    let a = pixel[3];

    (r, g, b, a).into_lua_multi(lua)
}

fn lua_set_pixel(
    _: &mlua::Lua,
    args: MethodArgs,
) -> mlua::Result<()> {
    let mut this = args.this::<&mut Image>()?;
    let x = args.get::<u32>(0)?;
    let y = args.get::<u32>(1)?;
    let r = args.get::<u8>(2)?;
    let g = args.get::<u8>(3)?;
    let b = args.get::<u8>(4)?;
    let a = args.get::<u8>(5)?;

    if x >= this.inner.width() {
        return Err(LuaError::position_out_of_bounds_named(
            "x",
            x as usize,
            0,
            this.inner.width() as usize - 1,
        ));
    }

    if y >= this.inner.height() {
        return Err(LuaError::position_out_of_bounds_named(
            "y",
            y as usize,
            0,
            this.inner.height() as usize - 1,
        ));
    }

    this.inner.put_pixel(x, y, image::Rgba([r, g, b, a]));

    Ok(())
}

fn lua_get_pixel_packed(
    _: &mlua::Lua,
    args: MethodArgs,
) -> mlua::Result<LuaValue> {
    let this = args.this::<&Image>()?;
    let x = args.get::<u32>(0)?;
    let y = args.get::<u32>(1)?;

    if x >= this.inner.width() || y >= this.inner.height() {
        return Ok(LuaValue::Nil);
    }

    let pixel = this.inner.get_pixel(x, y);
    let value = u32::from_le_bytes([pixel[0], pixel[1], pixel[2], pixel[3]]);

    Ok(LuaValue::Integer(value as i64))

}

fn lua_set_pixel_packed(
    _: &mlua::Lua,
    args: MethodArgs,
) -> mlua::Result<()> {
    let mut this = args.this::<&mut Image>()?;
    let x = args.get::<u32>(0)?;
    let y = args.get::<u32>(1)?;
    let value = args.get::<u32>(2)?;

    if x >= this.inner.width() {
        return Err(LuaError::position_out_of_bounds_named(
            "x",
            x as usize,
            0,
            this.inner.width() as usize - 1,
        ));
    }

    if y >= this.inner.height() {
        return Err(LuaError::position_out_of_bounds_named(
            "y",
            y as usize,
            0,
            this.inner.height() as usize - 1,
        ));
    }

    let bytes = value.to_le_bytes();
    this.inner.put_pixel(x, y, image::Rgba([bytes[0], bytes[1], bytes[2], bytes[3]]));

    Ok(())
}
