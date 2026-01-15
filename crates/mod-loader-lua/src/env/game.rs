use crate::env::AppState;

mod assets;
mod guid;
mod types;

pub use types::Type;
pub use assets::{Content, Resource, value};

pub fn create(
    lua: &mlua::Lua
) -> mlua::Result<mlua::Table> {
    let table = lua.create_table()?;
    let app_state = lua.app_data_ref::<AppState>().unwrap();

    table.raw_set("version", app_state.kfc_file().game_version())?;
    table.raw_set("assets", assets::create(lua)?)?;
    table.raw_set("types", types::create(lua)?)?;
    table.raw_set("guid", guid::create(lua)?)?;

    Ok(table)
}
