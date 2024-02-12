use std::collections::HashMap;
use std::fmt::{Debug, Display};
use shared::hash::hash_filename_string;
use crate::error::CrpfNodeParseError;

pub mod read;

#[derive(Clone)]
pub struct Guid {
    pub data: [u8; 16],
}

#[derive(Clone)]
pub struct CrpfGuid {
    pub data: [u8; 16],
    pub data2: [u8; 4],
    pub data3: u32,
    pub data4: u64,
}

#[derive(Debug, Clone)]
pub enum CrpfNode {
    None,
    Bool(bool),
    UInt8(u8),
    SInt8(i8),
    UInt16(u16),
    SInt16(i16),
    UInt32(u32),
    SInt32(i32),
    UInt64(u64),
    SInt64(i64),
    Float32(f32),
    Float64(f64),
    Enum(CrpfEnum),
    Bitmask8(Bitmask8),
    Bitmask16(Bitmask16),
    Bitmask32(Bitmask32),
    Bitmask64(Bitmask64),
    Typedef(CrpfTypedef),
    Struct(CrpfStruct),
    StaticArray,
    DsArray,
    DsString,
    DsOptional,
    DsVariant,
    BlobArray(Vec<CrpfNode>),
    BlobString(String),
    BlobOptional(BlobOptional),
    BlobVariant(BlobVariant),
    ObjectReference(ObjectReference),
    Guid(Guid),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum PrimitiveType {
    None,
    Bool,
    UInt8,
    SInt8,
    UInt16,
    SInt16,
    UInt32,
    SInt32,
    UInt64,
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

#[derive(Debug, Clone)]
pub struct CrpfEnum {
    pub type_name: String,
    pub value_name: String,
    pub value: Box<CrpfNode>,
}

#[derive(Debug, Clone)]
pub struct CrpfTypedef {
    pub type_name: String,
    pub value: Box<CrpfNode>,
}

#[derive(Debug, Clone)]
pub struct CrpfStruct {
    pub type_name: String,
    pub fields: HashMap<String, CrpfNode>,
    pub parent: Option<Box<CrpfNode>>,
}

#[derive(Debug, Clone, Copy)]
pub struct Bitmask8(pub u8);

#[derive(Debug, Clone, Copy)]
pub struct Bitmask16(pub u16);

#[derive(Debug, Clone, Copy)]
pub struct Bitmask32(pub u32);

#[derive(Debug, Clone, Copy)]
pub struct Bitmask64(pub u64);

#[derive(Debug, Clone)]
pub struct BlobOptional(pub Option<Box<CrpfNode>>);

#[derive(Debug, Clone)]
pub struct BlobVariant(pub Box<CrpfNode>);

#[derive(Debug, Clone)]
pub struct ObjectReference(pub Guid);

impl Guid {
    pub fn hash(&self) -> u64 {
        hash_filename_string(&self.to_string())
    }
}

impl Debug for Guid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "\"{}\" ({})", self, self.hash())
    }
}

impl Display for Guid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for i in (0..16).rev() {
            write!(f, "{:0>2x}", self.data[i])?;
        }

        Ok(())
    }
}

impl CrpfGuid {
    pub fn hash(&self) -> u64 {
        hash_filename_string(&self.to_string())
    }
}

impl Debug for CrpfGuid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "\"{}\" ({})", self, self.hash())
    }
}

impl Display for CrpfGuid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let part_0 = u32::from_le_bytes([self.data[0], self.data[1], self.data[2], self.data[3]]);
        let part_1 = u16::from_le_bytes([self.data[4], self.data[5]]);
        let part_2 = u16::from_le_bytes([self.data[6], self.data[7]]);
        let part_3 = u16::from_be_bytes([self.data[8], self.data[9]]);
        let part_4 = u32::from_be_bytes([self.data[10], self.data[11], self.data[12], self.data[13]]);
        let part_5 = u16::from_be_bytes([self.data[14], self.data[15]]);

        let part_6 = u32::from_le_bytes([self.data2[0], self.data2[1], self.data2[2], self.data2[3]]);

        write!(f, "{:0>8x}-{:0>4x}-{:0>4x}-{:0>4x}-{:0>8x}{:0>4x}_{:0>8x}_{}", part_0, part_1, part_2, part_3, part_4, part_5, part_6, self.data4)
    }
}

macro_rules! impl_into_and_as {
    ($into_name:ident, $as_name:ident, $ty:ty, $primitive_type:ident) => {
        pub fn $into_name(self) -> Option<$ty> {
            match self {
                CrpfNode::$primitive_type(value) => Some(value),
                _ => None,
            }
        }

        pub fn $as_name(&self) -> Option<&$ty> {
            match self {
                CrpfNode::$primitive_type(value) => Some(value),
                _ => None,
            }
        }
    };
}

impl CrpfNode {
    impl_into_and_as!(into_bool, as_bool, bool, Bool);
    impl_into_and_as!(into_uint8, as_uint8, u8, UInt8);
    impl_into_and_as!(into_sint8, as_sint8, i8, SInt8);
    impl_into_and_as!(into_uint16, as_uint16, u16, UInt16);
    impl_into_and_as!(into_sint16, as_sint16, i16, SInt16);
    impl_into_and_as!(into_uint32, as_uint32, u32, UInt32);
    impl_into_and_as!(into_sint32, as_sint32, i32, SInt32);
    impl_into_and_as!(into_uint64, as_uint64, u64, UInt64);
    impl_into_and_as!(into_sint64, as_sint64, i64, SInt64);
    impl_into_and_as!(into_float32, as_float32, f32, Float32);
    impl_into_and_as!(into_float64, as_float64, f64, Float64);
    impl_into_and_as!(into_enum, as_enum, CrpfEnum, Enum);
    impl_into_and_as!(into_bitmask8, as_bitmask8, Bitmask8, Bitmask8);
    impl_into_and_as!(into_bitmask16, as_bitmask16, Bitmask16, Bitmask16);
    impl_into_and_as!(into_bitmask32, as_bitmask32, Bitmask32, Bitmask32);
    impl_into_and_as!(into_bitmask64, as_bitmask64, Bitmask64, Bitmask64);
    impl_into_and_as!(into_typedef, as_typedef, CrpfTypedef, Typedef);
    impl_into_and_as!(into_struct, as_struct, CrpfStruct, Struct);

    impl_into_and_as!(into_blob_array, as_blob_array, Vec<CrpfNode>, BlobArray);
    impl_into_and_as!(into_blob_string, as_blob_string, String, BlobString);

    pub fn as_u64(&self) -> Option<u64> {
        match self {
            CrpfNode::UInt8(value) => Some(*value as u64),
            CrpfNode::UInt16(value) => Some(*value as u64),
            CrpfNode::UInt32(value) => Some(*value as u64),
            CrpfNode::UInt64(value) => Some(*value),
            _ => None,
        }
    }

    pub fn primitive_type(&self) -> PrimitiveType {
        match self {
            CrpfNode::None => PrimitiveType::None,
            CrpfNode::Bool(_) => PrimitiveType::Bool,
            CrpfNode::UInt8(_) => PrimitiveType::UInt8,
            CrpfNode::SInt8(_) => PrimitiveType::SInt8,
            CrpfNode::UInt16(_) => PrimitiveType::UInt16,
            CrpfNode::SInt16(_) => PrimitiveType::SInt16,
            CrpfNode::UInt32(_) => PrimitiveType::UInt32,
            CrpfNode::SInt32(_) => PrimitiveType::SInt32,
            CrpfNode::UInt64(_) => PrimitiveType::UInt64,
            CrpfNode::SInt64(_) => PrimitiveType::SInt64,
            CrpfNode::Float32(_) => PrimitiveType::Float32,
            CrpfNode::Float64(_) => PrimitiveType::Float64,
            CrpfNode::Enum(_) => PrimitiveType::Enum,
            CrpfNode::Bitmask8(_) => PrimitiveType::Bitmask8,
            CrpfNode::Bitmask16(_) => PrimitiveType::Bitmask16,
            CrpfNode::Bitmask32(_) => PrimitiveType::Bitmask32,
            CrpfNode::Bitmask64(_) => PrimitiveType::Bitmask64,
            CrpfNode::Typedef(_) => PrimitiveType::Typedef,
            CrpfNode::Struct(_) => PrimitiveType::Struct,
            CrpfNode::StaticArray => PrimitiveType::StaticArray,
            CrpfNode::DsArray => PrimitiveType::DsArray,
            CrpfNode::DsString => PrimitiveType::DsString,
            CrpfNode::DsOptional => PrimitiveType::DsOptional,
            CrpfNode::DsVariant => PrimitiveType::DsVariant,
            CrpfNode::BlobArray(_) => PrimitiveType::BlobArray,
            CrpfNode::BlobString(_) => PrimitiveType::BlobString,
            CrpfNode::BlobOptional(_) => PrimitiveType::BlobOptional,
            CrpfNode::BlobVariant(_) => PrimitiveType::BlobVariant,
            CrpfNode::ObjectReference(_) => PrimitiveType::ObjectReference,
            CrpfNode::Guid(_) => PrimitiveType::Guid,
        }
    }
}

impl CrpfStruct {
    pub fn get_raw(&self, name: &str) -> Result<&CrpfNode, CrpfNodeParseError> {
        let mut node = self.fields.get(name)
            .ok_or_else(|| CrpfNodeParseError::MissingField(
                name.to_string(),
                self.type_name.clone()
            ))?;

        while let CrpfNode::Typedef(typedef) = node {
            node = &typedef.value;
        }

        Ok(node)
    }
}
