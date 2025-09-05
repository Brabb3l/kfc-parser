use mlua::{AnyUserData, Table};

use crate::{env::value::{clear as clear_ud, insert as insert_ud, pop as pop_ud, push as push_ud, remove as remove_ud}, lua::{Either, FunctionArgs, LuaValue}};

#[allow(clippy::get_first)]
pub fn modify(
    table: &mlua::Table,
    lua: &mlua::Lua,
) -> mlua::Result<()> {
    let insert = table.get::<mlua::Function>("insert")?;
    let remove = table.get::<mlua::Function>("remove")?;
    // TODO: add `concat` support for `ArrayValue` and `MappedArrayValue`
    // TODO: add `sort` support for `ArrayValue` and `MappedArrayValue`
    // TODO: add `move` support for `ArrayValue` and `MappedArrayValue`
    // TODO: add `unpack` support for `ArrayValue` and `MappedArrayValue`

    table.set("insert", lua.create_function(move |lua, args: FunctionArgs| {
        match args.len() {
            2 => {
                let list = args.get::<LuaValue>(0)?;

                if let LuaValue::UserData(ud) = list {
                    let value = args.get::<LuaValue>(1)?;

                    if push_ud(ud, value, lua)? {
                        return Ok(LuaValue::Nil);
                    }
                }

                insert.call::<LuaValue>(args)
            }
            3.. => {
                let list = args.get::<LuaValue>(0)?;

                if let LuaValue::UserData(ud) = list {
                    let pos = args.get::<usize>(1)?;
                    let value = args.get::<LuaValue>(2)?;

                    if insert_ud(ud, pos, value, lua)? {
                        return Ok(LuaValue::Nil);
                    }
                }

                insert.call::<LuaValue>(args)
            }
            _ => Err(mlua::Error::runtime("wrong number of arguments to 'insert'")),
        }
    })?)?;

    table.set("remove", lua.create_function(move |lua, args: FunctionArgs| {
        match args.len() {
            1 => {
                let list = args.get::<LuaValue>(0)?;

                if let LuaValue::UserData(ud) = list {
                    let (value, valid) = pop_ud(ud, lua)?;

                    if valid {
                        return Ok(value.unwrap_or(LuaValue::Nil));
                    }
                }

                remove.call::<LuaValue>(args)
            }
            2 => {
                let list = args.get::<LuaValue>(0)?;

                if let LuaValue::UserData(ud) = list {
                    let pos = args.get::<usize>(1)?;
                    let (value, valid) = remove_ud(ud, pos, lua)?;

                    if valid {
                        return Ok(value.unwrap_or(LuaValue::Nil));
                    }
                }

                remove.call::<LuaValue>(args)
            }
            _ => Err(mlua::Error::runtime("wrong number of arguments to 'remove'")),
        }
    })?)?;

    table.set("clear", lua.create_function(move |_, args: FunctionArgs| {
        let list = args.get::<Either<AnyUserData, Table>>(0)?;

        match list {
            Either::A(ud) => {
                if clear_ud(ud)? {
                    Ok(LuaValue::Nil)
                } else {
                    // TODO: generalize error message
                    Err(mlua::Error::runtime(
                        format!(
                            "'clear' is not supported for userdata `{:?}`",
                            args.get::<LuaValue>(0)?.type_name()
                        )
                    ))
                }
            }
            Either::B(table) => {
                table.clear()?;
                Ok(LuaValue::Nil)
            }
        }
    })?)?;

    Ok(())
}
