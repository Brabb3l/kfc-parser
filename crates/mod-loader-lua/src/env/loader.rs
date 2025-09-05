use crate::{env::{util::add_function, AppFeatures, AppState}, lua::FunctionArgs};

pub fn create(
    lua: &mlua::Lua
) -> mlua::Result<mlua::Table> {
    let app_state = lua.app_data_ref::<AppState>().unwrap();
    let table = lua.create_table()?;

    add_function(lua, &table, "has_mod", lua_has_mod)?;

    table.raw_set("is_client", app_state.is_client())?;
    table.raw_set("is_server", app_state.is_server())?;

    let feature_table = lua.create_table()?;

    feature_table.raw_set("patch", app_state.has_feature(AppFeatures::PATCH))?;
    feature_table.raw_set("export", app_state.has_feature(AppFeatures::EXPORT))?;

    table.raw_set("features", feature_table)?;

    Ok(table)
}

fn lua_has_mod(
    lua: &mlua::Lua,
    args: FunctionArgs,
) -> mlua::Result<bool> {
    let app_state = lua.app_data_ref::<AppState>().unwrap();
    let id = args.get::<String>(0)?;

    Ok(app_state.env().mod_registry().contains_key(&id))
}
