use std::collections::BTreeMap;
use std::fmt::Debug;

use crate::types::{CrpfGuid, Guid, PrimitiveType};

mod read;

const CRPF_MAGIC: u32 = 0x46505243;
const CTCB_MAGIC: u32 = 0x42435443;
const KBF_MAGIC: u32 = 0x3046424B;

#[derive(Debug)]
pub struct Crpf {
    pub header: CrpfHeader,
    pub guid: CrpfGuid,
    pub r#type: u32,
    pub unk0: u32, // 2
    pub data_guids: Vec<Guid>,
    pub crpf_guids: Vec<CrpfGuid>,

    pub kbf: Kbf,
    pub ctcb: Ctcb,

    // other
    pub name: String,
}

#[derive(Debug)]
pub struct CrpfHeader {
    pub magic: u32,

    pub unk0: u32, // 2
    pub unk1: u32, // 0
    pub unk2: u32, // 0

    pub guid_offset: u64,
    pub unk3: u32, // 1
    pub crpf_guids_offset: u64,
    pub crpf_guids_count: u32,
    pub data_guids_offset: u64,
    pub data_guids_count: u32,
    pub kbf_offset: u64,
    pub kbf_size: u32,
    pub name_data_offset: u64,
    pub name_data_unk: u32, // var
    pub name_offset: u64,
    pub name_len: u32,
}

pub struct Kbf {
    pub header: KbfHeader,
    pub data: Vec<u8>, // 8-byte aligned
}

#[derive(Debug)]
pub struct KbfHeader {
    pub magic: u32,
    pub name_offset: u32,
    pub crpf_type: u32,
    pub r#type: u32,
    pub kbf_content_size: u32,
    pub ctcb_size: u32,
    pub unk1: u64, // 0
    pub unk2: u64, // 0
    pub unk3: u64, // 0
}

#[derive(Debug)]
pub struct Ctcb {
    pub header: CtcbHeader,
    pub namespaces: Vec<CtcbNamespace>,
    pub type_entries: Vec<CtcbTypeEntry>,
    pub name_entries: BTreeMap<u16, String>, // 4-byte aligned
    pub value_entries: Vec<CtcbFieldInfo>,
}

#[derive(Debug)]
pub struct CtcbHeader {
    pub magic: u32,
    pub unk0: u32,
    pub namespace_offset: u64,
    pub namespace_count: u32,
    pub type_table_offset: u64,
    pub type_table_count: u32,
    pub name_table_offset: u64,
    pub name_table_size: u32,
    pub value_table_offset: u64,
    pub value_table_count: u32,
    pub enum_table_offset: u64,
    pub enum_table_count: u32,
}

#[derive(Debug)]
pub struct CtcbNamespace {
    pub name_offset: u16,
    pub unk1: u16,
}

#[derive(Debug)]
pub struct CtcbTypeEntry {
    pub name0_offset: u16,
    pub name1_offset: u16,
    pub namespace_index: u16,
    pub ref_type_index: u16,
    pub size: u32,
    pub field_count: u32,
    pub primitive_type: PrimitiveType,
    pub unk0: u8,
    pub unk1: u16,
    pub type_hash1: u32,
    pub type_hash2: u32,
    pub field_info_start_index: u16,
    pub enum_index: u16,
    pub enum_values: BTreeMap<u64, String>,
}

#[derive(Debug)]
pub struct CtcbFieldInfo {
    pub key_offset: u16,
    pub type_index: u16,
    pub offset: u32,
}

// Debug

impl Debug for Kbf {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("KBF")
            .field("header", &self.header)
            .field("data", &format_args!("{} bytes", self.data.len()))
            .finish()
    }
}

// util

impl Crpf {
    pub fn get_opt_type(&self, type_index: u16) -> Option<&CtcbTypeEntry> {
        if type_index == 0 {
            return None;
        }

        self.ctcb
            .type_entries
            .get(type_index as usize - 1)
    }

    pub fn get_type(&self, type_index: u16) -> anyhow::Result<&CtcbTypeEntry> {
        self.ctcb
            .type_entries
            .get(type_index as usize - 1)
            .ok_or_else(|| anyhow::anyhow!("Type entry not found for index: {}", type_index))
    }

    pub fn get_type_by_hash(&self, type_hash1: u32) -> anyhow::Result<&CtcbTypeEntry> {
        self.ctcb
            .type_entries
            .iter()
            .find(|entry| entry.type_hash1 == type_hash1)
            .ok_or_else(|| anyhow::anyhow!("Type entry not found for type hash: {}", type_hash1))
    }

    pub fn get_field_info(&self, index: u32) -> anyhow::Result<&CtcbFieldInfo> {
        self.ctcb
            .value_entries
            .get(index as usize - 1)
            .ok_or_else(|| anyhow::anyhow!("Field info not found for index: {}", index))
    }

    pub fn get_name(&self, offset: u16) -> anyhow::Result<&String> {
        self.ctcb
            .name_entries
            .get(&offset)
            .ok_or_else(|| anyhow::anyhow!("Name not found for key_offset: {}", offset))
    }
}
