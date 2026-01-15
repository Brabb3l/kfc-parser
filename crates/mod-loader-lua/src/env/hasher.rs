use crc::Crc;
use kfc::hash::fnv_bytes;

use crate::{env::buffer::Buffer, lua::{Either, FunctionArgs, LuaValue}};

pub fn create(
    lua: &mlua::Lua
) -> mlua::Result<mlua::Table> {
    let table = lua.create_table()?;

    table.raw_set("fnv1a32", lua.create_function(lua_fnv1a32)?)?;
    table.raw_set("crc32", lua.create_function(lua_crc32)?)?;
    table.raw_set("crc64", lua.create_function(lua_crc64)?)?;

    Ok(table)
}

fn lua_fnv1a32(
    _: &mlua::Lua,
    args: FunctionArgs
) -> mlua::Result<u32> {
    let value = args.get::<Either<&[u8], &Buffer>>(0)?;

    let hash = match value {
        Either::A(s) => fnv_bytes(&s),
        Either::B(buf) => fnv_bytes(buf.data()?),
    };

    Ok(hash)
}

fn lua_crc32(
    _: &mlua::Lua,
    args: FunctionArgs
) -> mlua::Result<u32> {
    let value = args.get::<Either<&[u8], &Buffer>>(0)?;
    let crc = Crc::<u32>::new(&crc::CRC_32_ISO_HDLC);

    let hash = match value {
        Either::A(s) => crc.checksum(&s),
        Either::B(buf) => crc.checksum(buf.data()?),
    };

    Ok(hash)
}

fn lua_crc64(
    _: &mlua::Lua,
    args: FunctionArgs
) -> mlua::Result<LuaValue> {
    let value = args.get::<Either<&[u8], &Buffer>>(0)?;
    let crc = Crc::<u64>::new(&crc::CRC_64_ECMA_182);

    let hash = match value {
        Either::A(s) => crc.checksum(&s),
        Either::B(buf) => crc.checksum(buf.data()?),
    };

    Ok(LuaValue::Integer(hash as i64))
}
