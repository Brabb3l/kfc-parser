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
        matches!(self, Value::None)
    }

    #[inline]
    pub fn as_none(&self) -> Option<()> {
        if let Value::None = self {
            Some(())
        } else {
            None
        }
    }

    #[inline]
    pub fn is_bool(&self) -> bool {
        matches!(self, Value::Bool(_))
    }

    #[inline]
    pub fn as_bool(&self) -> Option<bool> {
        if let Value::Bool(value) = self {
            Some(*value)
        } else {
            None
        }
    }

    #[inline]
    pub fn is_u64(&self) -> bool {
        match self {
            Value::UInt(_) => true,
            Value::SInt(value) => *value >= 0,
            _ => false,
        }
    }

    #[inline]
    pub fn as_u64(&self) -> Option<u64> {
        if let Value::UInt(value) = self {
            Some(*value)
        } else if let Value::SInt(value) = self {
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
            Value::SInt(_) => true,
            Value::UInt(value) => *value <= i64::MAX as u64,
            _ => false,
        }
    }

    #[inline]
    pub fn as_i64(&self) -> Option<i64> {
        if let Value::SInt(value) = self {
            Some(*value)
        } else if let Value::UInt(value) = self {
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
            Value::Float(_) => true,
            Value::UInt(_) => true,
            Value::SInt(_) => true,
            Value::String(value) => value.parse::<f64>().is_ok(),
            _ => false,
        }
    }

    #[inline]
    pub fn as_f64(&self) -> Option<f64> {
        if let Value::Float(value) = self {
            Some(*value)
        } else if let Some(value) = self.as_u64() {
            Some(value as f64)
        } else if let Some(value) = self.as_i64() {
            Some(value as f64)
        } else if let Value::String(value) = self {
            value.parse::<f64>().ok()
        } else {
            None
        }
    }

    #[inline]
    pub fn is_string(&self) -> bool {
        matches!(self, Value::String(_))
    }

    #[inline]
    pub fn as_string(&self) -> Option<&String> {
        if let Value::String(value) = self {
            Some(value)
        } else {
            None
        }
    }

    #[inline]
    pub fn as_string_mut(&mut self) -> Option<&mut String> {
        if let Value::String(value) = self {
            Some(value)
        } else {
            None
        }
    }

    #[inline]
    pub fn into_string(self) -> Option<String> {
        if let Value::String(value) = self {
            Some(value)
        } else {
            None
        }
    }

    #[inline]
    pub fn is_guid(&self) -> bool {
        matches!(self, Value::Guid(_))
    }

    #[inline]
    pub fn as_guid(&self) -> Option<&BlobGuid> {
        if let Value::Guid(value) = self {
            Some(value)
        } else {
            None
        }
    }

    #[inline]
    pub fn as_guid_mut(&mut self) -> Option<&mut BlobGuid> {
        if let Value::Guid(value) = self {
            Some(value)
        } else {
            None
        }
    }

    #[inline]
    pub fn into_guid(self) -> Option<BlobGuid> {
        if let Value::Guid(value) = self {
            Some(value)
        } else {
            None
        }
    }

    #[inline]
    pub fn is_struct(&self) -> bool {
        matches!(self, Value::Struct(_))
    }

    #[inline]
    pub fn as_struct(&self) -> Option<&Struct> {
        if let Value::Struct(value) = self {
            Some(value)
        } else {
            None
        }
    }

    #[inline]
    pub fn as_struct_mut(&mut self) -> Option<&mut Struct> {
        if let Value::Struct(value) = self {
            Some(value)
        } else {
            None
        }
    }

    #[inline]
    pub fn into_struct(self) -> Option<Struct> {
        if let Value::Struct(value) = self {
            Some(*value)
        } else {
            None
        }
    }

    #[inline]
    pub fn is_array(&self) -> bool {
        matches!(self, Value::Array(_))
    }

    #[inline]
    pub fn as_array(&self) -> Option<&[Value]> {
        if let Value::Array(value) = self {
            Some(value)
        } else {
            None
        }
    }

    #[inline]
    pub fn as_array_mut(&mut self) -> Option<&mut Vec<Value>> {
        if let Value::Array(value) = self {
            Some(value)
        } else {
            None
        }
    }

    #[inline]
    pub fn into_array(self) -> Option<Vec<Value>> {
        if let Value::Array(value) = self {
            Some(value)
        } else {
            None
        }
    }

    #[inline]
    pub fn is_variant(&self) -> bool {
        matches!(self, Value::Variant(_))
    }

    #[inline]
    pub fn as_variant(&self) -> Option<&Variant> {
        if let Value::Variant(value) = self {
            Some(value)
        } else {
            None
        }
    }

    #[inline]
    pub fn as_variant_mut(&mut self) -> Option<&mut Variant> {
        if let Value::Variant(value) = self {
            Some(value)
        } else {
            None
        }
    }

    #[inline]
    pub fn into_variant(self) -> Option<Variant> {
        if let Value::Variant(value) = self {
            Some(*value)
        } else {
            None
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::None => write!(f, "None"),
            Value::Bool(value) => write!(f, "{}", value),
            Value::UInt(value) => write!(f, "{}", value),
            Value::SInt(value) => write!(f, "{}", value),
            Value::Float(value) => write!(f, "{}", value),
            Value::String(value) => write!(f, "{}", value),
            Value::Guid(value) => write!(f, "{}", value),
            Value::Struct(value) => write!(f, "{:?}", value),
            Value::Array(value) => write!(f, "{:?}", value),
            Value::Variant(value) => write!(f, "{:?}", value),
        }
    }
}
