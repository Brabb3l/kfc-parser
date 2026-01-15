use std::any::TypeId;
use kfc::{resource::value::Value, reflection::{PrimitiveType, TypeIndex}};
use mlua::AnyUserData;

use crate::{alias::{MappedValue, TypeHandle}, env::value::{converter::{value_to_lua, Converter, LuaConversionError}, mapped::{MappedArrayValue, MappedStructValue, MappedVariantValue}, simple::{ArrayValue, StructValue, VariantValue}, validator::{LuaValidationError, Validator}}, lua::LuaValue};

mod converter;
mod validator;
mod mapped;
mod simple;
mod util;

pub fn convert_lua_to_value(
    lua_value: &LuaValue,
    r#type: &TypeHandle,
) -> Result<Value, LuaConversionError> {
    Converter::convert(lua_value, r#type)
}

pub fn convert_value_to_lua(
    value: &MappedValue,
    lua: &mlua::Lua,
) -> mlua::Result<LuaValue> {
    value_to_lua(value, lua)
}

pub fn validate_and_clone_lua_value(
    lua_value: &LuaValue,
    r#type: &TypeHandle,
    lua: &mlua::Lua,
) -> Result<LuaValue, LuaValidationError> {
    Validator::validate_and_clone(lua_value, r#type, lua)
}

pub fn is_dirty_lua_value(
    lua_value: &LuaValue,
) -> mlua::Result<bool> {
    match lua_value {
        LuaValue::UserData(ud) => {
            let type_id = match ud.type_id() {
                Some(type_id) => type_id,
                None => panic!("UserData has no type_id"),
            };

            match type_id {
                id if id == TypeId::of::<StructValue>() => {
                    ud.borrow::<StructValue>()?.is_dirty()
                },
                id if id == TypeId::of::<MappedStructValue>() => {
                    ud.borrow::<MappedStructValue>()?.is_dirty()
                },
                id if id == TypeId::of::<ArrayValue>() => {
                    ud.borrow::<ArrayValue>()?.is_dirty()
                }
                id if id == TypeId::of::<MappedArrayValue>() => {
                    ud.borrow::<MappedArrayValue>()?.is_dirty()
                }
                id if id == TypeId::of::<VariantValue>() => {
                    ud.borrow::<VariantValue>()?.is_dirty()
                },
                id if id == TypeId::of::<MappedVariantValue>() => {
                    ud.borrow::<MappedVariantValue>()?.is_dirty()
                },
                _ => Ok(false)
            }
        },
        _ => Ok(false)
    }
}

pub fn push(
    ud: &AnyUserData,
    value: &LuaValue,
    lua: &mlua::Lua,
) -> mlua::Result<bool> {
    let type_id = match ud.type_id() {
        Some(type_id) => type_id,
        None => panic!("UserData has no type_id"),
    };

    match type_id {
        id if id == TypeId::of::<ArrayValue>() => {
            let mut array_value = ud.borrow_mut::<ArrayValue>()?;
            let len = array_value.len();

            array_value.insert(len + 1, value, lua)
                .map(|_| true)
        },
        id if id == TypeId::of::<MappedArrayValue>() => {
            let mut mapped_array_value = ud.borrow_mut::<MappedArrayValue>()?;
            let len = mapped_array_value.len();

            mapped_array_value.insert(len + 1, value, lua)
                .map(|_| true)
        },
        _ => Ok(false)
    }
}

pub fn insert(
    ud: &AnyUserData,
    pos: usize,
    value: &LuaValue,
    lua: &mlua::Lua,
) -> mlua::Result<bool> {
    let type_id = match ud.type_id() {
        Some(type_id) => type_id,
        None => panic!("UserData has no type_id"),
    };

    match type_id {
        id if id == TypeId::of::<ArrayValue>() => {
            ud.borrow_mut::<ArrayValue>()?.insert(pos, value, lua)
                .map(|_| true)
        },
        id if id == TypeId::of::<MappedArrayValue>() => {
            ud.borrow_mut::<MappedArrayValue>()?.insert(pos, value, lua)
                .map(|_| true)
        },
        _ => Ok(false)
    }
}

pub fn pop(
    ud: &AnyUserData,
    lua: &mlua::Lua,
) -> mlua::Result<(Option<LuaValue>, bool)> {
    let type_id = match ud.type_id() {
        Some(type_id) => type_id,
        None => panic!("UserData has no type_id"),
    };

    match type_id {
        id if id == TypeId::of::<ArrayValue>() => {
            let mut array_value = ud.borrow_mut::<ArrayValue>()?;
            let len = array_value.len();

            array_value.remove(len)
                .map(|value| (value, true))
        },
        id if id == TypeId::of::<MappedArrayValue>() => {
            let mut mapped_array_value = ud.borrow_mut::<MappedArrayValue>()?;
            let len = mapped_array_value.len();

            mapped_array_value.remove(len, lua)
                .map(|value| (value, true))
        },
        _ => Ok((None, false))
    }
}

pub fn remove(
    ud: &AnyUserData,
    pos: usize,
    lua: &mlua::Lua,
) -> mlua::Result<(Option<LuaValue>, bool)> {
    let type_id = match ud.type_id() {
        Some(type_id) => type_id,
        None => panic!("UserData has no type_id"),
    };

    match type_id {
        id if id == TypeId::of::<ArrayValue>() => {
            ud.borrow_mut::<ArrayValue>()?.remove(pos)
                .map(|value| (value, true))
        },
        id if id == TypeId::of::<MappedArrayValue>() => {
            ud.borrow_mut::<MappedArrayValue>()?.remove(pos, lua)
                .map(|value| (value, true))
        },
        _ => Ok((None, false))
    }
}

pub fn clear(
    ud: &AnyUserData,
) -> mlua::Result<bool> {
    let type_id = match ud.type_id() {
        Some(type_id) => type_id,
        None => panic!("UserData has no type_id"),
    };

    match type_id {
        id if id == TypeId::of::<ArrayValue>() => {
            ud.borrow_mut::<ArrayValue>()?.clear()
                .map(|_| true)
        },
        id if id == TypeId::of::<MappedArrayValue>() => {
            ud.borrow_mut::<MappedArrayValue>()?.clear()
                .map(|_| true)
        },
        _ => Ok(false)
    }
}

pub fn type_of(
    lua_value: &LuaValue,
) -> mlua::Result<Option<TypeIndex>> {
    match lua_value {
        LuaValue::UserData(ud) => {
            let type_id = match ud.type_id() {
                Some(type_id) => type_id,
                None => panic!("UserData has no type_id"),
            };

            match type_id {
                id if id == TypeId::of::<StructValue>() => {
                    let struct_value = ud.borrow::<StructValue>()?;
                    let r#type = struct_value.r#type();

                    Ok(Some(r#type.index()))
                },
                id if id == TypeId::of::<MappedStructValue>() => {
                    let mapped_struct_value = ud.borrow::<MappedStructValue>()?;
                    let r#type = mapped_struct_value.r#type();

                    Ok(Some(r#type.index()))
                },
                id if id == TypeId::of::<ArrayValue>() => {
                    let array_value = ud.borrow::<ArrayValue>()?;
                    let r#type = array_value.r#type();

                    Ok(Some(r#type.index()))
                }
                id if id == TypeId::of::<MappedArrayValue>() => {
                    let mapped_array_value = ud.borrow::<MappedArrayValue>()?;
                    let r#type = mapped_array_value.r#type();

                    Ok(Some(r#type.index()))
                }
                id if id == TypeId::of::<VariantValue>() => {
                    let variant_value = ud.borrow::<VariantValue>()?;
                    let r#type = variant_value.r#type();

                    Ok(Some(r#type.index()))
                },
                id if id == TypeId::of::<MappedVariantValue>() => {
                    let mapped_variant_value = ud.borrow::<MappedVariantValue>()?;
                    let r#type = mapped_variant_value.r#type();

                    Ok(Some(r#type.index()))
                },
                _ => Ok(None)
            }
        },
        _ => Ok(None)
    }
}

fn name_of(
    r#type: &TypeHandle,
) -> String {
    if r#type.primitive_type == PrimitiveType::Typedef {
        name_of(
            &r#type.inner_type()
                .expect("invalid typedef type"),
        )
    } else {
        r#type.qualified_name.clone()
    }
}
