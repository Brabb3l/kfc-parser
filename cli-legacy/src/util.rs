use std::borrow::Borrow;

use kfc::{container::{KFCFile, KFCReader}, descriptor::value::{ConversionOptions, Value}, guid::DescriptorGuid, reflection::{LookupKey, TypeRegistry}};

pub fn read_descriptor_into<F, T>(
    reader: &mut KFCReader<F, T>,
    guid: &DescriptorGuid,
    buf: &mut Vec<u8>
) -> anyhow::Result<Option<Value>>
where
    F: Borrow<KFCFile>,
    T: Borrow<TypeRegistry>
{
    if !reader.read_descriptor_into(guid, buf)? {
        return Ok(None);
    }

    let result = deserialize_descriptor(
        reader.type_registry(),
        guid,
        buf
    )?;

    Ok(Some(result))
}

pub fn serialize_descriptor(
    type_registry: &TypeRegistry,
    value: &Value
) -> anyhow::Result<(DescriptorGuid, Vec<u8>)> {
    let mut result = Vec::new();

    serialize_descriptor_into(type_registry, value, &mut result)
        .map(|guid| (guid, result))
}

pub fn serialize_descriptor_into(
    type_registry: &TypeRegistry,
    value: &Value,
    dst: &mut Vec<u8>
) -> anyhow::Result<DescriptorGuid> {
    if let Some(obj) = value.as_struct() {
        let type_name = obj.get("$type")
            .and_then(|v| v.as_string())
            .ok_or_else(|| anyhow::anyhow!("Missing or invalid $type field"))?;
        let guid = obj.get("$guid")
            .and_then(|v| v.as_string())
            .ok_or_else(|| anyhow::anyhow!("Missing or invalid $guid field"))?;
        let part = obj.get("$part")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;

        let r#type = type_registry.get_by_name(LookupKey::Qualified(type_name))
            .ok_or_else(|| anyhow::anyhow!("Invalid type: {}", type_name))?;
        let guid = DescriptorGuid::from_str(
            guid,
            r#type.qualified_hash,
            part
        ).ok_or_else(|| anyhow::anyhow!("Invalid GUID: {}", guid))?;

        value.write_into(
            type_registry,
            r#type,
            dst
        )?;

        Ok(guid)
    } else {
        Err(anyhow::anyhow!("Root value must be an object"))
    }
}

pub fn deserialize_descriptor(
    type_registry: &TypeRegistry,
    guid: &DescriptorGuid,
    data: &[u8]
) -> anyhow::Result<Value> {
    let r#type = type_registry.get_by_hash(LookupKey::Qualified(guid.type_hash))
        .ok_or_else(|| anyhow::anyhow!("Type not found for GUID: {}", guid))?;
    let mut result = Value::from_bytes_with_options(
        type_registry,
        r#type,
        data,
        ConversionOptions::HUMAN_READABLE,
    )?;

    if let Some(obj) = result.as_struct_mut() {
        obj.insert("$type".into(), Value::String(r#type.qualified_name.clone()));
        obj.insert("$guid".into(), Value::String(guid.to_string()));
        obj.insert("$part".into(), Value::UInt(guid.part_number.into()));

        Ok(result)
    } else {
        Err(anyhow::anyhow!("Root value must be an object"))
    }
}
