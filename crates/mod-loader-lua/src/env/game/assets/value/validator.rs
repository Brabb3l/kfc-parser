use std::{any::TypeId, collections::HashMap, ops::Deref};

use kfc::{guid::{ContentHash, Guid}, reflection::{LookupKey, PrimitiveType}, resource::mapped::MappingError};
use mlua::AnyUserData;
use thiserror::Error;

use crate::{alias::TypeHandle, env::{game::{Content, Resource}, value::{mapped::{MappedArrayValue, MappedStructValue, MappedVariantValue}, name_of, simple::{ArrayValue, StructValue, VariantValue}, util::TreePath}}, lua::{CastLuaExt, LuaValue}};

#[derive(Debug, Error)]
#[error("at {path}: {kind}")]
pub struct LuaValidationError {
    path: String,
    kind: LuaValidationErrorKind,
}

#[derive(Debug, Error)]
pub enum LuaValidationErrorKind {
    #[error("expected `{expected}`, got `{found}`")]
    IncompatibleType {
        expected: String,
        found: String,
    },
    #[error("missing field `{field}` in struct `{type}`")]
    MissingField {
        field: String,
        r#type: String,
    },
    #[error("invalid type `{0}`")]
    InvalidType(String),
    #[error("invalid guid `{0}`")]
    InvalidGuid(String),

    #[error("{0}")]
    Lua(#[from] mlua::Error),
    #[error("{0}")]
    Mapping(#[from] MappingError),
}

/// Validator and unifier for Lua values against type metadata.
///
/// Unified Layout:
/// None => nil
/// Bool => boolean
/// UInt8 => integer
/// SInt8 => integer
/// UInt16 => integer
/// SInt16 => integer
/// UInt32 => integer
/// SInt32 => integer
/// UInt64 => integer
/// SInt64 => integer
/// Float32 => number
/// Float64 => number
/// Enum => string|integer
/// Bitmask8 => BitmaskValue
/// Bitmask16 => BitmaskValue
/// Bitmask32 => BitmaskValue
/// Bitmask64 => BitmaskValue
/// Typedef => ...
/// Struct => StructValue|MappedStructValue
/// StaticArray => ArrayValue|MappedArrayValue
/// DsArray => todo
/// DsString => todo
/// DsOptional => todo
/// DsVariant => todo
/// BlobArray => ArrayValue|MappedArrayValue
/// BlobString => string
/// BlobOptional => nil|any
/// BlobVariant => VariantValue
/// ObjectReference => string
/// Guid => string
pub struct Validator {
    pub path: TreePath,
    pub lua: mlua::Lua,
}

impl Validator {

    pub fn validate_and_clone(
        value: &LuaValue,
        r#type: &TypeHandle,
        lua: &mlua::Lua,
    ) -> Result<LuaValue, LuaValidationError> {
        let mut converter = Self {
            path: TreePath::new(),
            lua: lua.clone(),
        };

        let result = converter.process(value, r#type);

        match result {
            Ok(value) => Ok(value),
            Err(err) => Err(LuaValidationError {
                path: converter.path.to_string(),
                kind: err,
            }),
        }
    }

    pub fn process(
        &mut self,
        value: &LuaValue,
        r#type: &TypeHandle,
    ) -> Result<LuaValue, LuaValidationErrorKind> {
        match r#type.primitive_type {
            PrimitiveType::None => self.process_nil(value),
            PrimitiveType::Bool => self.process_bool(value),
            PrimitiveType::UInt8 => self.process_uint(value, 0, u8::MAX as u64),
            PrimitiveType::SInt8 => self.process_sint(value, i8::MIN as i64, i8::MAX as i64),
            PrimitiveType::UInt16 => self.process_uint(value, 0, u16::MAX as u64),
            PrimitiveType::SInt16 => self.process_sint(value, i16::MIN as i64, i16::MAX as i64),
            PrimitiveType::UInt32 => self.process_uint(value, 0, u32::MAX as u64),
            PrimitiveType::SInt32 => self.process_sint(value, i32::MIN as i64, i32::MAX as i64),
            PrimitiveType::UInt64 => self.process_uint(value, 0, u64::MAX),
            PrimitiveType::SInt64 => self.process_sint(value, i64::MIN, i64::MAX),
            PrimitiveType::Float32 => self.process_float(value),
            PrimitiveType::Float64 => self.process_float(value),
            PrimitiveType::Enum => self.process_enum(value, r#type),
            PrimitiveType::Bitmask8 => self.process_bitmask(value, r#type),
            PrimitiveType::Bitmask16 => self.process_bitmask(value, r#type),
            PrimitiveType::Bitmask32 => self.process_bitmask(value, r#type),
            PrimitiveType::Bitmask64 => self.process_bitmask(value, r#type),
            PrimitiveType::Typedef => self.process_typedef(value, r#type),
            PrimitiveType::Struct => self.process_struct(value, r#type),
            PrimitiveType::StaticArray => self.process_static_array(value, r#type),
            PrimitiveType::DsArray => todo!(),
            PrimitiveType::DsString => todo!(),
            PrimitiveType::DsOptional => todo!(),
            PrimitiveType::DsVariant => todo!(),
            PrimitiveType::BlobArray => self.process_blob_array(value, r#type),
            PrimitiveType::BlobString => self.process_blob_string(value),
            PrimitiveType::BlobOptional => self.process_blob_optional(value, r#type),
            PrimitiveType::BlobVariant => self.process_blob_variant(value, r#type),
            PrimitiveType::ObjectReference => self.process_object_reference(value),
            PrimitiveType::Guid => self.process_guid(value),
        }
    }

    #[inline]
    fn process_nil(
        &mut self,
        value: &LuaValue,
    ) -> Result<LuaValue, LuaValidationErrorKind> {
        match value {
            LuaValue::Nil => Ok(LuaValue::Nil),
            _ => Err(LuaValidationErrorKind::IncompatibleType {
                expected: "nil".to_string(),
                found: value.to_string()?,
            }),
        }
    }

    #[inline]
    fn process_bool(
        &mut self,
        value: &LuaValue,
    ) -> Result<LuaValue, LuaValidationErrorKind> {
        match value {
            LuaValue::Boolean(b) => Ok(LuaValue::Boolean(*b)),
            _ => Err(LuaValidationErrorKind::IncompatibleType {
                expected: "boolean".to_string(),
                found: value.to_string()?,
            }),
        }
    }

    #[inline]
    fn process_uint(
        &mut self,
        value: &LuaValue,
        min: u64,
        max: u64,
    ) -> Result<LuaValue, LuaValidationErrorKind> {
        match value {
            LuaValue::Integer(i) if *i as u64 >= min && *i as u64 <= max => Ok(LuaValue::Integer(*i)),
            LuaValue::Number(n) if n.fract() == 0.0 && *n >= min as f64 && *n <= max as f64 => Ok(LuaValue::Integer(*n as i64)),
            _ => Err(LuaValidationErrorKind::IncompatibleType {
                expected: format!("unsigned integer in range [{min}, {max}]"),
                found: value.to_string()?,
            })
        }
    }

    #[inline]
    fn process_sint(
        &mut self,
        value: &LuaValue,
        min: i64,
        max: i64,
    ) -> Result<LuaValue, LuaValidationErrorKind> {
        match value {
            LuaValue::Integer(i) if *i >= min && *i <= max => Ok(LuaValue::Integer(*i)),
            LuaValue::Number(n) if n.fract() == 0.0 && *n >= min as f64 && *n <= max as f64 => Ok(LuaValue::Integer(*n as i64)),
            _ => Err(LuaValidationErrorKind::IncompatibleType {
                expected: format!("signed integer in range [{min}, {max}]"),
                found: value.to_string()?,
            }),
        }
    }

    #[inline]
    fn process_float(
        &mut self,
        value: &LuaValue,
    ) -> Result<LuaValue, LuaValidationErrorKind> {
        match value {
            LuaValue::Integer(i) => Ok(LuaValue::Number(*i as f64)),
            LuaValue::Number(n) => Ok(LuaValue::Number(*n)),
            _ => Err(LuaValidationErrorKind::IncompatibleType {
                expected: "float".to_string(),
                found: value.to_string()?,
            }),
        }
    }

    #[inline]
    fn process_enum(
        &mut self,
        value: &LuaValue,
        r#type: &TypeHandle,
    ) -> Result<LuaValue, LuaValidationErrorKind> {
        match &value {
            LuaValue::String(s) => {
                if r#type.enum_fields.get(&s.to_string_lossy()).is_some() {
                    return Ok(LuaValue::String(s.clone()))
                }
            },
            LuaValue::Integer(i) => {
                let i = *i as u64;

                if let Some(field) = r#type.enum_fields.values().find(|f| f.value == i) {
                    return Ok(LuaValue::String(self.lua.create_string(&field.name)?))
                }

                // TODO: how should it handle unknown enum values?
            },
            _ => {},
        };

        let enum_values = r#type.enum_fields
            .keys()
            .map(|k| format!("\"{k}\""))
            .collect::<Vec<String>>()
            .join(", ");

        Err(LuaValidationErrorKind::IncompatibleType {
            expected: format!("one of [{enum_values}]"),
            found: value.to_string()?,
        })
    }

    #[inline]
    fn process_bitmask(
        &mut self,
        value: &LuaValue,
        r#type: &TypeHandle,
    ) -> Result<LuaValue, LuaValidationErrorKind> {
        self.process_array_like(value, r#type)
    }

    #[inline]
    fn process_typedef(
        &mut self,
        value: &LuaValue,
        r#type: &TypeHandle,
    ) -> Result<LuaValue, LuaValidationErrorKind> {
        let inner_type = r#type.inner_type()
            .expect("invalid typedef type");

        self.process(value, &inner_type)
    }

    #[inline]
    fn process_struct(
        &mut self,
        value: &LuaValue,
        r#type: &TypeHandle,
    ) -> Result<LuaValue, LuaValidationErrorKind> {
        match value {
            LuaValue::Table(table) => {
                self.process_struct_fields(table, r#type)
            },
            LuaValue::UserData(user_data) => {
                self.try_clone_struct_ud_lua(user_data, r#type)
            },
            _ => Err(LuaValidationErrorKind::IncompatibleType {
                expected: name_of(r#type),
                found: value.to_string()?,
            }),
        }
    }

    #[inline]
    fn process_struct_fields(
        &mut self,
        table: &mlua::Table,
        r#type: &TypeHandle,
    ) -> Result<LuaValue, LuaValidationErrorKind> {
        let type_registry = r#type.type_registry();
        let mut current = r#type.deref();
        let mut field_count = current.field_count;

        while let Some(inner_type) = type_registry.get_inner_type(current) {
            current = inner_type;
            field_count += inner_type.field_count;
        }

        let mut fields = HashMap::with_capacity(field_count as usize);

        for field in r#type.iter_fields() {
            let field_type = r#type.get_field_type(&field.name)
                .expect("invalid field type");
            let field_value = match table.get::<LuaValue>(field.name.as_str()) {
                Ok(value) => value,
                Err(_) => {
                    return Err(LuaValidationErrorKind::MissingField {
                        field: field.name.clone(),
                        r#type: name_of(r#type),
                    });
                },
            };

            self.path.push(&field.name);

            let field_value = self.process(
                &field_value,
                &field_type,
            )?;

            fields.insert(field.name.clone(), field_value);

            self.path.pop();
        }

        let struct_value = StructValue::new_with_data(
            r#type.clone(),
            fields,
        )?;
        let ud = self.lua.create_userdata(struct_value)?;

        Ok(LuaValue::UserData(ud))
    }

    #[inline]
    fn process_static_array(
        &mut self,
        value: &LuaValue,
        r#type: &TypeHandle,
    ) -> Result<LuaValue, LuaValidationErrorKind> {
        self.process_array_like(value, r#type)
    }

    #[inline]
    fn process_blob_array(
        &mut self,
        value: &LuaValue,
        r#type: &TypeHandle,
    ) -> Result<LuaValue, LuaValidationErrorKind> {
        self.process_array_like(value, r#type)
    }

    #[inline]
    fn process_blob_string(
        &mut self,
        value: &LuaValue,
    ) -> Result<LuaValue, LuaValidationErrorKind> {
        match value {
            LuaValue::String(s) => Ok(LuaValue::String(s.clone())),
            _ => Err(LuaValidationErrorKind::IncompatibleType {
                expected: "string".to_string(),
                found: value.to_string()?,
            }),
        }
    }

    #[inline]
    fn process_blob_optional(
        &mut self,
        value: &LuaValue,
        r#type: &TypeHandle,
    ) -> Result<LuaValue, LuaValidationErrorKind> {
        let inner_type = r#type.inner_type()
            .expect("invalid optional type");

        match value {
            LuaValue::Nil => Ok(LuaValue::Nil),
            _ => self.process(value, &inner_type),
        }
    }

    #[inline]
    fn process_blob_variant(
        &mut self,
        value: &LuaValue,
        r#type: &TypeHandle,
    ) -> Result<LuaValue, LuaValidationErrorKind> {
        match value {
            LuaValue::Table(table) => {
                self.path.push("type");

                let type_name = table.get::<String>("type")?;
                let variant_type = r#type.type_registry()
                    .get_by_name(LookupKey::Qualified(&type_name))
                    .ok_or_else(|| LuaValidationErrorKind::InvalidType(type_name))?;
                let variant_type = TypeHandle::new(
                    r#type.type_registry().clone(),
                    variant_type.index,
                );

                let base_variant_type = r#type.inner_type()
                    .expect("invalid variant type");

                if !variant_type.is_sub_type_of(&base_variant_type) {
                    return Err(LuaValidationErrorKind::IncompatibleType {
                        expected: name_of(&base_variant_type),
                        found: name_of(&variant_type),
                    });
                }

                self.path.pop();
                self.path.push("value");

                let value = table.get::<LuaValue>("value")?;
                let result = self.process_struct(&value, &variant_type)?;

                self.path.pop();

                let variant_value = VariantValue::new_with_data(
                    r#type.clone(),
                    variant_type,
                    result,
                )?;
                let ud = self.lua.create_userdata(variant_value)?;

                Ok(LuaValue::UserData(ud))
            },
            LuaValue::UserData(ud) => {
                self.try_clone_variant_ud_lua(ud, r#type)
            },
            _ => Err(LuaValidationErrorKind::IncompatibleType {
                expected: name_of(r#type),
                found: value.to_string()?,
            }),
        }
    }

    #[inline]
    fn process_object_reference(
        &mut self,
        value: &LuaValue,
    ) -> Result<LuaValue, LuaValidationErrorKind> {
        match value {
            LuaValue::String(s) => {
                let guid = s.to_string_lossy();
                let _ = ContentHash::parse(&guid)
                    .ok_or_else(|| LuaValidationErrorKind::InvalidGuid(guid))?;

                Ok(LuaValue::String(s.clone()))
            },
            LuaValue::UserData(ud) => {
                let type_id = match ud.type_id() {
                    Some(type_id) => type_id,
                    None => panic!("UserData has no type_id"),
                };

                if type_id == TypeId::of::<Resource>() {
                    let resource = ud.borrow::<Resource>()?;
                    let guid = resource.info().resource_id.to_string();
                    let lua_str = self.lua.create_string(&guid)?;

                    return Ok(LuaValue::String(lua_str));
                }

                Err(LuaValidationErrorKind::IncompatibleType {
                    expected: "object reference".to_string(),
                    found: format!("userdata `{type_id:?}`"),
                })
            },
            _ => Err(LuaValidationErrorKind::IncompatibleType {
                expected: "object reference".to_string(),
                found: value.to_string()?,
            }),
        }
    }

    #[inline]
    fn process_guid(
        &mut self,
        value: &LuaValue,
    ) -> Result<LuaValue, LuaValidationErrorKind> {
        let guid = value.cast::<Guid>()?.to_string();
        let lua_str = self.lua.create_string(&guid)?;

        Ok(LuaValue::String(lua_str))
    }

    fn process_array_like(
        &mut self,
        lua_value: &LuaValue,
        r#type: &TypeHandle,
    ) -> Result<LuaValue, LuaValidationErrorKind> {
        match &lua_value {
            LuaValue::Table(table) => {
                let mut result = Vec::with_capacity(table.len()? as usize);
                let inner_type = r#type.inner_type()
                    .expect("invalid bitmask type");

                if r#type.primitive_type == PrimitiveType::StaticArray &&
                    table.len()? != r#type.field_count as i64
                {
                    return Err(LuaValidationErrorKind::IncompatibleType {
                        expected: format!("array of length {}", r#type.field_count),
                        found: format!("array of length {}", table.len()?),
                    });
                }

                // TODO: truncate duplicates for set-like arrays
                for (i, entry) in table.sequence_values::<LuaValue>().enumerate() {

                    self.path.push_index(i);

                    let value = entry?;
                    let value = self.process(&value, &inner_type)?;

                    result.push(value);

                    self.path.pop();
                }

                let array_value = ArrayValue::new_with_data(
                    r#type.clone(),
                    result
                )?;
                let ud = self.lua.create_userdata(array_value)?;

                Ok(LuaValue::UserData(ud))
            },
            LuaValue::UserData(ud) => self.try_clone_array_ud_lua(ud, r#type),
            _ => Err(LuaValidationErrorKind::IncompatibleType {
                expected: name_of(r#type),
                found: lua_value.to_string()?,
            }),
        }
    }

    fn try_clone_struct_ud_lua(
        &mut self,
        ud: &AnyUserData,
        r#type: &TypeHandle,
    ) -> Result<LuaValue, LuaValidationErrorKind> {
        let type_id = match ud.type_id() {
            Some(type_id) => type_id,
            None => panic!("UserData has no type_id"),
        };

        match type_id {
            id if id == TypeId::of::<StructValue>() => {
                let struct_value = ud.borrow::<StructValue>()?;

                if !struct_value.is(r#type) {
                    return Err(LuaValidationErrorKind::IncompatibleType {
                        expected: name_of(r#type),
                        found: name_of(struct_value.r#type()),
                    });
                }

                let new_struct_value = struct_value.try_clone_lua(&self.lua)?;
                let ud = self.lua.create_userdata(new_struct_value)?;

                Ok(LuaValue::UserData(ud))
            },
            id if id == TypeId::of::<MappedStructValue>() => {
                let struct_value = ud.borrow::<MappedStructValue>()?;

                if !struct_value.is(r#type) {
                    return Err(LuaValidationErrorKind::IncompatibleType {
                        expected: name_of(r#type),
                        found: name_of(struct_value.r#type()),
                    });
                }

                let new_struct_value = struct_value.try_clone_lua(&self.lua)?;
                let ud = self.lua.create_userdata(new_struct_value)?;

                Ok(LuaValue::UserData(ud))
            },
            id if id == TypeId::of::<Content>() => {
                let content = ud.borrow::<Content>()?;

                let guid = content.guid();
                let table = self.lua.create_table()?;

                table.set("size", guid.size())?;
                table.set("hash0", guid.hash0())?;
                table.set("hash1", guid.hash1())?;
                table.set("hash2", guid.hash2())?;

                self.process_struct_fields(&table, r#type)
            },
            _ => Err(LuaValidationErrorKind::IncompatibleType {
                expected: name_of(r#type),
                found: format!("userdata `{type_id:?}`"),
            })
        }
    }

    fn try_clone_array_ud_lua(
        &self,
        ud: &AnyUserData,
        r#type: &TypeHandle,
    ) -> Result<LuaValue, LuaValidationErrorKind> {
        let type_id = match ud.type_id() {
            Some(type_id) => type_id,
            None => panic!("UserData has no type_id"),
        };

        match type_id {
            id if id == TypeId::of::<ArrayValue>() => {
                let array_value = ud.borrow::<ArrayValue>()?;

                if !array_value.is(r#type) {
                    return Err(LuaValidationErrorKind::IncompatibleType {
                        expected: name_of(r#type),
                        found: name_of(array_value.r#type()),
                    });
                }

                let new_array_value = array_value.try_clone_lua(&self.lua)?;
                let ud = self.lua.create_userdata(new_array_value)?;

                Ok(LuaValue::UserData(ud))
            },
            id if id == TypeId::of::<MappedArrayValue>() => {
                let array_value = ud.borrow::<MappedArrayValue>()?;

                if !array_value.is(r#type) {
                    return Err(LuaValidationErrorKind::IncompatibleType {
                        expected: name_of(r#type),
                        found: name_of(array_value.r#type()),
                    });
                }

                let new_array_value = array_value.try_clone_lua(&self.lua)?;
                let ud = self.lua.create_userdata(new_array_value)?;

                Ok(LuaValue::UserData(ud))
            },
            _ => Err(LuaValidationErrorKind::IncompatibleType {
                expected: name_of(r#type),
                found: format!("userdata `{type_id:?}`"),
            })
        }
    }

    fn try_clone_variant_ud_lua(
        &self,
        ud: &AnyUserData,
        r#type: &TypeHandle,
    ) -> Result<LuaValue, LuaValidationErrorKind> {
        let type_id = match ud.type_id() {
            Some(type_id) => type_id,
            None => panic!("UserData has no type_id"),
        };

        match type_id {
            id if id == TypeId::of::<VariantValue>() => {
                let variant_value = ud.borrow::<VariantValue>()?;

                if !variant_value.is(r#type) {
                    return Err(LuaValidationErrorKind::IncompatibleType {
                        expected: name_of(r#type),
                        found: name_of(variant_value.r#type()),
                    });
                }

                let new_variant_value = variant_value.try_clone_lua(&self.lua)?;
                let ud = self.lua.create_userdata(new_variant_value)?;

                Ok(LuaValue::UserData(ud))
            },
            id if id == TypeId::of::<MappedVariantValue>() => {
                let variant_value = ud.borrow::<MappedVariantValue>()?;

                if !variant_value.is(r#type) {
                    return Err(LuaValidationErrorKind::IncompatibleType {
                        expected: name_of(r#type),
                        found: name_of(variant_value.r#type()),
                    });
                }

                let new_variant_value = variant_value.try_clone_lua(&self.lua)?;
                let ud = self.lua.create_userdata(new_variant_value)?;

                Ok(LuaValue::UserData(ud))
            },
            _ => Err(LuaValidationErrorKind::IncompatibleType {
                expected: name_of(r#type),
                found: format!("userdata `{type_id:?}`"),
            })
        }
    }

}

pub fn try_clone_lua_value(
    lua_value: &LuaValue,
    lua: &mlua::Lua,
) -> mlua::Result<LuaValue> {
    match lua_value {
        LuaValue::UserData(ud) => {
            let type_id = match ud.type_id() {
                Some(type_id) => type_id,
                None => panic!("UserData has no type_id"),
            };

            match type_id {
                id if id == TypeId::of::<StructValue>() => {
                    let struct_value = ud.borrow::<StructValue>()?;
                    let cloned_struct = struct_value.try_clone_lua(lua)?;

                    lua.create_userdata(cloned_struct)
                        .map(LuaValue::UserData)
                },
                id if id == TypeId::of::<MappedStructValue>() => {
                    let struct_value = ud.borrow::<MappedStructValue>()?;
                    let cloned_struct = struct_value.try_clone_lua(lua)?;

                    lua.create_userdata(cloned_struct)
                        .map(LuaValue::UserData)
                },
                id if id == TypeId::of::<ArrayValue>() => {
                    let array_value = ud.borrow::<ArrayValue>()?;
                    let cloned_array = array_value.try_clone_lua(lua)?;

                    lua.create_userdata(cloned_array)
                        .map(LuaValue::UserData)
                },
                id if id == TypeId::of::<MappedArrayValue>() => {
                    let array_value = ud.borrow::<MappedArrayValue>()?;
                    let cloned_array = array_value.try_clone_lua(lua)?;

                    lua.create_userdata(cloned_array)
                        .map(LuaValue::UserData)
                },
                id if id == TypeId::of::<VariantValue>() => {
                    let variant_value = ud.borrow::<VariantValue>()?;
                    let cloned_variant = variant_value.try_clone_lua(lua)?;

                    lua.create_userdata(cloned_variant)
                        .map(LuaValue::UserData)
                },
                id if id == TypeId::of::<MappedVariantValue>() => {
                    let variant_value = ud.borrow::<MappedVariantValue>()?;
                    let cloned_variant = variant_value.try_clone_lua(lua)?;

                    lua.create_userdata(cloned_variant)
                        .map(LuaValue::UserData)
                },
                _ => panic!("unexpected user data type in clone_lua_value"),
            }
        },
        _ => Ok(lua_value.clone()),
    }
}
