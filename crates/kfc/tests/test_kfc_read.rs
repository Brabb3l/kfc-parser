use std::{collections::HashMap, path::PathBuf};

use kfc::{container::KFCReader, guid::BlobGuid};
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
    let mut reader = KFCReader::new(&kfc_path, &kfc_file, &type_registry)?;

    // check type registry for valid descriptor types
    for guid in kfc_file.get_descriptor_guids() {
        let type_hash = guid.type_hash;
        let r#type = type_registry.get_by_hash(LookupKey::Qualified(type_hash));

        assert!(r#type.is_some(), "Type {} of {} not found in type registry", type_hash, guid);
    }

    // check descriptor map
    for guid in kfc_file.get_descriptor_guids() {
        // check existence, if either of those fail:
        // - the hash function is likely broken
        // - the KFC file is corrupted
        assert!(kfc_file.contains_descriptor(guid), "Descriptor {} not found in KFC file", guid);
        assert!(kfc_file.get_descriptor_link(guid).is_some(), "Missing descriptor link for {}", guid);

        // check reading
        buf.clear();

        let exists = reader.read_descriptor_into(guid, &mut buf)?;
        assert!(exists, "Descriptor {} not found in KFC file", guid);

        // check the size
        let link = kfc_file.get_descriptor_link(guid)
            .expect("Descriptor link not found"); // checked above

        assert_eq!(
            buf.len(),
            link.size as usize,
            "Descriptor {} has size {}, but expected {}", guid, buf.len(), link.size
        );
    }

    // check blob map
    for guid in kfc_file.get_blob_guids() {
        // If either of those fail:
        // - the hash function is likely broken
        // - the KFC file is corrupted
        assert!(kfc_file.contains_blob(guid), "Blob {} not found in KFC file", guid);
        assert!(kfc_file.get_blob_link(guid).is_some(), "Missing blob link for {}", guid);

        // check reading
        buf.clear();

        let exists = reader.read_blob_into(guid, &mut buf)?;
        assert!(exists, "Blob {} not found in KFC file", guid);

        // check the size
        let link = kfc_file.get_blob_link(guid)
            .expect("Blob link not found"); // checked above

        assert!(link.flags == 0, "Blob {} has non-zero flags: {}", guid, link.flags);

        assert_eq!(
            buf.len(),
            guid.size() as usize,
            "Blob {} has size {}, but expected {}", guid, buf.len(), guid.size()
        );

        // validate guid
        let computed_guid = BlobGuid::from_data(&buf);

        assert_eq!(
            computed_guid,
            *guid,
            "Blob {} has computed guid {}, but expected {}", guid, computed_guid, guid
        );
    }

    // check group map
    let mut type_counts = HashMap::new();

    for guid in kfc_file.get_descriptor_guids() {
        let type_hash = guid.type_hash;
        let count = type_counts.entry(type_hash).or_insert(0u32);
        *count += 1;
    }

    for &type_hash in kfc_file.get_descriptor_types() {
        let mut count = 0;

        for guid in kfc_file.get_descriptor_guids_by_type_hash(type_hash) {
            assert_eq!(
                guid.type_hash,
                type_hash,
                "Descriptor {} has type hash {}, but expected {}", guid, guid.type_hash, type_hash
            );

            count += 1;
        }

        let expected_count = type_counts.get(&type_hash);

        assert!(expected_count.is_some(), "Type {} is not a descriptor type", type_hash);

        let &expected_count = expected_count.unwrap();

        assert_eq!(
            count,
            expected_count,
            "Type {} has {} descriptors, but expected {}", type_hash, count, expected_count
        );
    }

    Ok(())
}
