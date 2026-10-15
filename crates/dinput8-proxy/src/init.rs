use std::path::PathBuf;

use camino::{Utf8Path, Utf8PathBuf};
use mod_loader::{lua::{self, RunArgs, RunOptions}, runtime, Config, ModEnvironment};

use crate::{log::{error, warn}, logging};

pub fn init(config: Config) {
    logging::setup();

    // Errors are logged where they occur. The game must still be able to
    // start, so a failure here only skips the mod loader.
    if load_mods(config).is_err() {
        warn!("Mod loading failed, starting the game without applying mods");
    }
}

pub fn deinit() {
    if let Err(e) = runtime::loader_detach() {
        error!(
            error = %e,
            "Failed to detach runtime loader"
        );
    }
}

fn load_mods(config: Config) -> Result<(), ()> {
    let current_dir = std::env::current_dir()
        .map_err(|e| {
            error!(
                error = %e,
                "Failed to get current directory"
            );
        })?;
    let current_dir = Utf8PathBuf::from_path_buf(current_dir)
        .map_err(|path| {
            error!(
                path = ?path,
                "Current directory path is not valid UTF-8"
            );
        })?;

    let env = match ModEnvironment::load(&current_dir) {
        Ok(env) => env,
        Err(e) => {
            if let Some(error) = e.error {
                error!(
                    error = %error,
                    "Error loading mod environment"
                );
                return Err(());
            }

            let mut report = String::from("Errors loading some mods:");

            for mod_error in e.mods {
                if let Some(id) = &mod_error.id {
                    report += &format!(
                        "\n  In mod '{}' at {}: {}",
                        id,
                        mod_error.path,
                        mod_error.error
                    );
                } else {
                    report += &format!(
                        "\n  In mod at {}: {}",
                        mod_error.path,
                        mod_error.error
                    );
                }
            }

            error!(
                report = %report,
                "Error loading mod environment"
            );
            return Err(());
        }
    };

    let result = lua::run(
        &env,
        RunArgs {
            file_name: get_file_name(&current_dir)?,
            options: RunOptions {
                patch: true,
                export: config.use_export_flag,
                export_dir: config.export_directory.map(PathBuf::from),
                runtime: true,
                ..Default::default()
            },
        },
    );

    let result = match result {
        Ok(result) => result,
        Err(e) => {
            error!(
                error = %e,
                "Error running mod loader"
            );
            return Err(());
        }
    };

    runtime::loader_attach(
        &env,
        runtime::RuntimeOptions {
            dlls: result.dlls,
        },
    ).map_err(|e| {
        error!(
            error = %e,
            "Failed to attach runtime loader"
        );
    })
}

fn get_file_name(
    game_dir: &Utf8Path,
) -> Result<String, ()> {
    if game_dir.join("enshrouded.kfc").exists() || game_dir.join("enshrouded.exe").exists() {
        Ok("enshrouded".into())
    } else if game_dir.join("enshrouded_server.kfc").exists() || game_dir.join("enshrouded_server.exe").exists() {
        Ok("enshrouded_server".into())
    } else {
        // try to find file with .kfc or .exe extension
        for file in std::fs::read_dir(game_dir).into_iter().flatten().flatten() {
            if let Some(ext) = file.path().extension() {
                if ext == "kfc" || ext == "exe" {
                    let file = file.path();
                    let file_name = file.file_stem()
                        .and_then(|s| s.to_str());

                    if let Some(file_name) = file_name {
                        return Ok(file_name.into());
                    }
                }
            }
        }

        error!(
            game_dir = %game_dir,
            "Could not find required game files"
        );
        Err(())
    }
}
