use std::borrow::Borrow;

use kfc::reflection::{LookupKey, TypeHandle, TypeRegistry};
use mod_loader::Mod;

use crate::{env::Type, lua::{Args, Either3, FunctionArgs, LuaError}};

pub fn get_type<T>(
    args: &impl Args,
    index: usize,
    type_registry: T,
) -> mlua::Result<TypeHandle<T>>
where
    T: Borrow<TypeRegistry> + Clone,
{
    let r#type = args.get::<Either3::<u32, String, &Type>>(index)?;
    let r#type = match r#type {
        Either3::A(type_hash) => {
            type_registry.borrow()
                .get_by_hash(LookupKey::Qualified(type_hash))
                .map(|t| TypeHandle::new(type_registry.clone(), t.index))
                .ok_or_else(|| LuaError::generic(format!("type with hash '{type_hash}' not found")))?
        }
        Either3::B(type_name) => {
            type_registry.borrow()
                .get_by_name(LookupKey::Qualified(&type_name))
                .map(|t| TypeHandle::new(type_registry.clone(), t.index))
                .ok_or_else(|| LuaError::generic(format!("type '{type_name}' not found")))?
        }
        Either3::C(type_data) => TypeHandle::new(type_registry, type_data.index)
    };

    Ok(r#type)
}

#[inline]
pub fn add_function<F, R>(
    lua: &mlua::Lua,
    table: &mlua::Table,
    name: &str,
    func: F,
) -> mlua::Result<()>
where
    F: Fn(&mlua::Lua, FunctionArgs) -> mlua::Result<R> + mlua::MaybeSend + 'static,
    R: mlua::IntoLuaMulti,
{
    let function = lua.create_function(func)?;

    table.raw_set(name, function)?;

    Ok(())
}

#[inline]
pub fn add_function_with_mod<F, R>(
    lua: &mlua::Lua,
    table: &mlua::Table,
    name: &str,
    r#mod: &Mod,
    func: F,
) -> mlua::Result<()>
where
    F: Fn(&mlua::Lua, FunctionArgs, &Mod) -> mlua::Result<R> + mlua::MaybeSend + 'static,
    R: mlua::IntoLuaMulti,
{
    let mod_clone = r#mod.clone();
    let function = lua.create_function(move |lua, args: FunctionArgs| {
        func(lua, args, &mod_clone)
    })?;

    table.raw_set(name, function)?;

    Ok(())
}

