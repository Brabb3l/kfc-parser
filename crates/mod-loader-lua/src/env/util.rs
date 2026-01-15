use std::borrow::{Borrow, Cow};

use kfc::{guid::{ContentHash, Guid}, reflection::{LookupKey, TypeHandle, TypeRegistry}};
use mlua::ObjectLike;
use mod_loader::Mod;

use crate::{env::{game::{Content, Resource}, Type}, lua::{Args, CastLua, CastLuaExt, Either3, FunctionArgs, LuaError, LuaValue}};

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

impl<'a> CastLua<'a> for Guid {
    type Output = Self;

    fn try_cast_lua(
        value: &'a LuaValue,
    ) -> mlua::Result<Option<Self::Output>> {
        match value {
            LuaValue::String(s) => {
                let s = s.to_str()?;
                let guid = match Self::parse(&s) {
                    Some(v) => v,
                    None => return Ok(None),
                };

                Ok(Some(guid))
            },
            LuaValue::UserData(ud) => {
                if ud.is::<Content>() {
                    let content = ud.borrow::<Content>()?;

                    return Ok(Some(content.guid().into_guid()));
                } else if ud.is::<Resource>() {
                    let resource = ud.borrow::<Resource>()?;

                    return Ok(Some(resource.info().resource_id.guid()));
                }

                Ok(None)
            },
            _ => Ok(None),
        }
    }

    fn name() -> Cow<'static, str> {
        "guid".into()
    }
}

impl<'a> CastLua<'a> for ContentHash {
    type Output = Self;

    fn try_cast_lua(
        value: &'a LuaValue,
    ) -> mlua::Result<Option<Self::Output>> {
        macro_rules! some_or_return {
            ($e:expr) => {
                match $e {
                    Some(v) => v,
                    None => return Ok(None),
                }
            };
        }

        match value {
            LuaValue::UserData(ud) => {
                let size = ud.get::<LuaValue>("size")?.try_cast::<u32>()?;
                let hash0 = ud.get::<LuaValue>("hash0")?.try_cast::<u32>()?;
                let hash1 = ud.get::<LuaValue>("hash1")?.try_cast::<u32>()?;
                let hash2 = ud.get::<LuaValue>("hash2")?.try_cast::<u32>()?;

                let size = some_or_return!(size);
                let hash0 = some_or_return!(hash0);
                let hash1 = some_or_return!(hash1);
                let hash2 = some_or_return!(hash2);

                Ok(Some(Self::new(size, hash0, hash1, hash2)))
            },
            LuaValue::Table(table) => {
                let size = table.get::<LuaValue>("size")?.try_cast::<u32>()?;
                let hash0 = table.get::<LuaValue>("hash0")?.try_cast::<u32>()?;
                let hash1 = table.get::<LuaValue>("hash1")?.try_cast::<u32>()?;
                let hash2 = table.get::<LuaValue>("hash2")?.try_cast::<u32>()?;

                let size = some_or_return!(size);
                let hash0 = some_or_return!(hash0);
                let hash1 = some_or_return!(hash1);
                let hash2 = some_or_return!(hash2);

                Ok(Some(Self::new(size, hash0, hash1, hash2)))
            },
            LuaValue::String(s) => {
                let s = s.to_str()?;
                let guid = Self::parse(&s);
                let guid = some_or_return!(guid);

                Ok(Some(guid))
            },
            _ => Ok(None),
        }
    }

    fn name() -> Cow<'static, str> {
        "keen::ContentHash or guid".into()
    }
}
