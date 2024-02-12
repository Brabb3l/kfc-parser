use crate::crpf::Crpf;
use crate::error::CrpfNodeParseError;
use crate::types::{Bitmask16, Bitmask32, Bitmask64, Bitmask8, BlobOptional, BlobVariant, CrpfEnum, CrpfNode, CrpfStruct, CrpfTypedef, Guid, ObjectReference};

pub mod structs;
pub mod enums;
mod export;

pub trait FromCrpf: Sized {
    fn parse(node: &CrpfNode, crpf: &Crpf, parent: Option<&CrpfStruct>) -> Result<Self, CrpfNodeParseError>;
}

pub trait FromCrpfWrapped<T: FromCrpf>: Sized {
    fn parse(node: &CrpfNode, crpf: &Crpf, parent: Option<&CrpfStruct>) -> Result<Self, CrpfNodeParseError>;
}

macro_rules! convert_node {
    ($parent:ident, $node:ident, $primitive_type:ident) => {
        match $node {
            $crate::types::CrpfNode::$primitive_type(value) => value,
            _ => return Err($crate::error::CrpfNodeParseError::InvalidFieldType(
                $node.primitive_type(),
                $crate::types::PrimitiveType::$primitive_type,
                $parent.map(|s| s.type_name.clone()).unwrap_or_else(|| "None".to_string())
            ))
        }
    };
}

macro_rules! impl_from_crpf_deref {
    ($type:ty, $primitive_type:ident) => {
        impl FromCrpf for $type {
            fn parse(node: &CrpfNode, _crpf: &Crpf, parent: Option<&CrpfStruct>) -> Result<Self, CrpfNodeParseError> {
                Ok(*convert_node!(parent, node, $primitive_type))
            }
        }
    };
}

macro_rules! impl_from_crpf {
    ($type:ty, $primitive_type:ident) => {
        impl FromCrpf for $type {
            fn parse(node: &CrpfNode, _crpf: &Crpf, parent: Option<&CrpfStruct>) -> Result<Self, CrpfNodeParseError> {
                Ok(convert_node!(parent, node, $primitive_type).clone())
            }
        }
    };
}

impl_from_crpf_deref!(bool, Bool);
impl_from_crpf_deref!(u8, UInt8);
impl_from_crpf_deref!(i8, SInt8);
impl_from_crpf_deref!(u16, UInt16);
impl_from_crpf_deref!(i16, SInt16);
impl_from_crpf_deref!(u32, UInt32);
impl_from_crpf_deref!(i32, SInt32);
impl_from_crpf_deref!(u64, UInt64);
impl_from_crpf_deref!(i64, SInt64);
impl_from_crpf_deref!(f32, Float32);
impl_from_crpf_deref!(f64, Float64);
impl_from_crpf!(CrpfEnum, Enum);
impl_from_crpf!(Bitmask8, Bitmask8);
impl_from_crpf!(Bitmask16, Bitmask16);
impl_from_crpf!(Bitmask32, Bitmask32);
impl_from_crpf!(Bitmask64, Bitmask64);
impl_from_crpf!(CrpfTypedef, Typedef);
impl_from_crpf!(CrpfStruct, Struct);

impl_from_crpf!(String, BlobString);
impl_from_crpf!(BlobOptional, BlobOptional);
impl_from_crpf!(BlobVariant, BlobVariant);
impl_from_crpf!(ObjectReference, ObjectReference);
impl_from_crpf!(Guid, Guid);

macro_rules! field_type {
    (Bool) => { bool };
    (UInt8) => { u8 };
    (SInt8) => { i8 };
    (UInt16) => { u16 };
    (SInt16) => { i16 };
    (UInt32) => { u32 };
    (SInt32) => { i32 };
    (UInt64) => { u64 };
    (SInt64) => { i64 };
    (Float32) => { f32 };
    (Float64) => { f64 };
    (Enum) => { $crate::types::CrpfEnum };
    (Bitmask8) => { $crate::types::Bitmask8 };
    (Bitmask16) => { $crate::types::Bitmask16 };
    (Bitmask32) => { $crate::types::Bitmask32 };
    (Bitmask64) => { $crate::types::Bitmask64 };
    // (Typedef) => { CrpfTypedef };
    (Struct) => { $crate::types::CrpfStruct };

    (BlobArray<$g0:ident $(, $g1:ident)?>) => { Vec<super::field_type!($g0 $(, $g1)?)> };
    (BlobString) => { String };
    (BlobOptional) => { $crate::types::BlobOptional };
    (BlobVariant) => { $crate::types::BlobVariant };
    (ObjectReference) => { $crate::types::ObjectReference };
    (Guid) => { $crate::types::Guid };

    ($ty:ty) => { $ty };
}

macro_rules! crpf_struct {
    (
        $(#[$attr:meta])*
        $name:ident {
            $(
                $(#[$field_attr:meta])*
                $field:ident($field_name:expr): $field_type:ident $(< $g0:ident $(, $g1:ident)? >)?
            ),* $(,)?
        }
    ) => {
        #[derive(Debug)]
        $(#[$attr])*
        pub struct $name {
            $(
                $(#[$field_attr])*
                pub $field: super::field_type!($field_type $(<$g0 $(, $g1)? >)?),
            )*
        }

        impl $crate::file::FromCrpf for $name {
            fn parse(
                node: &$crate::types::CrpfNode,
                crpf: &$crate::crpf::Crpf,
                parent: Option<&$crate::types::CrpfStruct>
            ) -> Result<Self, $crate::error::CrpfNodeParseError> {
                let node = super::convert_node!(parent, node, Struct);

                Ok(Self {
                    $(
                        $field: $crate::file::FromCrpf::parse(node.get_raw($field_name)?, crpf, Some(node))?
                    ),*
                })
            }
        }
    };
}

macro_rules! crpf_enum {
    (
        $(#[$attr:meta])*
        $name:ident {
            $(
                $(#[$field_attr:meta])*
                $field:ident($field_name:expr) = $field_value:expr
            ),* $(,)?
        }
    ) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        $(#[$attr])*
        pub enum $name {
            $(
                $(#[$field_attr])*
                $field
            ),*
        }

        #[allow(dead_code)]
        impl $name {
            pub fn from_value(value: u64) -> Option<Self> {
                match value {
                    $(
                        $field_value => Some(Self::$field),
                    )*
                    _ => None
                }
            }

            pub fn from_value_name(name: &str) -> Option<Self> {
                match name {
                    $(
                        $field_name => Some(Self::$field),
                    )*
                    _ => None
                }
            }
        }

        impl $crate::file::FromCrpf for $name {
            fn parse(
                node: &$crate::types::CrpfNode,
                _crpf: &$crate::crpf::Crpf,
                parent: Option<&$crate::types::CrpfStruct>
            ) -> Result<Self, $crate::error::CrpfNodeParseError> {
                let node = super::convert_node!(parent, node, Enum);
                let enum_value = node.value.as_u64()
                    .ok_or_else(|| $crate::error::CrpfNodeParseError::TypeMismatch(
                        node.value.primitive_type(),
                        &[
                            $crate::types::PrimitiveType::UInt8,
                            $crate::types::PrimitiveType::UInt16,
                            $crate::types::PrimitiveType::UInt32,
                            $crate::types::PrimitiveType::UInt64
                        ]
                    ))?;

                $name::from_value(enum_value)
                    .ok_or_else(|| $crate::error::CrpfNodeParseError::InvalidEnumValue(
                        node.value_name.clone()
                    ))
            }
        }
    };
}

impl<T: FromCrpf> FromCrpf for Vec<T> {
    fn parse(node: &CrpfNode, crpf: &Crpf, parent: Option<&CrpfStruct>) -> Result<Self, CrpfNodeParseError> {
        let array = convert_node!(parent, node, BlobArray);
        let mut vec = Vec::with_capacity(array.len());

        for item in array {
            vec.push(T::parse(item, crpf, parent)?);
        }

        Ok(vec)
    }
}

use crpf_struct;
use crpf_enum;
use convert_node;
use field_type;
