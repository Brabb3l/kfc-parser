use kfc::guid::ContentHash;
use mlua::ObjectLike;

use crate::{env::util::add_function, lua::{CastLuaExt, FunctionArgs, LuaError, LuaValue}};

pub fn create(
    lua: &mlua::Lua
) -> mlua::Result<mlua::Table> {
    let table = lua.create_table()?;

    table.raw_set("NONE", ContentHash::NONE.to_string())?;

    add_function(lua, &table, "from_content_hash", lua_from_content_hash)?;
    add_function(lua, &table, "to_content_hash", lua_to_content_hash)?;
    add_function(lua, &table, "hash", lua_hash)?;

    Ok(table)
}

fn lua_from_content_hash(
    lua: &mlua::Lua,
    args: FunctionArgs,
) -> mlua::Result<LuaValue> {
    let value = args.get::<LuaValue>(0)?;

    match value {
        LuaValue::UserData(ud) => {
            let size = ud.get::<LuaValue>("size")?.cast::<u32>()?;
            let hash0 = ud.get::<LuaValue>("hash0")?.cast::<u32>()?;
            let hash1 = ud.get::<LuaValue>("hash1")?.cast::<u32>()?;
            let hash2 = ud.get::<LuaValue>("hash2")?.cast::<u32>()?;

            let guid = ContentHash::new(size, hash0, hash1, hash2);
            let guid_str = guid.to_string();

            Ok(LuaValue::String(lua.create_string(&guid_str)?))
        },
        LuaValue::Table(table) => {
            let size = table.get::<LuaValue>("size")?.cast::<u32>()?;
            let hash0 = table.get::<LuaValue>("hash0")?.cast::<u32>()?;
            let hash1 = table.get::<LuaValue>("hash1")?.cast::<u32>()?;
            let hash2 = table.get::<LuaValue>("hash2")?.cast::<u32>()?;

            let guid = ContentHash::new(size, hash0, hash1, hash2);
            let guid_str = guid.to_string();

            Ok(LuaValue::String(lua.create_string(&guid_str)?))
        },
        _ => Err(LuaError::bad_argument_type(
            0,
            "userdata or table",
            value
        )),
    }
}

fn lua_to_content_hash(
    lua: &mlua::Lua,
    args: FunctionArgs,
) -> mlua::Result<LuaValue> {
    let guid = args.get::<String>(0)?;
    let guid = ContentHash::parse(&guid)
        .ok_or_else(|| LuaError::generic("invalid GUID format"))?;
    let table = lua.create_table()?;

    table.raw_set("size", guid.size())?;
    table.raw_set("hash0", guid.hash0())?;
    table.raw_set("hash1", guid.hash1())?;
    table.raw_set("hash2", guid.hash2())?;

    Ok(LuaValue::Table(table))
}

fn lua_hash(
    _lua: &mlua::Lua,
    args: FunctionArgs,
) -> mlua::Result<u32> {
    let guid = args.get::<String>(0)?;
    let guid = ContentHash::parse(&guid)
        .ok_or_else(|| LuaError::generic("invalid GUID format"))?;

    Ok(guid.hash32())
}
