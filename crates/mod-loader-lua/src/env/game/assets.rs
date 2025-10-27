use kfc::{guid::{ContentHash, Guid, ResourceId}, reflection::LookupKey};
use mlua::Table;
use tracing::warn;

use crate::{env::{util::{add_function, get_type}, AppState, Buffer}, lua::{FunctionArgs, LuaError, LuaValue}};

mod resource;
mod content;
pub mod value;

pub use resource::*;
pub use content::*;

pub fn create(
    lua: &mlua::Lua
) -> mlua::Result<mlua::Table> {
    let table = lua.create_table()?;

    add_function(lua, &table, "get_resource", lua_get_resource)?;
    add_function(lua, &table, "get_resource_parts", lua_get_resource_parts)?;
    add_function(lua, &table, "get_resources_by_type", lua_get_resources_by_type)?;
    add_function(lua, &table, "get_all_resources", lua_get_all_resources)?;
    add_function(lua, &table, "get_resource_types", lua_get_resource_types)?;
    add_function(lua, &table, "create_resource", lua_create_resource)?;
    add_function(lua, &table, "get_content", lua_get_content)?;
    add_function(lua, &table, "get_all_contents", lua_get_all_contents)?;
    add_function(lua, &table, "create_content", lua_create_content)?;

    Ok(table)
}

fn lua_get_resource(
    lua: &mlua::Lua,
    args: FunctionArgs,
) -> mlua::Result<Option<Resource>> {
    let app_state = lua.app_data_ref::<AppState>().unwrap();

    let guid = args.get::<Guid>(0)?;
    let r#type = get_type(&args, 1, app_state.type_registry().as_ref())?;
    let part = args.get::<Option<u32>>(2)?;

    let guid = ResourceId::new(
        guid,
        r#type.qualified_hash,
        part.unwrap_or(0)
    );

    Ok(app_state.get_resource_info(&guid)
        .map(Resource::new))
}

fn lua_get_resource_parts(
    lua: &mlua::Lua,
    args: FunctionArgs,
) -> mlua::Result<Table> {
    let app_state = lua.app_data_ref::<AppState>().unwrap();

    let guid = args.get::<Guid>(0)?;
    let r#type = get_type(&args, 1, app_state.type_registry().as_ref())?;

    let target_guid = ResourceId::new(
        guid,
        r#type.qualified_hash,
        0
    );

    let file = app_state.kfc_file();
    let result = lua.create_table()?;

    for guid in file.resources().keys() {
        if guid.guid() != target_guid.guid() || guid.type_hash() != target_guid.type_hash() {
            continue;
        }

        let resource = match app_state.get_resource_info(guid) {
            Some(info) => Resource::new(info),
            None => {
                warn!("Resource info not found for GUID: {}", guid);
                continue;
            }
        };

        result.push(resource)?;
    }

    Ok(result)
}

fn lua_get_resources_by_type(
    lua: &mlua::Lua,
    args: FunctionArgs,
) -> mlua::Result<Table> {
    let app_state = lua.app_data_ref::<AppState>().unwrap();

    let r#type = get_type(&args, 0, app_state.type_registry().as_ref())?;

    let file = app_state.kfc_file();
    let result = lua.create_table_with_capacity(file.resources().len(), 0)?;

    for guid in file.resources_by_type(r#type.qualified_hash) {
        let resource = match app_state.get_resource_info(guid) {
            Some(info) => Resource::new(info),
            None => {
                warn!("Resource info not found for GUID: {}", guid);
                continue;
            }
        };

        result.push(resource)?;
    }

    Ok(result)
}

fn lua_get_all_resources(
    lua: &mlua::Lua,
    _args: FunctionArgs,
) -> mlua::Result<Table> {
    let app_state = lua.app_data_ref::<AppState>().unwrap();

    let file = app_state.kfc_file();
    let result = lua.create_table_with_capacity(file.resources().len(), 0)?;

    for guid in file.resources().keys() {
        let resource = match app_state.get_resource_info(guid) {
            Some(info) => Resource::new(info),
            None => {
                warn!("Resource info not found for GUID: {}", guid);
                continue;
            }
        };

        result.push(resource)?;
    }

    Ok(result)
}

fn lua_get_resource_types(
    lua: &mlua::Lua,
    _args: FunctionArgs,
) -> mlua::Result<Table> {
    let app_state = lua.app_data_ref::<AppState>().unwrap();

    let file = app_state.kfc_file();
    let result = lua.create_table_with_capacity(
        file.resource_bundles().len(),
        0
    )?;

    for qualified_hash in file.resource_types() {
        let r#type = app_state.type_registry()
            .get_by_hash(LookupKey::Qualified(qualified_hash))
            .expect("type not found in registry");
        let value = app_state.get_type(lua, r#type.index)?
            .expect("type not found in context");

        result.push(value)?;
    }

    Ok(result)
}

fn lua_create_resource(
    lua: &mlua::Lua,
    args: FunctionArgs,
) -> mlua::Result<Resource> {
    let app_state = lua.app_data_ref::<AppState>().unwrap();

    let value = args.get::<LuaValue>(0)?;
    let r#type = get_type(&args, 1, app_state.type_registry().as_ref())?;

    let guid = if args.len() > 2 {
        let guid = args.get::<Guid>(2)?;
        let part = args.get::<u32>(3)?;

        let guid = ResourceId::new(
            guid,
            r#type.qualified_hash,
            part
        );

        app_state.add_resource(
            value,
            &guid,
            lua,
        )?;

        guid
    } else {
        app_state.create_resource(
            value,
            r#type.index(),
            lua,
        )?
    };

    Ok(app_state.get_resource_info(&guid)
        .map(Resource::new)
        .expect("just created resource info must exist"))
}

fn lua_get_content(
    lua: &mlua::Lua,
    args: FunctionArgs,
) -> mlua::Result<Option<Content>> {
    let app_state = lua.app_data_ref::<AppState>().unwrap();

    let guid = args.get::<ContentHash>(0)?;

    if !app_state.kfc_file().contents().contains_key(&guid) {
        return Ok(None);
    }

    Ok(Some(Content::new(guid)))
}

fn lua_get_all_contents(
    lua: &mlua::Lua,
    _args: FunctionArgs,
) -> mlua::Result<Table> {
    let app_state = lua.app_data_ref::<AppState>().unwrap();

    let file = app_state.kfc_file();
    let result = lua.create_table_with_capacity(file.contents().len(), 0)?;

    for guid in file.contents().keys() {
        result.push(Content::new(*guid))?;
    }

    Ok(result)
}

fn lua_create_content(
    lua: &mlua::Lua,
    args: FunctionArgs,
) -> mlua::Result<Content> {
    let app_state = lua.app_data_ref::<AppState>().unwrap();

    let buffer = args.get::<&Buffer>(0)?;

    let data = buffer.data()?;

    if data.len() > u32::MAX as usize {
        return Err(LuaError::generic("content size exceeds maximum limit"));
    }

    app_state.create_content(data)
        .map(Content::new)
}
