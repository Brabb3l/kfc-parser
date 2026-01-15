use std::io::Cursor;

use image::ImageFormat;
use kfc::content::image::PixelFormat;

use crate::{env::{buffer::Buffer, util::add_function}, lua::{Either, FunctionArgs, LuaError}};

mod value;
mod format;
mod pixel_format;

pub use value::Image;

pub fn create(
    lua: &mlua::Lua
) -> mlua::Result<mlua::Table> {
    let table = lua.create_table()?;

    add_function(lua, &table, "create", lua_create)?;
    add_function(lua, &table, "decode", lua_decode)?;
    add_function(lua, &table, "decode_texture", lua_decode_texture)?;
    add_function(lua, &table, "encode", lua_encode)?;
    add_function(lua, &table, "encode_texture", lua_encode_texture)?;

    Ok(table)
}

fn lua_create(
    _: &mlua::Lua,
    args: FunctionArgs,
) -> mlua::Result<Image> {
    let width = args.get::<u32>(0)?;
    let height = args.get::<u32>(1)?;

    Ok(Image::new(width, height))
}

fn lua_decode(
    _: &mlua::Lua,
    args: FunctionArgs,
) -> mlua::Result<Image> {
    let buffer = args.get::<Either<&[u8], &Buffer>>(0)?;
    let buffer = match &buffer {
        Either::A(slice) => slice.as_ref(),
        Either::B(buf) => buf.data()?,
    };
    let format = args.get::<Option<ImageFormat>>(1)?;

    let image = match format {
        Some(format) => image::load_from_memory_with_format(buffer, format)
            .map_err(|e| LuaError::generic(format!(
                "failed to decode image: {e}",
            )))?
            .to_rgba8(),
        None => image::load_from_memory(buffer)
            .map_err(|e| LuaError::generic(format!(
                "failed to decode image: {e}",
            )))?
            .to_rgba8(),
    };

    Ok(image.into())
}

fn lua_decode_texture(
    _: &mlua::Lua,
    args: FunctionArgs,
) -> mlua::Result<Image> {
    let buffer = args.get::<Either<&[u8], &Buffer>>(0)?;
    let buffer = match &buffer {
        Either::A(slice) => slice.as_ref(),
        Either::B(buf) => buf.data()?,
    };
    let format = args.get::<PixelFormat>(1)?;
    let width = args.get::<u32>(2)?;
    let height = args.get::<u32>(3)?;
    let _mip_level = args.get::<Option<u32>>(4)?.unwrap_or(1);

    let mut dst = vec![0u8; (width * height * 4) as usize];

    kfc::content::image::decode(
        format,
        width as usize,
        height as usize,
        buffer,
        &mut dst,
    ).unwrap();

    Image::from_vec(
        dst,
        width,
        height,
    )
}

fn lua_encode(
    _: &mlua::Lua,
    args: FunctionArgs,
) -> mlua::Result<Buffer> {
    let image = args.get::<&Image>(0)?;
    let format = args.get::<Option<ImageFormat>>(1)?
        .unwrap_or(ImageFormat::Png);

    let mut encoded = Cursor::new(Vec::new());

    image.inner().write_to(&mut encoded, format)
        .map_err(|e| LuaError::generic(format!(
            "failed to encode image: {e}",
        )))?;

    Ok(Buffer::wrap(encoded.into_inner()))
}

fn lua_encode_texture(
    _: &mlua::Lua,
    args: FunctionArgs,
) -> mlua::Result<Buffer> {
    let image = args.get::<&Image>(0)?;
    let format = args.get::<PixelFormat>(1)?;

    let width = image.width() as usize;
    let height = image.height() as usize;

    let required_bytes = kfc::content::image::size_of_format(
        format,
        width,
        height,
    );
    let mut encoded = vec![0u8; required_bytes];

    kfc::content::image::encode(
        format,
        width,
        height,
        image.inner(),
        &mut encoded
    ).map_err(|e| LuaError::generic(format!(
        "failed to encode image: {e}",
    )))?;

    Ok(Buffer::wrap(encoded))
}
