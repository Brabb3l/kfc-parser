use kfc_resource::value::{ConversionOptions, Value};
use serde::{Deserialize, Serialize};
use std::cmp::max;
use std::collections::HashMap;

use kfc::hash::fnv;
use kfc::guid::ContentHash;
use kfc::reflection::{LookupKey, TypeRegistry};

use crate::hash_types::HashKey32;

use super::{EventStream, ImpactProgram};
use super::{ImpactCommand, ImpactVariable};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImpactProgramData {
    pub data: Vec<ImpactProgramDataEntry>,
    pub query_call_ids: Vec<u32>,
    pub used_streams: Vec<EventStream>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImpactProgramDataEntry {
    pub name: String,
    pub r#type: String,
    pub config_id: u32,
    pub data: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent: Option<ImpactProgramDataMapping>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImpactProgramDataMapping {
    pub parent_name: String,
    pub name: String,
    pub field_name: String,
}

// TODO: Proper error handling

impl ImpactProgramData {
    pub fn from_program(
        type_collection: &TypeRegistry,
        program: &ImpactProgram,
    ) -> anyhow::Result<Self> {
        let mut data_entries = Vec::new();
        let mut mapping_offsets = HashMap::new();

        for (i, layout) in program.data_layout.iter().enumerate() {
            let type_info = type_collection
                .get_by_hash(LookupKey::Impact(layout.r#type.value))
                .ok_or_else(|| anyhow::anyhow!("Type not found: {}", layout.r#type.value))?;

            let start = layout.offset_in_bytes as usize;
            let end = start + layout.size as usize;

            if let Some(i) = mapping_offsets.get(&start) {
                let entry: &ImpactProgramDataEntry = &data_entries[*i];
                let parent_layout: &ImpactVariable = &program.data_layout[*i];
                let field_offset = (layout.offset_in_bytes - parent_layout.offset_in_bytes) as u64;
                let parent_type = type_collection
                    .get_by_name(LookupKey::Qualified(&entry.r#type))
                    .unwrap(); // already checked
                let field_name = parent_type.struct_fields.values()
                    .find(|x| x.data_offset == field_offset)
                    .map(|x| x.name.clone())
                    .ok_or_else(|| anyhow::anyhow!("Field not found: {}", layout.dbg_name))?;

                data_entries.push(ImpactProgramDataEntry {
                    name: layout.dbg_name.clone(),
                    r#type: type_info.qualified_name.clone(),
                    config_id: layout.config_id.value,
                    data: Value::None,
                    parent: Some(ImpactProgramDataMapping {
                        parent_name: entry.name.clone(),
                        name: layout.dbg_name.clone(),
                        field_name
                    }),
                });
            } else {
                let data = Value::from_bytes_with_options(
                    type_collection,
                    type_info,
                    &program.data[start..end],
                    ConversionOptions::HUMAN_READABLE
                )?;

                data_entries.push(ImpactProgramDataEntry {
                    name: layout.dbg_name.clone(),
                    r#type: type_info.qualified_name.clone(),
                    config_id: layout.config_id.value,
                    data,
                    parent: None,
                });
            }

            for field in type_info.struct_fields.values() {
                mapping_offsets.insert(start + field.data_offset as usize, i);
            }
        }

        Ok(Self {
            data: data_entries,
            query_call_ids: program.query_call_ids.clone(),
            used_streams: program.used_streams.clone(),
        })
    }

    pub fn into_program(
        self,
        type_collection: &TypeRegistry,
        guid: ContentHash,
        code: Vec<ImpactCommand>,
        code_shutdown: Vec<ImpactCommand>,
    ) -> anyhow::Result<ImpactProgram> {
        let id = HashKey32::from(guid.hash32());
        let program_guid = guid.to_string();
        let stack_size = 256;
        let used_streams = self.used_streams;
        let query_call_ids = self.query_call_ids;

        let mut buf = Vec::new();
        let mut data = Vec::new();
        let mut data_layout = Vec::<ImpactVariable>::new();

        for entry in self.data.into_iter() {
            let dbg_name = entry.name;
            let config_id = HashKey32::from(entry.config_id);
            let name = HashKey32::from(fnv(&dbg_name));

            let type_info = type_collection
                .get_by_name(LookupKey::Qualified(&entry.r#type))
                .ok_or_else(|| anyhow::anyhow!("Type not found: {}", entry.r#type))?;
            let r#type = HashKey32::from(type_info.impact_hash);

            if let Some(mapping) = entry.parent {
                let parent_hash = fnv(&mapping.parent_name);
                let parent_entry = data_layout.iter()
                    .find(|x| x.name.value == parent_hash)
                    .ok_or_else(|| anyhow::anyhow!("Parent not found: {}", mapping.parent_name))?;
                let parent_type = type_collection
                    .get_by_hash(LookupKey::Impact(parent_entry.r#type.value))
                    .unwrap(); // already checked
                let field = parent_type.struct_fields.values()
                    .find(|x| x.name == mapping.field_name)
                    .ok_or_else(|| anyhow::anyhow!("Field not found: {}", mapping.field_name))?;

                data_layout.push(ImpactVariable {
                    name,
                    config_id,
                    r#type,
                    size: type_info.size as u16,
                    offset_in_bytes: parent_entry.offset_in_bytes + field.data_offset as u16,
                    dbg_name,
                });
            } else {
                buf.clear();

                if !entry.data.is_none() {
                    entry.data.write_into(
                        type_collection,
                        type_info,
                        &mut buf
                    )?;
                } else {
                    buf.resize(type_info.size as usize, 0);
                }

                let alignment = max(buf.len(), 16);
                let padding = (alignment - (data.len() % alignment)) % alignment;
                data.resize(data.len() + padding, 0);

                let offset_in_bytes = data.len() as u16;
                let size = buf.len() as u16;

                data.extend(&buf);
                data_layout.push(ImpactVariable {
                    name,
                    config_id,
                    r#type,
                    size,
                    offset_in_bytes,
                    dbg_name,
                });
            }
        }

        Ok(ImpactProgram {
            id,
            program_guid,
            stack_size,
            used_streams,
            code,
            code_shutdown,
            data_layout,
            data,
            query_call_ids,
        })
    }

}
