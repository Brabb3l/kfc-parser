use std::any::TypeId;

use kfc::{guid::Guid, reflection::PrimitiveType, resource::{mapped::MappingError, value::Value}};
use mlua::AnyUserData;
use thiserror::Error;

use crate::{alias::{MappedBitmask, MappedValue, TypeHandle}, env::value::{convert_value_to_lua, mapped::{MappedArrayValue, MappedStructValue, MappedVariantValue}, name_of, simple::{ArrayValue, StructValue, VariantValue}, util::TreePath}, lua::{LuaError, LuaValue}};

#[derive(Debug, Error)]
#[error("at {path}: {kind}")]
pub struct LuaConversionError {
    path: String,
    kind: LuaConversionErrorKind,
}

#[derive(Debug, Error)]
pub enum LuaConversionErrorKind {
    #[error("expected `{expected}` but found `{found}`")]
    IncompatibleType {
        expected: String,
        found: String,
    },
    #[error("invalid guid `{0}`")]
    InvalidGuid(String),

    #[error("{0}")]
    Lua(#[from] mlua::Error),
    #[error("{0}")]
    Mapping(#[from] MappingError),
}

pub struct Converter {
    pub path: TreePath,
}

impl Converter {

    pub fn convert(
        value: &LuaValue,
        r#type: &TypeHandle,
    ) -> Result<Value, LuaConversionError> {
        let mut converter = Self {
            path: TreePath::new(),
        };

        let result = converter.process(value, r#type);

        match result {
            Ok(value) => Ok(value),
            Err(err) => Err(LuaConversionError {
                path: converter.path.to_string(),
                kind: err,
            }),
        }
    }

    pub fn process(
        &mut self,
        value: &LuaValue,
        r#type: &TypeHandle,
    ) -> Result<Value, LuaConversionErrorKind> {
        let lua_value = match r#type.primitive_type {
            PrimitiveType::None => self.convert_nil(value)?,
            PrimitiveType::Bool => self.convert_bool(value)?.into(),
            PrimitiveType::UInt8 => self.convert_uint(value, 0, u8::MAX as u64)?.into(),
            PrimitiveType::SInt8 => self.convert_sint(value, i8::MIN as i64, i8::MAX as i64)?.into(),
            PrimitiveType::UInt16 => self.convert_uint(value, 0, u16::MAX as u64)?.into(),
            PrimitiveType::SInt16 => self.convert_sint(value, i16::MIN as i64, i16::MAX as i64)?.into(),
            PrimitiveType::UInt32 => self.convert_uint(value, 0, u32::MAX as u64)?.into(),
            PrimitiveType::SInt32 => self.convert_sint(value, i32::MIN as i64, i32::MAX as i64)?.into(),
            PrimitiveType::UInt64 => self.convert_uint(value, 0, u64::MAX)?.into(),
            PrimitiveType::SInt64 => self.convert_sint(value, i64::MIN, i64::MAX)?.into(),
            PrimitiveType::Float32 => self.convert_float(value)?.into(),
            PrimitiveType::Float64 => self.convert_float(value)?.into(),
            PrimitiveType::Enum => self.convert_enum(value, r#type)?,
            PrimitiveType::Bitmask8 => self.convert_bitmask(value, r#type)?,
            PrimitiveType::Bitmask16 => self.convert_bitmask(value, r#type)?,
            PrimitiveType::Bitmask32 => self.convert_bitmask(value, r#type)?,
            PrimitiveType::Bitmask64 => self.convert_bitmask(value, r#type)?,
            PrimitiveType::Typedef => self.convert_typedef(value, r#type)?,
            PrimitiveType::Struct => self.convert_struct(value, r#type)?,
            PrimitiveType::StaticArray => self.convert_static_array(value, r#type)?,
            PrimitiveType::DsArray => todo!(),
            PrimitiveType::DsString => todo!(),
            PrimitiveType::DsOptional => todo!(),
            PrimitiveType::DsVariant => todo!(),
            PrimitiveType::BlobArray => self.convert_blob_array(value, r#type)?,
            PrimitiveType::BlobString => self.convert_blob_string(value)?,
            PrimitiveType::BlobOptional => self.convert_blob_optional(value, r#type)?,
            PrimitiveType::BlobVariant => self.convert_blob_variant(value, r#type)?,
            PrimitiveType::ObjectReference => self.convert_object_reference(value)?,
            PrimitiveType::Guid => self.convert_guid(value)?,
        };

        Ok(lua_value)
    }

    #[inline]
    fn convert_nil(
        &mut self,
        value: &LuaValue,
    ) -> Result<Value, LuaConversionErrorKind> {
        match value {
            LuaValue::Nil => Ok(Value::None),
            _ => Err(LuaConversionErrorKind::IncompatibleType {
                expected: "nil".to_string(),
                found: value.to_string()?,
            }),
        }
    }

    #[inline]
    fn convert_bool(
        &mut self,
        value: &LuaValue,
    ) -> Result<bool, LuaConversionErrorKind> {
        match value {
            LuaValue::Boolean(b) => Ok(*b),
            _ => Err(LuaConversionErrorKind::IncompatibleType {
                expected: "boolean".to_string(),
                found: value.to_string()?,
            }),
        }
    }

    #[inline]
    fn convert_uint(
        &mut self,
        value: &LuaValue,
        min: u64,
        max: u64,
    ) -> Result<u64, LuaConversionErrorKind> {
        match value {
            LuaValue::Integer(i) if *i as u64 >= min && *i as u64 <= max => Ok(*i as u64),
            LuaValue::Number(n) if n.fract() == 0.0 && *n >= min as f64 && *n <= max as f64 => Ok(*n as u64),
            _ => Err(LuaConversionErrorKind::IncompatibleType {
                expected: format!("unsigned integer in range [{min}, {max}]"),
                found: value.to_string()?,
            }),
        }
    }

    #[inline]
    fn convert_sint(
        &mut self,
        value: &LuaValue,
        min: i64,
        max: i64,
    ) -> Result<i64, LuaConversionErrorKind> {
        match value {
            LuaValue::Integer(i) if *i >= min && *i <= max => Ok(*i),
            LuaValue::Number(n) if n.fract() == 0.0 && *n >= min as f64 && *n <= max as f64 => Ok(*n as i64),
            _ => Err(LuaConversionErrorKind::IncompatibleType {
                expected: format!("signed integer in range [{min}, {max}]"),
                found: value.to_string()?,
            }),
        }
    }

    #[inline]
    fn convert_float(
        &mut self,
        value: &LuaValue,
    ) -> Result<f64, LuaConversionErrorKind> {
        match value {
            LuaValue::Integer(i) => Ok(*i as f64),
            LuaValue::Number(n) => Ok(*n),
            _ => Err(LuaConversionErrorKind::IncompatibleType {
                expected: "float".to_string(),
                found: value.to_string()?,
            }),
        }
    }

    #[inline]
    fn convert_enum(
        &mut self,
        value: &LuaValue,
        r#type: &TypeHandle,
    ) -> Result<Value, LuaConversionErrorKind> {
        match &value {
            LuaValue::String(s) => {
                if let Some(field) = r#type.enum_fields.get(&s.to_string_lossy()) {
                    return Ok(Value::String(field.name.clone()))
                }
            },
            LuaValue::Integer(i) => {
                let i = *i as u64;

                if let Some(field) = r#type.enum_fields.values().find(|f| f.value == i) {
                    return Ok(Value::String(field.name.clone()))
                }
            },
            _ => {},
        };

        let enum_values = r#type.enum_fields
            .keys()
            .map(|k| format!("\"{k}\""))
            .collect::<Vec<String>>()
            .join(", ");

        Err(LuaConversionErrorKind::IncompatibleType {
            expected: format!("one of [{enum_values}]"),
            found: value.to_string()?,
        })
    }

    #[inline]
    fn convert_bitmask(
        &mut self,
        value: &LuaValue,
        r#type: &TypeHandle,
    ) -> Result<Value, LuaConversionErrorKind> {
        match value {
            LuaValue::UserData(ud) => self.try_convert_struct_ud(ud, r#type),
            _ => Err(LuaConversionErrorKind::IncompatibleType {
                expected: name_of(r#type),
                found: value.to_string()?,
            }),
        }
    }

    #[inline]
    fn convert_typedef(
        &mut self,
        value: &LuaValue,
        r#type: &TypeHandle,
    ) -> Result<Value, LuaConversionErrorKind> {
        let inner_type = r#type.inner_type()
            .expect("invalid typedef type");

        self.process(value, &inner_type)
    }

    #[inline]
    fn convert_struct(
        &mut self,
        value: &LuaValue,
        r#type: &TypeHandle,
    ) -> Result<Value, LuaConversionErrorKind> {
        match value {
            LuaValue::UserData(ud) => self.try_convert_struct_ud(ud, r#type),
            _ => Err(LuaConversionErrorKind::IncompatibleType {
                expected: name_of(r#type),
                found: value.to_string()?,
            }),
        }
    }

    #[inline]
    fn convert_static_array(
        &mut self,
        value: &LuaValue,
        r#type: &TypeHandle,
    ) -> Result<Value, LuaConversionErrorKind> {
        match &value {
            LuaValue::UserData(ud) => self.try_convert_array_ud(ud, r#type),
            _ => {
                Err(LuaConversionErrorKind::IncompatibleType {
                    expected: name_of(r#type),
                    found: value.to_string()?,
                })
            },
        }
    }

    #[inline]
    fn convert_blob_array(
        &mut self,
        value: &LuaValue,
        r#type: &TypeHandle,
    ) -> Result<Value, LuaConversionErrorKind> {
        match &value {
            LuaValue::UserData(user_data) => self.try_convert_array_ud(user_data, r#type),
            _ => Err(LuaConversionErrorKind::IncompatibleType {
                expected: name_of(r#type),
                found: value.to_string()?,
            }),
        }
    }

    #[inline]
    fn convert_blob_string(
        &mut self,
        value: &LuaValue,
    ) -> Result<Value, LuaConversionErrorKind> {
        match value {
            LuaValue::String(s) => Ok(Value::String(s.to_string_lossy())),
            _ => Err(LuaConversionErrorKind::IncompatibleType {
                expected: "string".to_string(),
                found: value.to_string()?,
            }),
        }
    }

    #[inline]
    fn convert_blob_optional(
        &mut self,
        value: &LuaValue,
        r#type: &TypeHandle,
    ) -> Result<Value, LuaConversionErrorKind> {
        match value {
            LuaValue::Nil => Ok(Value::None),
            _ => {
                let inner_type = r#type.inner_type()
                    .expect("invalid optional type");

                self.process(value, &inner_type)
            }
        }
    }

    #[inline]
    fn convert_blob_variant(
        &mut self,
        value: &LuaValue,
        r#type: &TypeHandle,
    ) -> Result<Value, LuaConversionErrorKind> {
        match value {
            LuaValue::UserData(ud) => {
                self.try_convert_variant_ud(ud, r#type)
            },
            _ => Err(LuaConversionErrorKind::IncompatibleType {
                expected: name_of(r#type),
                found: value.to_string()?,
            }),
        }
    }


    #[inline]
    fn convert_object_reference(
        &mut self,
        value: &LuaValue,
    ) -> Result<Value, LuaConversionErrorKind> {
        match value {
            LuaValue::String(s) => {
                let guid = s.to_string_lossy();
                let guid = Guid::parse(&guid)
                    .ok_or_else(|| LuaConversionErrorKind::InvalidGuid(guid))?;

                Ok(Value::Guid(guid))
            },
            _ => Err(LuaConversionErrorKind::IncompatibleType {
                expected: "object reference".to_string(),
                found: value.to_string()?,
            }),
        }
    }

    #[inline]
    fn convert_guid(
        &mut self,
        value: &LuaValue,
    ) -> Result<Value, LuaConversionErrorKind> {
        match value {
            LuaValue::String(s) => {
                let guid = s.to_string_lossy();
                let guid = Guid::parse(&guid)
                    .ok_or_else(|| LuaConversionErrorKind::InvalidGuid(guid))?;

                Ok(Value::Guid(guid))
            },
            _ => Err(LuaConversionErrorKind::IncompatibleType {
                expected: "guid".to_string(),
                found: value.to_string()?,
            }),
        }
    }

    fn try_convert_struct_ud(
        &mut self,
        ud: &AnyUserData,
        r#type: &TypeHandle,
    ) -> Result<Value, LuaConversionErrorKind> {
        let type_id = match ud.type_id() {
            Some(type_id) => type_id,
            None => panic!("UserData has no type_id"),
        };

        match type_id {
            id if id == TypeId::of::<StructValue>() => {
                let struct_value = ud.borrow::<StructValue>()?;

                if !struct_value.is(r#type) {
                    return Err(LuaConversionErrorKind::IncompatibleType {
                        expected: name_of(r#type),
                        found: name_of(struct_value.r#type()),
                    });
                }

                struct_value.convert(self)
            },
            id if id == TypeId::of::<MappedStructValue>() => {
                let struct_value = ud.borrow::<MappedStructValue>()?;

                if !struct_value.is(r#type) {
                    return Err(LuaConversionErrorKind::IncompatibleType {
                        expected: name_of(r#type),
                        found: name_of(struct_value.r#type()),
                    });
                }

                struct_value.convert(self)
            },
            _ => Err(LuaConversionErrorKind::IncompatibleType {
                expected: name_of(r#type),
                found: format!("userdata `{type_id:?}`"),
            })
        }
    }

    fn try_convert_array_ud(
        &mut self,
        ud: &AnyUserData,
        r#type: &TypeHandle,
    ) -> Result<Value, LuaConversionErrorKind> {
        let type_id = match ud.type_id() {
            Some(type_id) => type_id,
            None => panic!("UserData has no type_id"),
        };

        match type_id {
            id if id == TypeId::of::<ArrayValue>() => {
                let array_value = ud.borrow::<ArrayValue>()?;

                if !array_value.is(r#type) {
                    return Err(LuaConversionErrorKind::IncompatibleType {
                        expected: name_of(r#type),
                        found: name_of(array_value.r#type()),
                    });
                }

                array_value.convert(self)
            },
            id if id == TypeId::of::<MappedArrayValue>() => {
                let array_value = ud.borrow::<MappedArrayValue>()?;

                if !array_value.is(r#type) {
                    return Err(LuaConversionErrorKind::IncompatibleType {
                        expected: name_of(r#type),
                        found: name_of(array_value.r#type()),
                    });
                }

                array_value.convert(self)
            },
            _ => Err(LuaConversionErrorKind::IncompatibleType {
                expected: name_of(r#type),
                found: format!("userdata `{type_id:?}`"),
            })
        }
    }

    fn try_convert_variant_ud(
        &mut self,
        ud: &AnyUserData,
        r#type: &TypeHandle,
    ) -> Result<Value, LuaConversionErrorKind> {
        let type_id = match ud.type_id() {
            Some(type_id) => type_id,
            None => panic!("UserData has no type_id"),
        };

        match type_id {
            id if id == TypeId::of::<VariantValue>() => {
                let variant_value = ud.borrow::<VariantValue>()?;

                if !variant_value.is(r#type) {
                    return Err(LuaConversionErrorKind::IncompatibleType {
                        expected: name_of(r#type),
                        found: name_of(variant_value.r#type()),
                    });
                }

                variant_value.convert(self)
            },
            id if id == TypeId::of::<MappedVariantValue>() => {
                let variant_value = ud.borrow::<MappedVariantValue>()?;

                if !variant_value.is(r#type) {
                    return Err(LuaConversionErrorKind::IncompatibleType {
                        expected: name_of(r#type),
                        found: name_of(variant_value.r#type()),
                    });
                }

                variant_value.convert(self)
            },
            _ => Err(LuaConversionErrorKind::IncompatibleType {
                expected: name_of(r#type),
                found: format!("userdata `{type_id:?}`"),
            })
        }
    }

}

pub fn value_to_lua(
    value: &MappedValue,
    lua: &mlua::Lua,
) -> mlua::Result<LuaValue> {
    let lua_value = match value {
        MappedValue::None => LuaValue::Nil,
        MappedValue::Bool(b) => LuaValue::Boolean(*b),
        MappedValue::UInt8(v) => LuaValue::Integer(*v as i64),
        MappedValue::SInt8(v) => LuaValue::Integer(*v as i64),
        MappedValue::UInt16(v) => LuaValue::Integer(*v as i64),
        MappedValue::SInt16(v) => LuaValue::Integer(*v as i64),
        MappedValue::UInt32(v) => LuaValue::Integer(*v as i64),
        MappedValue::SInt32(v) => LuaValue::Integer(*v as i64),
        MappedValue::UInt64(v) => LuaValue::Integer(*v as i64),
        MappedValue::SInt64(v) => LuaValue::Integer(*v),
        MappedValue::Float32(v) => LuaValue::Number(*v as f64),
        MappedValue::Float64(v) => LuaValue::Number(*v),
        MappedValue::Enum(v) => {
            v.name()
                .map(|n| lua.create_string(n))
                .unwrap_or_else(|| lua.create_string(v.value().to_string()))
                .map(LuaValue::String)?
        },
        MappedValue::Bitmask(v) => bitmask_to_lua(lua, v)?,
        MappedValue::Struct(v) => {
            let wrapper = MappedStructValue::new(v.clone());
            LuaValue::UserData(lua.create_userdata(wrapper)?)
        },
        MappedValue::Array(v) => {
            let wrapper = MappedArrayValue::new(v.clone());
            LuaValue::UserData(lua.create_userdata(wrapper)?)
        },
        MappedValue::String(string_value) => {
            string_value.as_str()
                .map_err(LuaError::external)
                .map(|s| lua.create_string(s))?
                .map(LuaValue::String)?
        }
        MappedValue::Optional(value) => match value.value() {
            Some(v) => convert_value_to_lua(v, lua)?,
            None => LuaValue::Nil,
        },
        MappedValue::Variant(variant_value) => {
            let wrapper = MappedVariantValue::new(variant_value.clone());
            LuaValue::UserData(lua.create_userdata(wrapper)?)
        },
        MappedValue::Guid(guid) => {
            let guid_str = guid.to_string();
            lua.create_string(&guid_str)
                .map(LuaValue::String)?
        },
        MappedValue::Reference(reference) => {
            let guid_str = reference.guid().to_string();
            lua.create_string(&guid_str)
                .map(LuaValue::String)?
        },
    };

    Ok(lua_value)
}

fn bitmask_to_lua(
    lua: &mlua::Lua,
    bitmask: &MappedBitmask,
) -> mlua::Result<LuaValue> {
    let table = lua.create_table_with_capacity(
        bitmask.value().count_ones() as usize,
        0
    )?;

    for bit in bitmask.iter() {
        let value = match bit.name() {
            Some(name) => {
                lua.create_string(name)
                    .map(LuaValue::String)?
            },
            None => {
                lua.create_string(bit.value().to_string())
                    .map(LuaValue::String)?
            },
        };

        table.push(value)?;
    }

    Ok(LuaValue::Table(table))
}
