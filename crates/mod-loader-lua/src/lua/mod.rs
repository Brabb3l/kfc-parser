use std::{cell::RefCell, ops::Deref};

use mlua::{MetaMethod, Table};

mod cast;
mod args;
mod error;

pub use cast::*;
pub use args::*;
pub use error::*;

pub type LuaValue = mlua::Value;

#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct LuaVM(mlua::Lua);

impl LuaVM {

    pub fn new() -> mlua::Result<Self> {
        let lua = mlua::Lua::new_with(
            mlua::StdLib::COROUTINE |
            mlua::StdLib::TABLE |
            mlua::StdLib::STRING |
            mlua::StdLib::UTF8 |
            mlua::StdLib::MATH,
            mlua::LuaOptions::default(),
        )?;

        let globals = lua.globals();

        globals.raw_remove("load")?;
        globals.raw_remove("dofile")?;
        globals.raw_remove("loadfile")?;

        register_warn_function(&lua);

        Ok(Self(lua))
    }

    pub fn create_base_environment(&self) -> mlua::Result<Table> {
        let env = self.create_table()?;
        let globals = self.globals();

        env.raw_set("utf8", self.copy_table(&globals.raw_get::<Table>("utf8")?)?)?; // table
        env.raw_set("math", self.copy_table(&globals.raw_get::<Table>("math")?)?)?; // table
        env.raw_set("string", self.copy_table(&globals.raw_get::<Table>("string")?)?)?; // table
        env.raw_set("table", self.copy_table(&globals.raw_get::<Table>("table")?)?)?; // table
        env.raw_set("coroutine", self.copy_table(&globals.raw_get::<Table>("coroutine")?)?)?; // table

        env.raw_set("collectgarbage", globals.raw_get::<LuaValue>("collectgarbage")?)?;
        env.raw_set("type", globals.raw_get::<LuaValue>("type")?)?;

        env.raw_set("rawlen", globals.raw_get::<LuaValue>("rawlen")?)?;
        env.raw_set("rawget", globals.raw_get::<LuaValue>("rawget")?)?;
        env.raw_set("rawset", globals.raw_get::<LuaValue>("rawset")?)?;
        env.raw_set("rawequal", globals.raw_get::<LuaValue>("rawequal")?)?;

        env.raw_set("assert", globals.raw_get::<LuaValue>("assert")?)?;
        env.raw_set("pcall", globals.raw_get::<LuaValue>("pcall")?)?;
        env.raw_set("xpcall", globals.raw_get::<LuaValue>("xpcall")?)?;

        env.raw_set("setmetatable", globals.raw_get::<LuaValue>("setmetatable")?)?;
        env.raw_set("getmetatable", globals.raw_get::<LuaValue>("getmetatable")?)?;

        env.raw_set("error", globals.raw_get::<LuaValue>("error")?)?;
        // env.raw_set("warn", globals.raw_get::<LuaValue>("warn")?)?;
        // env.raw_set("print", globals.raw_get::<LuaValue>("print")?)?;

        env.raw_set("ipairs", globals.raw_get::<LuaValue>("ipairs")?)?;
        env.raw_set("pairs", globals.raw_get::<LuaValue>("pairs")?)?;
        env.raw_set("next", globals.raw_get::<LuaValue>("next")?)?;
        env.raw_set("select", globals.raw_get::<LuaValue>("select")?)?;

        env.raw_set("tostring", globals.raw_get::<LuaValue>("tostring")?)?;
        env.raw_set("tonumber", globals.raw_get::<LuaValue>("tonumber")?)?;

        env.raw_set("_G", env.clone())?;

        // typeof uses __name or __type instead of returning "userdata"
        // falls back to the default type function if it is not userdata
        let type_fn = env.raw_get::<LuaValue>("type")?
            .as_function()
            .expect("type is not a function")
            .clone();

        env.raw_set(
            "typeof",
            self.create_function({
                move |_, value: LuaValue| {
                    if let LuaValue::UserData(data) = &value {
                        let r#type = data.metatable()?.get::<LuaValue>(MetaMethod::Type)?;

                        // TODO: check for function as well
                        if let LuaValue::String(_) = r#type {
                            return Ok(r#type)
                        }
                    }

                    type_fn.call(value)
                }
            })?
        )?;

        Ok(env)
    }

    fn copy_table(&self, from: &Table) -> mlua::Result<Table> {
        let to = self.create_table()?;

        for entry in from.pairs::<LuaValue, LuaValue>() {
            let (key, value) = entry?;

            if let LuaValue::Table(table) = value {
                let new_table = self.copy_table(&table)?;
                to.raw_set(key, new_table)?;
            } else {
                to.raw_set(key, value)?;
            }
        }

        Ok(to)
    }

}

impl Deref for LuaVM {
    type Target = mlua::Lua;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

fn register_warn_function(
    lua: &mlua::Lua,
) {
    lua.set_warning_function({
        let output = RefCell::new(String::new());

        move |_, message, r#continue| {
            let mut output = output.borrow_mut();

            if !output.is_empty() {
                output.push('\t');
            }

            output.push_str(message);

            if !r#continue {
                tracing::warn_span!("lua").in_scope(|| {
                    tracing::warn!("{}", output);
                });

                output.clear();
            }

            Ok(())
        }
    });
}

