use std::io::BufReader;

use mod_loader::Mod;

use crate::{alias::{Path, PathBuf}, env::{util::{add_function, add_function_with_mod}, AppFeatures, AppState, Buffer}, log::{warn, warn_span}, lua::{Either, FunctionArgs, LuaError}};

pub fn create(
    lua: &mlua::Lua,
    r#mod: Mod,
) -> mlua::Result<mlua::Table> {
    let table = lua.create_table()?;

    add_function_with_mod(lua, &table, "read", &r#mod, lua_read)?;
    add_function_with_mod(lua, &table, "read_to_string", &r#mod, lua_read_to_string)?;
    add_function_with_mod(lua, &table, "list_files", &r#mod, lua_list_files)?;
    add_function_with_mod(lua, &table, "exists", &r#mod, lua_exists)?;
    add_function_with_mod(lua, &table, "is_file", &r#mod, lua_is_file)?;
    add_function_with_mod(lua, &table, "is_directory", &r#mod, lua_is_directory)?;
    add_function(lua, &table, "name", lua_name)?;
    add_function(lua, &table, "name_without_extension", lua_name_without_extension)?;
    add_function(lua, &table, "extension", lua_extension)?;
    add_function(lua, &table, "parent", lua_parent)?;
    add_function(lua, &table, "join", lua_join)?;
    add_function(lua, &table, "export", lua_export)?;

    Ok(table)
}

fn lua_read(
    _lua: &mlua::Lua,
    args: FunctionArgs,
    r#mod: &Mod,
) -> mlua::Result<Buffer> {
    let path = args.get::<String>(0)?;

    let mut fs = r#mod.fs();
    let file = fs.read_file(path)?;

    // read all bytes

    use std::io::Read;
    let mut buffer = Vec::new();
    let mut reader = BufReader::new(file);
    reader.read_to_end(&mut buffer)?;

    Ok(Buffer::wrap(buffer))
}

fn lua_read_to_string(
    lua: &mlua::Lua,
    args: FunctionArgs,
    r#mod: &Mod,
) -> mlua::Result<mlua::String> {
    let path = args.get::<String>(0)?;

    let mut fs = r#mod.fs();
    let file = fs.read_file(path)?;

    // read all bytes

    use std::io::Read;
    let mut buffer = Vec::new();
    let mut reader = BufReader::new(file);
    reader.read_to_end(&mut buffer)?;

    lua.create_string(&buffer)
}

fn lua_list_files(
    _lua: &mlua::Lua,
    args: FunctionArgs,
    r#mod: &Mod,
) -> mlua::Result<Vec<String>> {
    let path = args.get::<String>(0)?;

    let fs = r#mod.fs();
    let mod_id = &r#mod.info().id;

    let files = fs.read_directory(&path)?;
    let mut result = Vec::with_capacity(files.len());

    for file in files {
        let file = match file {
            Ok(p) => into_forward_slash(p),
            Err(e) => {
                warn_span!("lua", mod_id).in_scope(|| {
                    warn!(
                        error = %e,
                        path = path.as_str(),
                        "failed to read directory",
                    );
                });

                continue;
            }
        };

        result.push(file);
    }

    Ok(result)
}

fn lua_exists(
    _lua: &mlua::Lua,
    args: FunctionArgs,
    r#mod: &Mod,
) -> mlua::Result<bool> {
    let path = args.get::<String>(0)?;
    let fs = r#mod.fs();

    Ok(fs.exists(path))
}

fn lua_is_file(
    _lua: &mlua::Lua,
    args: FunctionArgs,
    r#mod: &Mod,
) -> mlua::Result<bool> {
    let path = args.get::<String>(0)?;
    let mut fs = r#mod.fs();

    Ok(fs.is_file(path))
}

fn lua_is_directory(
    _lua: &mlua::Lua,
    args: FunctionArgs,
    r#mod: &Mod,
) -> mlua::Result<bool> {
    let path = args.get::<String>(0)?;
    let mut fs = r#mod.fs();

    Ok(fs.is_directory(path))
}

fn lua_name(
    lua: &mlua::Lua,
    args: FunctionArgs,
) -> mlua::Result<mlua::String> {
    let path = args.get::<String>(0)?;

    PathBuf::from(path).file_name()
        .map(|s| lua.create_string(s))
        .unwrap_or_else(|| lua.create_string(""))
}

fn lua_name_without_extension(
    lua: &mlua::Lua,
    args: FunctionArgs,
) -> mlua::Result<mlua::String> {
    let path = args.get::<String>(0)?;

    PathBuf::from(path).file_stem()
        .map(|s| lua.create_string(s))
        .unwrap_or_else(|| lua.create_string(""))
}

fn lua_extension(
    lua: &mlua::Lua,
    args: FunctionArgs,
) -> mlua::Result<mlua::String> {
    let path = args.get::<String>(0)?;

    PathBuf::from(path).extension()
        .map(|s| lua.create_string(s))
        .unwrap_or_else(|| lua.create_string(""))
}

fn lua_parent(
    lua: &mlua::Lua,
    args: FunctionArgs,
) -> mlua::Result<mlua::String> {
    let path = args.get::<String>(0)?;

    PathBuf::from(path).parent()
        .map(|p| into_forward_slash(p.to_path_buf()))
        .map(|s| lua.create_string(s.as_str()))
        .unwrap_or_else(|| lua.create_string(""))
}

fn lua_join(
    lua: &mlua::Lua,
    args: FunctionArgs,
) -> mlua::Result<mlua::String> {
    let mut path = PathBuf::new();

    for p in 0..args.len() {
        let part = args.get::<String>(p)?;
        path.push(part);
    }

    let path = into_forward_slash(path);

    lua.create_string(path.as_str())
}

fn lua_export(
    lua: &mlua::Lua,
    args: FunctionArgs,
) -> mlua::Result<()> {
    let app_state = lua.app_data_ref::<AppState>().unwrap();

    if !app_state.has_feature(AppFeatures::EXPORT) {
        return Err(LuaError::generic("exporting is disabled"));
    }

    let path = args.get::<String>(0)?;
    let buffer = args.get::<Either<&[u8], &Buffer>>(1)?;

    let export_dir = &app_state.export_dir();

    assert!(export_dir.is_absolute(), "Export path must be absolute");

    let path = export_dir.join(
        sanitize_path(Path::new(&path))
            .map_err(|e| LuaError::generic(format!("invalid path: {e}")))?
    );

    if path.exists() && !path.is_file() {
        return Err(LuaError::generic("path exists but is not a file"));
    }

    if let Some(parent) = path.parent() {
        if let Err(e) = std::fs::create_dir_all(parent) {
            return Err(LuaError::generic(format!(
                "failed to create parent directories: {e}"
            )));
        }
    }

    match buffer {
        Either::A(data) => {
            std::fs::write(&path, data)
                .map_err(|e| {
                    LuaError::generic(format!("failed to write file: {e}"))
                })?;
        },
        Either::B(buf) => {
            std::fs::write(&path, buf.data()?)
                .map_err(|e| {
                    LuaError::generic(format!("failed to write file: {e}"))
                })?;
        },
    }

    Ok(())
}

fn sanitize_path(
    path: &Path,
) -> std::io::Result<PathBuf> {
    let mut result = PathBuf::new();

    for component in path.components() {
        match component {
            camino::Utf8Component::Prefix(_) =>
                return Err(std::io::Error::from(std::io::ErrorKind::PermissionDenied)),
            camino::Utf8Component::RootDir =>
                return Err(std::io::Error::from(std::io::ErrorKind::PermissionDenied)),
            camino::Utf8Component::CurDir => {}
            camino::Utf8Component::ParentDir => if !result.pop() {
                return Err(std::io::Error::from(std::io::ErrorKind::PermissionDenied));
            },
            camino::Utf8Component::Normal(part) => result.push(part),
        }
    }

    Ok(result)
}

fn into_forward_slash(pb: PathBuf) -> String {
    pb.into_string().replace('\\', "/")
}
