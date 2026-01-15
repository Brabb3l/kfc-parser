use mlua::Table;
use mod_loader::Mod;

mod game;
mod integer;
mod log;
mod table;
mod hasher;
mod io;
mod buffer;
mod loader;
mod image;

mod util;
mod app_state;

pub use app_state::*;

#[allow(unused_imports)]
use game::{Content, Resource, Type, value};
use buffer::Buffer;

pub fn register(
    lua: &mlua::Lua,
    env: &Table,
    r#mod: &Mod,
) -> mlua::Result<()> {
    let lua_builtin = lua.create_table()?;

    let lua_io = io::create(lua, r#mod.clone())?;
    let lua_integer = integer::create(lua)?;
    let lua_game = game::create(lua)?;
    let lua_buffer = buffer::create(lua)?;
    let lua_hasher = hasher::create(lua)?;
    let lua_loader = loader::create(lua)?;
    let lua_image = image::create(lua)?;

    lua_builtin.raw_set("io", lua_io.clone())?;
    lua_builtin.raw_set("integer", lua_integer.clone())?;
    lua_builtin.raw_set("game", lua_game.clone())?;
    lua_builtin.raw_set("buffer", lua_buffer.clone())?;
    lua_builtin.raw_set("hasher", lua_hasher.clone())?;
    lua_builtin.raw_set("loader", lua_loader.clone())?;
    lua_builtin.raw_set("image", lua_image.clone())?;

    env.raw_set("builtin", lua_builtin)?;
    env.raw_set("io", lua_io)?;
    env.raw_set("integer", lua_integer)?;
    env.raw_set("game", lua_game)?;
    env.raw_set("buffer", lua_buffer)?;
    env.raw_set("hasher", lua_hasher)?;
    env.raw_set("loader", lua_loader)?;
    env.raw_set("image", lua_image)?;

    table::modify(&env.raw_get::<Table>("table")?, lua)?;
    log::register(lua, env, &r#mod.info().id)?;

    Ok(())
}
