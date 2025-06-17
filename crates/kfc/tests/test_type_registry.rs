use std::path::PathBuf;

use kfc_base::reflection::TypeRegistry;

fn get_game_dir() -> PathBuf {
    std::env::var("GAME_DIR")
        .expect("GAME_DIR environment variable not set")
        .into()
}

#[test]
#[ignore = "requires GAME_DIR environment variable"]
fn test_validate_type_registry() -> Result<(), Box<dyn std::error::Error>> {
    let dir = get_game_dir();
    let _kfc_path = dir.join("enshrouded.kfc");
    let exe_path = dir.join("enshrouded.exe");

    // Load TypeRegistry from the executable
    let _type_registry = TypeRegistry::load_from_executable(&exe_path)?;

    Ok(())
}
