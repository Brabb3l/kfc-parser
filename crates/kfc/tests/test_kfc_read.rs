use std::{collections::HashMap, path::PathBuf};

use kfc::{container::KFCReader, guid::ContentHash};
use kfc_base::{container::KFCFile, reflection::{LookupKey, TypeRegistry}};

fn get_game_dir() -> PathBuf {
    std::env::var("GAME_DIR")
        .expect("GAME_DIR environment variable not set")
        .into()
}

#[test]
#[ignore = "requires GAME_DIR environment variable"]
fn test_validate_kfc() -> Result<(), Box<dyn std::error::Error>> {
    let mut buf = vec![0; 1024 * 1024]; // 1 MB buffer

    let dir = get_game_dir();
    let kfc_path = dir.join("enshrouded.kfc");
    let exe_path = dir.join("enshrouded.exe");

    // Load TypeRegistry from the executable
    let type_registry = TypeRegistry::load_from_executable(&exe_path)?;

    // Open the KFC file
    let kfc_file = KFCFile::from_path(&kfc_path, false)?;
    let mut reader = KFCReader::new(&dir, "enshrouded")?.into_cursor()?;

    // check type registry for valid resource types
    for guid in kfc_file.resources().keys() {
        let type_hash = guid.type_hash();
        let r#type = type_registry.get_by_hash(LookupKey::Qualified(type_hash));

        assert!(r#type.is_some(), "Type {type_hash} of {guid} not found in type registry");
    }

    // check resource map
    for guid in kfc_file.resources().keys() {
        // check existence, if either of those fail:
        // - the hash function is likely broken
        // - the KFC file is corrupted
        assert!(kfc_file.resources().contains_key(guid), "Resource {guid} not found in KFC file");
        assert!(kfc_file.resources().get(guid).is_some(), "Missing resource entry for {guid}");

        // check reading
        buf.clear();

        let exists = reader.read_resource_into(guid, &mut buf)?;
        assert!(exists, "Resource {guid} not found in KFC file");

        // check the size
        let entry = kfc_file.resources().get(guid)
            .expect("Resource entry not found"); // checked above

        assert_eq!(
            buf.len(),
            entry.size as usize,
            "Resource {} has size {}, but expected {}", guid, buf.len(), entry.size
        );
    }

    // check content map
    for guid in kfc_file.contents().keys() {
        // If either of those fail:
        // - the hash function is likely broken
        // - the KFC file is corrupted
        assert!(kfc_file.contents().contains_key(guid), "Content {guid} not found in KFC file");
        assert!(kfc_file.contents().get(guid).is_some(), "Missing content entry for {guid}");

        // check reading
        buf.clear();

        let exists = reader.read_content_into(guid, &mut buf)?;
        assert!(exists, "Content {guid} not found in KFC file");

        // check the size
        let entry = kfc_file.contents().get(guid)
            .expect("Content entry not found"); // checked above

        assert!(entry.flags == 0, "Content {} has non-zero flags: {}", guid, entry.flags);

        assert_eq!(
            buf.len(),
            guid.size() as usize,
            "Content {} has size {}, but expected {}", guid, buf.len(), guid.size()
        );

        // validate guid
        let computed_guid = ContentHash::from_data(&buf);

        assert_eq!(
            computed_guid,
            *guid,
            "Content {guid} has computed guid {computed_guid}, but expected {guid}",
        );
    }

    // check group map
    let mut type_counts = HashMap::new();

    for guid in kfc_file.resources().keys() {
        let type_hash = guid.type_hash();
        let count = type_counts.entry(type_hash).or_insert(0u32);
        *count += 1;
    }

    for &type_hash in kfc_file.resource_bundles().keys() {
        let mut count = 0;

        for guid in kfc_file.resources_by_type(type_hash) {
            assert_eq!(
                guid.type_hash(),
                type_hash,
                "Resource {} has type hash {}, but expected {}", guid, guid.type_hash(), type_hash
            );

            count += 1;
        }

        let expected_count = type_counts.get(&type_hash);

        assert!(expected_count.is_some(), "Type {type_hash} is not a resource type");

        let &expected_count = expected_count.unwrap();

        assert_eq!(
            count,
            expected_count,
            "Type {type_hash} has {count} resources, but expected {expected_count}"
        );
    }

    Ok(())
}
