use std::fmt::Display;

use indexmap::IndexMap;
use kfc::{guid::BlobGuid, reflection::TypeIndex};

mod error;
mod read;
mod serde;
mod write;

pub use error::*;
pub use read::*;

type Struct = IndexMap<String, Value>;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    None,
    Bool(bool),
    UInt(u64),
    SInt(i64),
    Float(f64),
    String(String),
    Struct(Box<Struct>),
    Array(Vec<Value>),
    Variant(Box<Variant>),
    Guid(BlobGuid),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Variant {
    pub type_index: TypeIndex,
    pub value: IndexMap<String, Value>,
}

impl Value {
    #[inline]
    pub fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }

    #[inline]
    pub fn as_none(&self) -> Option<()> {
        match self {
            Self::None => Some(()),
            _ => None,
        }
    }

    #[inline]
    pub fn is_bool(&self) -> bool {
        matches!(self, Self::Bool(_))
    }

    #[inline]
    pub fn as_bool(&self) -> Option<bool> {
        if let Self::Bool(value) = self {
            Some(*value)
        } else {
            None
        }
    }

    #[inline]
    pub fn is_u64(&self) -> bool {
        match self {
            Self::UInt(_) => true,
            Self::SInt(value) => *value >= 0,
            _ => false,
        }
    }

    #[inline]
    pub fn as_u64(&self) -> Option<u64> {
        if let Self::UInt(value) = self {
            Some(*value)
        } else if let Self::SInt(value) = self {
            if *value >= 0 {
                Some(*value as u64)
            } else {
                None
            }
        } else {
            None
        }
    }

    #[inline]
    pub fn is_i64(&self) -> bool {
        match self {
            Self::SInt(_) => true,
            Self::UInt(value) => *value <= i64::MAX as u64,
            _ => false,
        }
    }

    #[inline]
    pub fn as_i64(&self) -> Option<i64> {
        if let Self::SInt(value) = self {
            Some(*value)
        } else if let Self::UInt(value) = self {
            if *value <= i64::MAX as u64 {
                Some(*value as i64)
            } else {
                None
            }
        } else {
            None
        }
    }

    #[inline]
    pub fn is_f64(&self) -> bool {
        match self {
            Self::Float(_) => true,
            Self::UInt(_) => true,
            Self::SInt(_) => true,
            Self::String(value) => value.parse::<f64>().is_ok(),
            _ => false,
        }
    }

    #[inline]
    pub fn as_f64(&self) -> Option<f64> {
        if let Self::Float(value) = self {
            Some(*value)
        } else if let Some(value) = self.as_u64() {
            Some(value as f64)
        } else if let Some(value) = self.as_i64() {
            Some(value as f64)
        } else if let Self::String(value) = self {
            value.parse::<f64>().ok()
        } else {
            None
        }
    }

    #[inline]
    pub fn is_string(&self) -> bool {
        matches!(self, Self::String(_))
    }

    #[inline]
    pub fn as_string(&self) -> Option<&String> {
        if let Self::String(value) = self {
            Some(value)
        } else {
            None
        }
    }

    #[inline]
    pub fn as_string_mut(&mut self) -> Option<&mut String> {
        if let Self::String(value) = self {
            Some(value)
        } else {
            None
        }
    }

    #[inline]
    pub fn into_string(self) -> Option<String> {
        if let Self::String(value) = self {
            Some(value)
        } else {
            None
        }
    }

    #[inline]
    pub fn is_guid(&self) -> bool {
        matches!(self, Self::Guid(_))
    }

    #[inline]
    pub fn as_guid(&self) -> Option<&BlobGuid> {
        if let Self::Guid(value) = self {
            Some(value)
        } else {
            None
        }
    }

    #[inline]
    pub fn as_guid_mut(&mut self) -> Option<&mut BlobGuid> {
        if let Self::Guid(value) = self {
            Some(value)
        } else {
            None
        }
    }

    #[inline]
    pub fn into_guid(self) -> Option<BlobGuid> {
        if let Self::Guid(value) = self {
            Some(value)
        } else {
            None
        }
    }

    #[inline]
    pub fn is_struct(&self) -> bool {
        matches!(self, Self::Struct(_))
    }

    #[inline]
    pub fn as_struct(&self) -> Option<&Struct> {
        if let Self::Struct(value) = self {
            Some(value)
        } else {
            None
        }
    }

    #[inline]
    pub fn as_struct_mut(&mut self) -> Option<&mut Struct> {
        if let Self::Struct(value) = self {
            Some(value)
        } else {
            None
        }
    }

    #[inline]
    pub fn into_struct(self) -> Option<Struct> {
        if let Self::Struct(value) = self {
            Some(*value)
        } else {
            None
        }
    }

    #[inline]
    pub fn is_array(&self) -> bool {
        matches!(self, Self::Array(_))
    }

    #[inline]
    pub fn as_array(&self) -> Option<&[Self]> {
        if let Self::Array(value) = self {
            Some(value)
        } else {
            None
        }
    }

    #[inline]
    pub fn as_array_mut(&mut self) -> Option<&mut Vec<Self>> {
        if let Self::Array(value) = self {
            Some(value)
        } else {
            None
        }
    }

    #[inline]
    pub fn into_array(self) -> Option<Vec<Self>> {
        if let Self::Array(value) = self {
            Some(value)
        } else {
            None
        }
    }

    #[inline]
    pub fn is_variant(&self) -> bool {
        matches!(self, Self::Variant(_))
    }

    #[inline]
    pub fn as_variant(&self) -> Option<&Variant> {
        if let Self::Variant(value) = self {
            Some(value)
        } else {
            None
        }
    }

    #[inline]
    pub fn as_variant_mut(&mut self) -> Option<&mut Variant> {
        if let Self::Variant(value) = self {
            Some(value)
        } else {
            None
        }
    }

    #[inline]
    pub fn into_variant(self) -> Option<Variant> {
        if let Self::Variant(value) = self {
            Some(*value)
        } else {
            None
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => write!(f, "None"),
            Self::Bool(value) => write!(f, "{value}"),
            Self::UInt(value) => write!(f, "{value}"),
            Self::SInt(value) => write!(f, "{value}"),
            Self::Float(value) => write!(f, "{value}"),
            Self::String(value) => write!(f, "{value}"),
            Self::Guid(value) => write!(f, "{value}"),
            Self::Struct(value) => write!(f, "{value:?}"),
            Self::Array(value) => write!(f, "{value:?}"),
            Self::Variant(value) => write!(f, "{value:?}"),
        }
    }
}


macro_rules! impl_into {
    ($type:ty, $variant:ident) => {
        impl From<$type> for Value {
            fn from(value: $type) -> Self {
                Self::$variant(value.into())
            }
        }
    };
}

impl_into!(bool, Bool);
impl_into!(u8, UInt);
impl_into!(i8, SInt);
impl_into!(u16, UInt);
impl_into!(i16, SInt);
impl_into!(u32, UInt);
impl_into!(i32, SInt);
impl_into!(u64, UInt);
impl_into!(i64, SInt);
impl_into!(f32, Float);
impl_into!(f64, Float);
impl_into!(String, String);
impl_into!(Struct, Struct);
impl_into!(Box<Struct>, Struct);
impl_into!(Vec<Value>, Array);
impl_into!(Variant, Variant);
impl_into!(Box<Variant>, Variant);
impl_into!(BlobGuid, Guid);
