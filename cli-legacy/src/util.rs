use std::borrow::Borrow;

use kfc::{container::{KFCFile, KFCReader}, guid::DescriptorGuid, reflection::TypeCollection};
use serde_json::Value as JsonValue;

pub fn read_descriptor_into<F, T>(
    reader: &mut KFCReader<F, T>,
    guid: &DescriptorGuid,
    buf: &mut Vec<u8>
) -> anyhow::Result<Option<JsonValue>>
where
    F: Borrow<KFCFile>,
    T: Borrow<TypeCollection>
{
    if !reader.read_descriptor_into(guid, buf)? {
        return Ok(None);
    }

    let result = deserialize_descriptor(
        reader.type_collection(),
        guid,
        buf
    )?;

    Ok(Some(result))
}

pub fn serialize_descriptor(
    type_collection: &TypeCollection,
    value: &JsonValue
) -> anyhow::Result<(DescriptorGuid, Vec<u8>)> {
    let mut result = Vec::new();

    serialize_descriptor_into(type_collection, value, &mut result)
        .map(|guid| (guid, result))
}

pub fn serialize_descriptor_into(
    type_collection: &TypeCollection,
    value: &JsonValue,
    dst: &mut Vec<u8>
) -> anyhow::Result<DescriptorGuid> {
    if let Some(obj) = value.as_object() {
        let type_name = obj.get("$type")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing or invalid $type field"))?;
        let guid = obj.get("$guid")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing or invalid $guid field"))?;
        let part = obj.get("$part")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;

        let type_info = type_collection.get_type_by_qualified_name(type_name)
            .ok_or_else(|| anyhow::anyhow!("Invalid type: {}", type_name))?;
        let guid = DescriptorGuid::from_str(
            guid,
            type_info.qualified_hash,
            part
        ).ok_or_else(|| anyhow::anyhow!("Invalid GUID: {}", guid))?;

        kfc::descriptor::json::serialize_into(
            type_collection,
            type_info,
            value,
            dst
        )?;

        Ok(guid)
    } else {
        Err(anyhow::anyhow!("Root value must be an object"))
    }
}

pub fn deserialize_descriptor(
    type_collection: &TypeCollection,
    guid: &DescriptorGuid,
    data: &[u8]
) -> anyhow::Result<JsonValue> {
    let type_entry = type_collection.get_type_by_qualified_hash(guid.type_hash)
        .ok_or_else(|| anyhow::anyhow!("Type not found for GUID: {}", guid))?;
    let mut result = kfc::descriptor::json::deserialize(type_collection, type_entry, data)?;

    if let Some(obj) = result.as_object_mut() {
        obj.insert("$type".into(), JsonValue::String(type_entry.qualified_name.clone()));
        obj.insert("$guid".into(), JsonValue::String(guid.to_string()));
        obj.insert("$part".into(), JsonValue::Number(guid.part_number.into()));

        Ok(result)
    } else {
        Err(anyhow::anyhow!("Root value must be an object"))
    }
}
