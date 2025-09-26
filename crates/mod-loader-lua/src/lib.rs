use kfc::reflection::LookupKey;
use mod_loader::ModEnvironment;

use crate::{alias::Path, cache::{CacheDiff, FileStateCache}, env::{AppFeatures, AppState}, log::info, runner::LuaModRunner};

mod runner;
mod definition;
mod env;
mod util;
mod log;
mod cache;
mod alias;
mod load;
mod lua;

#[derive(Debug, Clone, Default)]
pub struct RunOptions {
    /// If true, it will ignore the cache and re-apply all mods.
    pub skip_cache: bool,
    /// If None, it will auto-detect based on the files within the game directory.
    pub is_server: Option<bool>,
    /// If true, it will apply patches to the game files.
    pub patch: bool,
    /// If true, it will allow mods to use `io.export` to export files to the export directory.
    pub export: bool,
    /// If None, it will use the default export directory (`<game_dir>/export`).
    pub export_dir: Option<std::path::PathBuf>,
}

#[derive(Debug, Clone)]
pub struct RunArgs {
    pub file_name: String,
    /// The features to enable.
    pub options: RunOptions,
}

pub fn run(
    env: &ModEnvironment,
    args: RunArgs,
) -> anyhow::Result<()> {
    info!("Running lua with options: {:?}", args);

    // check cache if files have changed

    let cache_diff = if !args.options.skip_cache {
        let current_cache = FileStateCache::read(env.cache_dir());
        let mut new_cache = FileStateCache::default();

        new_cache.track_game_files(env.game_dir());
        new_cache.track_mod_files(env.mods_dir());

        let cache_diff = new_cache.diff(&current_cache);

        if cache_diff.is_none() {
            info!("No changes detected, skipping patching");

            return Ok(());
        }

        cache_diff
    } else {
        info!("Skipping cache check");
        CacheDiff::new_dirty()
    };

    // create a new app state

    let app_state = match AppState::new(
        env.clone(),
        args,
        &cache_diff,
    ) {
        Ok(context) => context,
        Err(_) => anyhow::bail!("Failed to create AppState"),
    };

    // setup and run all mods

    let runner = LuaModRunner::new(app_state)?;

    match runner.setup(env.mod_registry()) {
        Ok(_) => {},
        Err(err) => {
            anyhow::bail!("Failed to setup LuaModRunner: {}", err);
        }
    }

    info!("Running mods...");

    // TODO: move this somewhere else, maybe to the GameContext?

    runner.run()?;

    let app_state = runner.lua.app_data_ref::<AppState>().unwrap();

    if app_state.has_feature(AppFeatures::PATCH) {
        info!("Applying patches...");

        let mut buf = Vec::new();
        let mut writer = app_state.take_writer();
        let resources = app_state.get_cached_resources();

        for resource in resources.iter() {
            let value = resource.apply(&runner.lua)?;

            if let Some(value) = value {
                let r#type = app_state.type_registry()
                    .get_by_hash(LookupKey::Qualified(resource.guid.type_hash()))
                    .expect("Failed to find type by qualified hash");

                buf.clear();
                value.write_into(
                    app_state.type_registry(),
                    r#type,
                    &mut buf,
                ).unwrap();
                writer.write_resource(&resource.guid, &buf)?;
            }
        }

        writer.finalize()?;
    }

    // create a new cache file

    let mut new_cache = FileStateCache::default();

    new_cache.track_game_files(env.game_dir());
    new_cache.track_mod_files(env.mods_dir());

    new_cache.write(env.cache_dir());

    Ok(())
}

pub fn export_lua_definitions(
    game_dir: impl AsRef<Path>,
    file_name: &str,
    force: bool,
) -> bool {
    let game_dir = game_dir.as_ref();
    let cache_dir = game_dir.join(".cache");
    let (type_registry, is_dirty) = match crate::load::load_type_registry(
        game_dir,
        &cache_dir,
        file_name,
    ) {
        Ok(type_registry) => type_registry,
        Err(_) => return false,
    };

    crate::load::export_lua_definitions(
        &cache_dir,
        &type_registry,
        is_dirty || force,
    );

    true
}

pub fn restore(
    game_dir: impl AsRef<Path>,
    file_name: &str,
) -> bool {
    let kfc_path = game_dir.as_ref()
        .join(format!("{file_name}.kfc"));

    load::restore_backup(&kfc_path).is_ok()
}
