use std::{ops::Deref, rc::Rc};

use half::f16;
use mlua::{IntoLua, MetaMethod, UserData};

use crate::{alias::MappedValue, env::{game::value::convert_lua_to_value, util::{add_function, get_type}, value::convert_value_to_lua}, lua::{FunctionArgs, LuaError, LuaValue, MethodArgs}};

use super::AppState;

mod misc;
mod read;
mod write;
mod error;
mod order;

use error::*;
use order::*;

pub fn create(
    lua: &mlua::Lua
) -> mlua::Result<mlua::Table> {
    let table = lua.create_table()?;

    add_function(lua, &table, "create", lua_create)?;
    add_function(lua, &table, "wrap", lua_wrap)?;

    Ok(table)
}

fn lua_create(
    _lua: &mlua::Lua,
    args: FunctionArgs,
) -> mlua::Result<Buffer> {
    let initial_capacity = args.get::<Option<usize>>(0)?;

    Ok(match initial_capacity {
        Some(capacity) => Buffer::with_capacity(capacity),
        None => Buffer::new(),
    })
}

fn lua_wrap(
    _lua: &mlua::Lua,
    args: FunctionArgs,
) -> mlua::Result<Buffer> {
    let data = args.get::<&[u8]>(0)?;

    Ok(Buffer::wrap(data.to_vec()))
}

#[derive(Clone)]
pub struct Buffer {
    data: Vec<u8>,
    position: usize,
    limit: usize,
    state: BufferState,
    order: ByteOrder,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BufferState {
    ReadWrite,
    Read,
    Write,
    Closed,
}

impl Buffer {

    #[inline]
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            position: 0,
            limit: 0,
            state: BufferState::ReadWrite,
            order: ByteOrder::LittleEndian,
        }
    }

    #[inline]
    pub fn with_capacity(
        capacity: usize
    ) -> Self {
        Self {
            data: vec![0; capacity],
            position: 0,
            limit: 0,
            state: BufferState::ReadWrite,
            order: ByteOrder::LittleEndian,
        }
    }

    #[inline]
    pub fn wrap(
        data: Vec<u8>
    ) -> Self {
        let limit = data.len();

        Self {
            data,
            position: 0,
            limit,
            state: BufferState::ReadWrite,
            order: ByteOrder::LittleEndian,
        }
    }

}

impl UserData for Buffer {

    fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
        methods.add_function("position", lua_position);
        methods.add_function("limit", lua_limit);
        methods.add_function("remaining", lua_remaining);
        methods.add_function("capacity", lua_capacity);
        methods.add_function("flip", lua_flip);
        methods.add_function("rewind", lua_rewind);
        methods.add_function("reset", lua_reset);
        methods.add_function("reserve", lua_reserve);
        methods.add_function("order", lua_order);
        methods.add_function("skip", lua_skip);
        methods.add_function("copy", lua_copy);
        methods.add_function("close", lua_close);

        methods.add_function("read_bool", lua_read_bool);
        methods.add_function("read_i8", lua_read_i8);
        methods.add_function("read_u8", lua_read_u8);
        methods.add_function("read_i16", lua_read_i16);
        methods.add_function("read_u16", lua_read_u16);
        methods.add_function("read_i32", lua_read_i32);
        methods.add_function("read_u32", lua_read_u32);
        methods.add_function("read_i64", lua_read_i64);
        methods.add_function("read_u64", lua_read_u64);
        methods.add_function("read_f16", lua_read_f16);
        methods.add_function("read_f32", lua_read_f32);
        methods.add_function("read_f64", lua_read_f64);
        methods.add_function("read_string", lua_read_string);
        methods.add_function("read_resource", lua_read_resource);

        methods.add_function("write_bool", lua_write_bool);
        methods.add_function("write_i8", lua_write_i8);
        methods.add_function("write_u8", lua_write_u8);
        methods.add_function("write_i16", lua_write_i16);
        methods.add_function("write_u16", lua_write_u16);
        methods.add_function("write_i32", lua_write_i32);
        methods.add_function("write_u32", lua_write_u32);
        methods.add_function("write_i64", lua_write_i64);
        methods.add_function("write_u64", lua_write_u64);
        methods.add_function("write_f16", lua_write_f16);
        methods.add_function("write_f32", lua_write_f32);
        methods.add_function("write_f64", lua_write_f64);
        methods.add_function("write_string", lua_write_string);
        methods.add_function("write_resource", lua_write_resource);

        methods.add_function("to_string", lua_to_string);

        methods.add_meta_function(MetaMethod::ToString, lua_to_string);
    }

}

fn lua_position(
    lua: &mlua::Lua,
    args: MethodArgs,
) -> mlua::Result<LuaValue> {
    let mut this = args.this::<&mut Buffer>()?;
    let position = args.get::<Option<usize>>(0)?;

    if let Some(position) = position {
        this.set_position(position)?;
        Ok(LuaValue::Nil)
    } else {
        this.position()?.into_lua(lua)
    }
}

fn lua_limit(
    lua: &mlua::Lua,
    args: MethodArgs,
) -> mlua::Result<LuaValue> {
    let mut this = args.this::<&mut Buffer>()?;
    let limit = args.get::<Option<usize>>(0)?;

    if let Some(limit) = limit {
        this.set_limit(limit)?;
        Ok(LuaValue::Nil)
    } else {
        this.limit()?.into_lua(lua)
    }
}

fn lua_remaining(
    _: &mlua::Lua,
    args: MethodArgs,
) -> mlua::Result<usize> {
    let this = args.this::<&Buffer>()?;

    Ok(this.remaining()?)
}

fn lua_capacity(
    _: &mlua::Lua,
    args: MethodArgs,
) -> mlua::Result<usize> {
    let this = args.this::<&Buffer>()?;

    Ok(this.capacity()?)
}

fn lua_flip(
    _: &mlua::Lua,
    args: MethodArgs,
) -> mlua::Result<()> {
    let mut this = args.this::<&mut Buffer>()?;

    Ok(this.flip()?)
}

fn lua_rewind(
    _: &mlua::Lua,
    args: MethodArgs,
) -> mlua::Result<()> {
    let mut this = args.this::<&mut Buffer>()?;

    Ok(this.rewind()?)
}

fn lua_reset(
    _: &mlua::Lua,
    args: MethodArgs,
) -> mlua::Result<()> {
    let mut this = args.this::<&mut Buffer>()?;

    Ok(this.reset()?)
}

fn lua_reserve(
    _: &mlua::Lua,
    args: MethodArgs
) -> mlua::Result<()> {
    let mut this = args.this::<&mut Buffer>()?;
    let additional = args.get::<usize>(0)?;

    Ok(this.reserve(additional)?)
}

fn lua_order(
    lua: &mlua::Lua,
    args: MethodArgs,
) -> mlua::Result<LuaValue> {
    let mut this = args.this::<&mut Buffer>()?;
    let order = args.get::<Option<ByteOrder>>(0)?;

    if let Some(order) = order {
        this.set_order(order)?;
        Ok(LuaValue::Nil)
    } else {
        this.order()?.into_lua(lua)
    }
}

fn lua_skip(
    _: &mlua::Lua,
    args: MethodArgs,
) -> mlua::Result<()> {
    let mut this = args.this::<&mut Buffer>()?;
    let size = args.get::<usize>(0)?;

    Ok(this.skip(size)?)
}

fn lua_copy(
    _: &mlua::Lua,
    args: MethodArgs,
) -> mlua::Result<()> {
    let this = args.this::<&Buffer>()?;
    let other = args.get::<&Buffer>(0)?;
    let offset = args.get::<Option<usize>>(1)?;
    let length = args.get::<Option<usize>>(2)?;

    let is_same = std::ptr::eq(this.deref(), other.deref());

    drop(this);
    drop(other);

    if is_same {
        let mut this = args.this::<&mut Buffer>()?;
        let offset = offset.unwrap_or(0);
        let length = length.unwrap_or(this.remaining()?);

        this.copy_within(offset, length)?;
    } else {
        let mut this = args.this::<&mut Buffer>()?;
        let other = args.get::<&Buffer>(0)?;

        let offset = offset.unwrap_or(0);
        let length = length.unwrap_or(other.remaining()?);

        other.check_not_closed()?;
        other.check_readable()?;
        other.check_read_bounds_at(offset, length)?;
        this.copy(&other.data()?[offset..offset + length])?;
    }

    Ok(())
}

fn lua_close(
    _: &mlua::Lua,
    args: MethodArgs,
) -> mlua::Result<()> {
    let mut this = args.this::<&mut Buffer>()?;

    this.close();

    Ok(())
}

fn lua_read_bool(
    _: &mlua::Lua,
    args: MethodArgs,
) -> mlua::Result<bool> {
    let mut this = args.this::<&mut Buffer>()?;

    Ok(this.read_bool()?)
}

fn lua_read_i8(
    _: &mlua::Lua,
    args: MethodArgs,
) -> mlua::Result<i8> {
    let mut this = args.this::<&mut Buffer>()?;

    Ok(this.read_i8()?)
}

fn lua_read_u8(
    _: &mlua::Lua,
    args: MethodArgs,
) -> mlua::Result<u8> {
    let mut this = args.this::<&mut Buffer>()?;

    Ok(this.read_u8()?)
}

fn lua_read_i16(
    _: &mlua::Lua,
    args: MethodArgs,
) -> mlua::Result<i16> {
    let mut this = args.this::<&mut Buffer>()?;

    Ok(this.read_i16()?)
}

fn lua_read_u16(
    _: &mlua::Lua,
    args: MethodArgs,
) -> mlua::Result<u16> {
    let mut this = args.this::<&mut Buffer>()?;

    Ok(this.read_u16()?)
}

fn lua_read_i32(
    _: &mlua::Lua,
    args: MethodArgs,
) -> mlua::Result<i32> {
    let mut this = args.this::<&mut Buffer>()?;

    Ok(this.read_i32()?)
}

fn lua_read_u32(
    _: &mlua::Lua,
    args: MethodArgs,
) -> mlua::Result<u32> {
    let mut this = args.this::<&mut Buffer>()?;

    Ok(this.read_u32()?)
}

fn lua_read_i64(
    _: &mlua::Lua,
    args: MethodArgs,
) -> mlua::Result<i64> {
    let mut this = args.this::<&mut Buffer>()?;

    Ok(this.read_i64()?)
}

fn lua_read_u64(
    _: &mlua::Lua,
    args: MethodArgs,
) -> mlua::Result<LuaValue> {
    let mut this = args.this::<&mut Buffer>()?;

    Ok(LuaValue::Integer(this.read_u64()? as i64))
}

fn lua_read_f16(
    _: &mlua::Lua,
    args: MethodArgs,
) -> mlua::Result<LuaValue> {
    let mut this = args.this::<&mut Buffer>()?;

    Ok(LuaValue::Number(this.read_f16()?.to_f64()))
}

fn lua_read_f32(
    _: &mlua::Lua,
    args: MethodArgs,
) -> mlua::Result<f32> {
    let mut this = args.this::<&mut Buffer>()?;

    Ok(this.read_f32()?)
}

fn lua_read_f64(
    _: &mlua::Lua,
    args: MethodArgs,
) -> mlua::Result<f64> {
    let mut this = args.this::<&mut Buffer>()?;

    Ok(this.read_f64()?)
}

fn lua_read_string(
    lua: &mlua::Lua,
    args: MethodArgs,
) -> mlua::Result<mlua::String> {
    let mut this = args.this::<&mut Buffer>()?;
    let size = args.get::<usize>(0)?;
    let bytes = this.read_bytes(size)?;

    lua.create_string(bytes)
}

fn lua_read_resource(
    lua: &mlua::Lua,
    args: MethodArgs,
) -> mlua::Result<LuaValue> {
    let this = args.this::<&Buffer>()?;

    this.check_not_closed()?;
    this.check_readable()?;

    let app_state = lua.app_data_ref::<AppState>().unwrap();
    let type_registry = app_state.type_registry();

    let r#type = get_type(&args, 0, type_registry.clone())?;

    let value = MappedValue::from_bytes(
        app_state.type_registry(),
        &r#type,
        &Rc::from(&this.data[this.position..this.limit])
    ).map_err(LuaError::external)?;

    convert_value_to_lua(&value, lua)
}

fn lua_write_bool(
    _: &mlua::Lua,
    args: MethodArgs,
) -> mlua::Result<()> {
    let mut this = args.this::<&mut Buffer>()?;
    let value = args.get::<bool>(0)?;

    Ok(this.write_bool(value)?)
}

fn lua_write_i8(
    _: &mlua::Lua,
    args: MethodArgs,
) -> mlua::Result<()> {
    let mut this = args.this::<&mut Buffer>()?;
    let value = args.get::<i8>(0)?;

    Ok(this.write_i8(value)?)
}

fn lua_write_u8(
    _: &mlua::Lua,
    args: MethodArgs,
) -> mlua::Result<()> {
    let mut this = args.this::<&mut Buffer>()?;
    let value = args.get::<u8>(0)?;

    Ok(this.write_u8(value)?)
}

fn lua_write_i16(
    _: &mlua::Lua,
    args: MethodArgs,
) -> mlua::Result<()> {
    let mut this = args.this::<&mut Buffer>()?;
    let value = args.get::<i16>(0)?;

    Ok(this.write_i16(value)?)
}

fn lua_write_u16(
    _: &mlua::Lua,
    args: MethodArgs,
) -> mlua::Result<()> {
    let mut this = args.this::<&mut Buffer>()?;
    let value = args.get::<u16>(0)?;

    Ok(this.write_u16(value)?)
}

fn lua_write_i32(
    _: &mlua::Lua,
    args: MethodArgs,
) -> mlua::Result<()> {
    let mut this = args.this::<&mut Buffer>()?;
    let value = args.get::<i32>(0)?;

    Ok(this.write_i32(value)?)
}

fn lua_write_u32(
    _: &mlua::Lua,
    args: MethodArgs,
) -> mlua::Result<()> {
    let mut this = args.this::<&mut Buffer>()?;
    let value = args.get::<u32>(0)?;

    Ok(this.write_u32(value)?)
}

fn lua_write_i64(
    _: &mlua::Lua,
    args: MethodArgs,
) -> mlua::Result<()> {
    let mut this = args.this::<&mut Buffer>()?;
    let value = args.get::<i64>(0)?;

    Ok(this.write_i64(value)?)
}

fn lua_write_u64(
    _: &mlua::Lua,
    args: MethodArgs,
) -> mlua::Result<()> {
    let mut this = args.this::<&mut Buffer>()?;
    let value = args.get::<u64>(0)?;

    Ok(this.write_u64(value)?)
}

fn lua_write_f16(
    _: &mlua::Lua,
    args: MethodArgs,
) -> mlua::Result<()> {
    let mut this = args.this::<&mut Buffer>()?;
    let value = args.get::<f16>(0)?;

    Ok(this.write_f16(value)?)
}

fn lua_write_f32(
    _: &mlua::Lua,
    args: MethodArgs,
) -> mlua::Result<()> {
    let mut this = args.this::<&mut Buffer>()?;
    let value = args.get::<f32>(0)?;

    Ok(this.write_f32(value)?)
}

fn lua_write_f64(
    _: &mlua::Lua,
    args: MethodArgs,
) -> mlua::Result<()> {
    let mut this = args.this::<&mut Buffer>()?;
    let value = args.get::<f64>(0)?;

    Ok(this.write_f64(value)?)
}

fn lua_write_string(
    _: &mlua::Lua,
    args: MethodArgs,
) -> mlua::Result<()> {
    let mut this = args.this::<&mut Buffer>()?;
    let value = args.get::<&[u8]>(0)?;

    Ok(this.write_bytes(&value)?)
}

fn lua_write_resource(
    lua: &mlua::Lua,
    args: MethodArgs,
) -> mlua::Result<()> {
    let mut this = args.this::<&mut Buffer>()?;

    this.check_not_closed()?;
    this.check_writable()?;

    let app_state = lua.app_data_ref::<AppState>().unwrap();
    let type_registry = app_state.type_registry();

    let r#type = get_type(&args, 0, type_registry.clone())?;
    let lua_value = args.get::<LuaValue>(1)?;

    let value = convert_lua_to_value(
        lua_value,
        &r#type
    ).map_err(LuaError::external)?;

    let bytes = value.to_bytes(
        type_registry,
        &r#type
    ).map_err(LuaError::external)?;

    this.write_bytes(&bytes)?;

    Ok(())
}

fn lua_to_string(
    lua: &mlua::Lua,
    args: MethodArgs,
) -> mlua::Result<mlua::String> {
    let this = args.this::<&Buffer>()?;

    this.check_not_closed()?;
    this.check_readable()?;

    lua.create_string(this.data()?)
}
