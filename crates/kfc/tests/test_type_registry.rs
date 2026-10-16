use kfc_base::reflection::TypeRegistry;

mod common;

#[test]
#[ignore = "requires GAME_DIR environment variable"]
fn test_validate_type_registry() -> Result<(), Box<dyn std::error::Error>> {
    let game = common::GameFiles::from_env();
    let exe_path = game.path("exe");

    // Load TypeRegistry from the executable
    let _type_registry = TypeRegistry::load_from_executable(&exe_path)?;

    Ok(())
}
