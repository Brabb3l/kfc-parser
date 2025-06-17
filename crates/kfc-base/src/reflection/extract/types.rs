use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::hash::Hash;

use crate::{reflection::TypeIndex, Hash32};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypeMetadata {
    pub index: TypeIndex,
    pub name: String,
    pub impact_name: String,
    pub qualified_name: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub namespace: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inner_type: Option<TypeIndex>,
    pub size: u32,
    pub alignment: u16,
    pub element_alignment: u16,
    pub field_count: u32,
    pub primitive_type: PrimitiveType,
    pub flags: TypeFlags,
    pub name_hash: Hash32, // pre-computed
    pub impact_hash: Hash32, // pre-computed
    pub qualified_hash: Hash32,
    pub internal_hash: Hash32,
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub struct_fields: IndexMap<String, StructFieldMetadata>,
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub enum_fields: IndexMap<String, EnumFieldMetadata>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_value: Option<Vec<u8>>,
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub attributes: IndexMap<String, Attribute>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StructFieldMetadata {
    pub name: String,
    pub r#type: TypeIndex,
    pub data_offset: u64,
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub attributes: IndexMap<String, Attribute>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Attribute {
    pub name: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub namespace: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub r#type: Option<TypeIndex>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnumFieldMetadata {
    pub name: String,
    pub value: u64,
}

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
    pub struct TypeFlags: u8 {
        const NONE = 0x00;
        const HAS_DS = 0x01;
        const HAS_BLOB_ARRAY = 0x02;
        const HAS_BLOB_STRING = 0x04;
        const HAS_BLOB_OPTIONAL = 0x08;
        const HAS_BLOB_VARIANT = 0x10;
        const IS_GPU_UNIFORM = 0x20;
        const IS_GPU_STORAGE = 0x40;
        const IS_GPU_CONSTANT = 0x80;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[repr(u8)]
pub enum PrimitiveType {
    None,
    Bool,
    #[serde(rename = "UINT8")]
    UInt8,
    #[serde(rename = "SINT8")]
    SInt8,
    #[serde(rename = "UINT16")]
    UInt16,
    #[serde(rename = "SINT16")]
    SInt16,
    #[serde(rename = "UINT32")]
    UInt32,
    #[serde(rename = "SINT32")]
    SInt32,
    #[serde(rename = "UINT64")]
    UInt64,
    #[serde(rename = "SINT64")]
    SInt64,
    Float32,
    Float64,
    Enum,
    Bitmask8,
    Bitmask16,
    Bitmask32,
    Bitmask64,
    Typedef,
    Struct,
    StaticArray,
    DsArray,
    DsString,
    DsOptional,
    DsVariant,
    BlobArray,
    BlobString,
    BlobOptional,
    BlobVariant,
    ObjectReference,
    Guid,
}

impl PrimitiveType {
    pub fn from_u8(value: u8) -> Self {
        match value {
            0x0 => Self::None,
            0x1 => Self::Bool,
            0x2 => Self::UInt8,
            0x3 => Self::SInt8,
            0x4 => Self::UInt16,
            0x5 => Self::SInt16,
            0x6 => Self::UInt32,
            0x7 => Self::SInt32,
            0x8 => Self::UInt64,
            0x9 => Self::SInt64,
            0xA => Self::Float32,
            0xB => Self::Float64,
            0xC => Self::Enum,
            0xD => Self::Bitmask8,
            0xE => Self::Bitmask16,
            0xF => Self::Bitmask32,
            0x10 => Self::Bitmask64,
            0x11 => Self::Typedef,
            0x12 => Self::Struct,
            0x13 => Self::StaticArray,
            0x14 => Self::DsArray,
            0x15 => Self::DsString,
            0x16 => Self::DsOptional,
            0x17 => Self::DsVariant,
            0x18 => Self::BlobArray,
            0x19 => Self::BlobString,
            0x1A => Self::BlobOptional,
            0x1B => Self::BlobVariant,
            0x1C => Self::ObjectReference,
            0x1D => Self::Guid,
            _ => panic!("Invalid PrimitiveType: 0x{:X}", value),
        }
    }

    pub fn to_u8(&self) -> u8 {
        match self {
            Self::None => 0x0,
            Self::Bool => 0x1,
            Self::UInt8 => 0x2,
            Self::SInt8 => 0x3,
            Self::UInt16 => 0x4,
            Self::SInt16 => 0x5,
            Self::UInt32 => 0x6,
            Self::SInt32 => 0x7,
            Self::UInt64 => 0x8,
            Self::SInt64 => 0x9,
            Self::Float32 => 0xA,
            Self::Float64 => 0xB,
            Self::Enum => 0xC,
            Self::Bitmask8 => 0xD,
            Self::Bitmask16 => 0xE,
            Self::Bitmask32 => 0xF,
            Self::Bitmask64 => 0x10,
            Self::Typedef => 0x11,
            Self::Struct => 0x12,
            Self::StaticArray => 0x13,
            Self::DsArray => 0x14,
            Self::DsString => 0x15,
            Self::DsOptional => 0x16,
            Self::DsVariant => 0x17,
            Self::BlobArray => 0x18,
            Self::BlobString => 0x19,
            Self::BlobOptional => 0x1A,
            Self::BlobVariant => 0x1B,
            Self::ObjectReference => 0x1C,
            Self::Guid => 0x1D,
        }
    }
}
