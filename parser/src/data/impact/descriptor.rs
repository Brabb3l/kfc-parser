use serde::{Deserialize, Serialize};
use crate::data::hash_types::HashKey32;
use crate::guid::BlobGuid;

pub type ProgramId = HashKey32;
pub type ImpactCommand = u32;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventStream {
    OnCollision,
    OnHit,
    OnParry,
    OnFootDown,
}


#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImpactVariable {
    pub name: HashKey32, // fnv hash of dbg_name
    pub config_id: HashKey32,
    pub r#type: HashKey32, // fnv(name) of the type
    pub size: u16,
    pub offset_in_bytes: u16,
    pub dbg_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImpactProgram {
    pub id: ProgramId, // fnv hash of the program_guid
    pub program_guid: BlobGuid, // BlobGuid part of the descriptor guid
    pub stack_size: u16,
    pub used_streams: Vec<EventStream>,
    pub code: Vec<ImpactCommand>,
    pub code_shutdown: Vec<ImpactCommand>,
    pub data_layout: Vec<ImpactVariable>,
    pub data: Vec<u8>,
}
