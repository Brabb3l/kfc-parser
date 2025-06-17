use std::path::PathBuf;

use kfc::container::KFCReader;
use kfc_base::{container::KFCFile, reflection::{LookupKey, TypeRegistry}};
use kfc_descriptor::value::{ConversionOptions, Value};

fn get_game_dir() -> PathBuf {
    std::env::var("GAME_DIR")
        .expect("GAME_DIR environment variable not set")
        .into()
}

#[test]
#[ignore = "requires GAME_DIR environment variable"]
fn test_validate_descriptors_with_value() -> Result<(), Box<dyn std::error::Error>> {
    let mut buf = vec![0; 1024 * 1024]; // 1 MB buffer

    let dir = get_game_dir();
    let kfc_path = dir.join("enshrouded.kfc");
    let exe_path = dir.join("enshrouded.exe");

    // Load TypeRegistry from the executable
    let type_registry = TypeRegistry::load_from_executable(&exe_path)?;

    // Open the KFC file
    let kfc_file = KFCFile::from_path(&kfc_path, false)?;
    let mut reader = KFCReader::new(&kfc_path, &kfc_file, &type_registry)?;

    // check parse json
    for guid in kfc_file.get_descriptor_guids() {
        buf.clear();

        let exists = reader.read_descriptor_into(guid, &mut buf);
        assert!(exists.is_ok(), "Failed to read descriptor {}: {}", guid, exists.err().unwrap());
        let exists = exists.unwrap();
        assert!(exists, "Descriptor {} not found in KFC file", guid);

        let r#type = type_registry.get_by_hash(LookupKey::Qualified(guid.type_hash));
        assert!(r#type.is_some(), "Type for descriptor {} not found in TypeRegistry", guid);
        let r#type = r#type.unwrap();

        // deserialize
        let deserialized = kfc::descriptor::value::Value::from_bytes(
            &type_registry,
            r#type,
            &buf
        );
        assert!(deserialized.is_ok(), "Failed to deserialize descriptor {}: {}", guid, deserialized.err().unwrap());
        let deserialized = deserialized.unwrap();

        // reserialize
        let reserialized = deserialized.to_bytes(
            &type_registry,
            r#type
        );
        assert!(reserialized.is_ok(), "Failed to reserialize descriptor {}: {}", guid, reserialized.err().unwrap());
        let reserialized = reserialized.unwrap();

        // check if reserialized matches original
        assert_eq!(
            buf,
            reserialized,
            "Reserialized data for descriptor {} does not match original",
            guid
        );
    }

    Ok(())
}

#[test]
#[ignore = "requires GAME_DIR environment variable"]
fn test_validate_descriptors_with_value_human() -> Result<(), Box<dyn std::error::Error>> {
    let mut buf = vec![0; 1024 * 1024]; // 1 MB buffer

    let dir = get_game_dir();
    let kfc_path = dir.join("enshrouded.kfc");
    let exe_path = dir.join("enshrouded.exe");

    // Load TypeRegistry from the executable
    let type_registry = TypeRegistry::load_from_executable(&exe_path)?;

    // Open the KFC file
    let kfc_file = KFCFile::from_path(&kfc_path, false)?;
    let mut reader = KFCReader::new(&kfc_path, &kfc_file, &type_registry)?;

    // check parse json
    for guid in kfc_file.get_descriptor_guids() {
        buf.clear();

        let exists = reader.read_descriptor_into(guid, &mut buf);
        assert!(exists.is_ok(), "Failed to read descriptor {}: {}", guid, exists.err().unwrap());
        let exists = exists.unwrap();
        assert!(exists, "Descriptor {} not found in KFC file", guid);

        let r#type = type_registry.get_by_hash(LookupKey::Qualified(guid.type_hash));
        assert!(r#type.is_some(), "Type for descriptor {} not found in TypeRegistry", guid);
        let r#type = r#type.unwrap();

        // deserialize
        let deserialized = kfc::descriptor::value::Value::from_bytes_with_options(
            &type_registry,
            r#type,
            &buf,
            ConversionOptions::HUMAN_READABLE,
        );
        assert!(deserialized.is_ok(), "Failed to deserialize descriptor {}: {}", guid, deserialized.err().unwrap());
        let deserialized = deserialized.unwrap();

        // reserialize
        let reserialized = deserialized.to_bytes(
            &type_registry,
            r#type
        );
        assert!(reserialized.is_ok(), "Failed to reserialize descriptor {}: {}", guid, reserialized.err().unwrap());
        let reserialized = reserialized.unwrap();

        // check if reserialized matches original
        assert_eq!(
            buf,
            reserialized,
            "Reserialized data for descriptor {} does not match original",
            guid
        );
    }

    Ok(())
}

#[test]
#[ignore = "requires GAME_DIR environment variable"]
fn test_value_serde_to_json() -> Result<(), Box<dyn std::error::Error>> {
    let mut buf = vec![0; 1024 * 1024]; // 1 MB buffer

    let dir = get_game_dir();
    let kfc_path = dir.join("enshrouded.kfc");
    let exe_path = dir.join("enshrouded.exe");

    // Load TypeRegistry from the executable
    let type_registry = TypeRegistry::load_from_executable(&exe_path)?;

    // Open the KFC file
    let kfc_file = KFCFile::from_path(&kfc_path, false)?;
    let mut reader = KFCReader::new(&kfc_path, &kfc_file, &type_registry)?;

    // check parse json
    for guid in kfc_file.get_descriptor_guids() {
        buf.clear();

        let exists = reader.read_descriptor_into(guid, &mut buf);
        assert!(exists.is_ok(), "Failed to read descriptor {}: {}", guid, exists.err().unwrap());
        let exists = exists.unwrap();
        assert!(exists, "Descriptor {} not found in KFC file", guid);

        let r#type = type_registry.get_by_hash(LookupKey::Qualified(guid.type_hash));
        assert!(r#type.is_some(), "Type for descriptor {} not found in TypeRegistry", guid);
        let r#type = r#type.unwrap();

        // deserialize
        let deserialized = kfc::descriptor::value::Value::from_bytes(
            &type_registry,
            r#type,
            &buf
        );
        assert!(deserialized.is_ok(), "Failed to deserialize descriptor {}: {}", guid, deserialized.err().unwrap());
        let deserialized = deserialized.unwrap();

        let json = serde_json::to_string(&deserialized)
            .expect("Failed to serialize descriptor to JSON");
        let deserialized_json = serde_json::from_str::<Value>(&json)
            .expect("Failed to deserialize JSON back to Value");
        let reserialized_json = deserialized_json.to_bytes(
            &type_registry,
            r#type
        );
        assert!(reserialized_json.is_ok(), "Failed to reserialize descriptor {} from JSON: {}", guid, reserialized_json.err().unwrap());
        let reserialized_json = reserialized_json.unwrap();

        assert_eq!(
            buf,
            reserialized_json,
            "Reserialized data for descriptor {} from JSON does not match original",
            guid
        );
    }

    Ok(())
}
