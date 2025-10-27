mod cli;
mod logging;
mod log;

use std::path::{Path, PathBuf};

use clap::Parser;
use dialoguer::{theme::ColorfulTheme, Input, MultiSelect};
use mod_loader::{lua::{export_lua_definitions, RunArgs, RunOptions}, Capability, ModEnvironment, ModManifest};

use crate::{cli::{Cli, Commands}, log::{error, info}};

const LUARC_JSON: &str = include_str!("../files/luarc.json");
const DEFAULT_MOD: &str = include_str!("../files/default_mod.lua");

fn main() -> Result<(), Box<dyn std::error::Error>> {
    logging::setup();

    let cli = Cli::parse();

    match cli.commands {
        Commands::Create {
            game_directory,
            file_name
        } => create(game_directory, file_name),
        Commands::Run {
            patch,
            export,
            runtime,
            game_directory,
            export_directory,
            file_name,
            force
        } => run(
            game_directory,
            file_name,
            force,

            patch,
            export,
            runtime,

            export_directory
        ),
        Commands::Restore {
            game_directory,
            file_name
        } => restore(game_directory, file_name),
    }
}

fn check_game_directory(
    game_directory: &Path,
    file_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    if !game_directory.exists() {
        error!("Game directory does not exist");
        return Ok(());
    }

    if !game_directory.is_dir() {
        error!("Game directory is not a directory");
        return Ok(());
    }

    if !game_directory.join(file_name).with_extension("kfc").exists() &&
        !game_directory.join(file_name).with_extension("exe").exists() {
        error!("Game directory does not look like the enshrouded game directory");
        return Ok(());
    }

    Ok(())
}

fn create(
    game_directory: PathBuf,
    file_name: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let file_name = file_name.unwrap_or_else(|| "enshrouded".to_string());

    check_game_directory(&game_directory, &file_name)?;

    let mods_dir = game_directory.join("mods");

    if !mods_dir.exists() {
        std::fs::create_dir(&mods_dir)?;
    }

    let theme = ColorfulTheme::default();
    let mut mod_id: String;

    loop {
        mod_id = Input::with_theme(&theme)
            .with_prompt("ID")
            .interact_text()?;

        let mod_dir = mods_dir.join(&mod_id);

        if mod_dir.exists() {
            error!("Mod directory already exists: {}", mod_dir.display());
        } else {
            break;
        }
    }

    let mod_name: String = Input::with_theme(&theme)
        .with_prompt("Name")
        .default(mod_id.clone())
        .interact_text()?;
    let mod_version: String = Input::with_theme(&theme)
        .with_prompt("Version")
        .default("0.1.0".to_string())
        .interact_text()?;
    let mod_author: String = Input::with_theme(&theme)
        .with_prompt("Author")
        .default("".to_string())
        .interact_text()?;

    let capability_list = [
        "Patch",
        "Export",
        "Runtime",
    ];
    let capabilities = MultiSelect::with_theme(&theme)
        .with_prompt("Select mod capabilities")
        .items(&capability_list)
        .defaults(&[true])
        .interact()?;

    let capabilities = capabilities.iter()
        .map(|&capability| {
            match capability {
                0 => Capability::Patch,
                1 => Capability::Export,
                2 => Capability::Runtime,
                _ => unreachable!(),
            }
        })
        .collect::<Vec<_>>();

    let mod_manifest = ModManifest {
        id: mod_id,
        name: mod_name,
        version: mod_version,

        capabilities,
        dependencies: vec![],

        description: None,
        authors: vec![mod_author],
        license: None,
        icon: None,
    };

    // create the following directory structure:
    // mods/
    // └── <mod_id>/
    //    ├── mod.json
    //    ├── README.md
    //    ├── .luarc.json
    //    └── src/
    //        └── mod.lua

    let mod_dir = mods_dir.join(&mod_manifest.id);

    if !mod_dir.exists() {
        std::fs::create_dir(&mod_dir)?;
    } else {
        error!("Mod directory already exists: {}", mod_dir.display());
        return Ok(());
    }

    // create the mod.json file
    std::fs::write(
        mod_dir.join("mod.json"),
        serde_json::to_string_pretty(&mod_manifest)?
    )?;

    // create the README.md file
    std::fs::write(
        mod_dir.join("README.md"),
        match mod_manifest.description {
            Some(desc) => format!("# {}\n\n{}\n", mod_manifest.name, desc),
            None => format!("# {}\n", mod_manifest.name),
        }
    )?;

    // create the .luarc.json file
    std::fs::write(
        mod_dir.join(".luarc.json"),
        LUARC_JSON
    )?;

    // create the src directory
    let src_dir = mod_dir.join("src");

    if !src_dir.exists() {
        std::fs::create_dir(&src_dir)?;
    }

    // create the mod.lua file
    std::fs::write(
        src_dir.join("mod.lua"),
        DEFAULT_MOD
    )?;

    match game_directory.to_str() {
        Some(path) => {
            export_lua_definitions(
                path,
                &file_name,
                false
            );
        }
        None => {
            error!("Game directory is not valid UTF-8, unable to export Lua definitions");
            return Ok(());
        }
    }

    info!("Mod has been created at {}", mod_dir.display());

    Ok(())
}

fn run(
    game_directory: PathBuf,
    file_name: Option<String>,
    force: bool,
    patch: bool,
    export: bool,
    _runtime: bool,
    export_directory: Option<PathBuf>,
) -> Result<(), Box<dyn std::error::Error>> {
    let export_directory = match export_directory {
        Some(dir) => dir,
        None => game_directory.join("export"),
    };
    let file_name = file_name.unwrap_or_else(|| "enshrouded".to_string());

    check_game_directory(&game_directory, &file_name)?;

    let utf8_game_directory = match game_directory.to_str() {
        Some(path) => path,
        None => {
            error!("Game directory is not valid UTF-8");
            return Ok(());
        }
    };

    let mod_registry = match ModEnvironment::load(utf8_game_directory) {
        Ok(registry) => registry,
        Err(e) => {
            if let Some(error) = e.error {
                error!("Error loading mod environment: {}", error);
                return Ok(());
            }

            error!("Errors loading some mods:");

            for mod_error in e.mods {
                if let Some(id) = &mod_error.id {
                    error!(
                        "  In mod '{}' at {}: {}",
                        id,
                        mod_error.path,
                        mod_error.error
                    );
                } else {
                    error!(
                        "  In mod at {}: {}",
                        mod_error.path,
                        mod_error.error
                    );
                }
            }

            return Ok(());
        }
    };

    mod_loader::lua::run(
        &mod_registry,
        RunArgs {
            file_name,
            options: RunOptions {
                skip_cache: force,
                patch,
                export,
                export_dir: Some(export_directory),
                ..Default::default()
            },
        },
    )?;

    Ok(())
}

fn restore(
    game_directory: PathBuf,
    file_name: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let file_name = file_name.unwrap_or_else(|| "enshrouded".to_string());

    check_game_directory(&game_directory, &file_name)?;

    mod_loader::lua::restore(
        game_directory.to_str().unwrap(),
        &file_name
    );

    Ok(())
}
