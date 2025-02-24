use std::io::{Cursor, Seek, SeekFrom, Write};
use std::str::FromStr;

use serde_json::Value as JsonValue;
use shared::io::WriteExt;

use crate::guid::BlobGuid;

use super::types::*;
use super::{TypeCollection, WriteError};

impl TypeCollection {
    pub fn serialize_into_by_hash(
        &self,
        type_hash: u32,
        value: &JsonValue,
        dst: &mut Vec<u8>
    ) -> Result<(), WriteError> {
        let type_info = self.get_type_by_qualified_hash(type_hash)
            .ok_or(WriteError::UnknownType(type_hash))?;

        self.serialize_into(type_info, value, dst)
    }
    
    pub fn serialize_by_hash(
        &self,
        type_info: u32,
        value: &JsonValue
    ) -> Result<Vec<u8>, WriteError> {
        let type_entry = self.get_type_by_qualified_hash(type_info)
            .ok_or(WriteError::UnknownType(type_info))?;

        self.serialize(type_entry, value)
    }
    
    pub fn serialize_into(
        &self,
        type_info: &TypeInfo,
        value: &JsonValue,
        dst: &mut Vec<u8>
    ) -> Result<(), WriteError> {
        let mut writer = Cursor::new(dst);
        let mut blob_offset = type_info.size as u64;
        
        self.write_type(type_info, &mut writer, value, &mut blob_offset, 0)?;

        Ok(())
    }

    pub fn serialize(
        &self,
        type_info: &TypeInfo,
        value: &JsonValue
    ) -> Result<Vec<u8>, WriteError> {
        let mut writer = Cursor::new(Vec::new());
        let mut blob_offset = type_info.size as u64;
        
        self.write_type(type_info, &mut writer, value, &mut blob_offset, 0)?;

        Ok(writer.into_inner())
    }

    fn write_type<W: Write + Seek>(
        &self,
        type_info: &TypeInfo,
        writer: &mut W,
        value: &JsonValue,
        blob_offset: &mut u64,
        base_offset: u64
    ) -> Result<(), WriteError> {
        writer.seek(SeekFrom::Start(base_offset))?;

        match &type_info.primitive_type {
            PrimitiveType::None => {}
            PrimitiveType::Bool => {
                return if let Some(value) = value.as_bool() {
                    writer.write_u8(value as u8)?;
                    Ok(())
                } else {
                    Err(WriteError::IncompatibleType {
                        got: value.to_string(),
                        expected: "bool".to_string()
                    })
                }
            }
            PrimitiveType::UInt8 => {
                if let Some(value) = value.as_u64() {
                    if value <= u8::MAX as u64 {
                        writer.write_u8(value as u8)?;
                        return Ok(());
                    }
                }
                
                return Err(WriteError::IncompatibleType {
                    got: value.to_string(),
                    expected: "u8".to_string()
                });
            }
            PrimitiveType::SInt8 => {
                if let Some(value) = value.as_i64() {
                    if value >= i8::MIN as i64 && value <= i8::MAX as i64 {
                        writer.write_i8(value as i8)?;
                        return Ok(());
                    }
                }
                
                return Err(WriteError::IncompatibleType {
                    got: value.to_string(),
                    expected: "i8".to_string()
                });
            }
            PrimitiveType::UInt16 => {
                if let Some(value) = value.as_u64() {
                    if value <= u16::MAX as u64 {
                        writer.write_u16(value as u16)?;
                        return Ok(());
                    }
                }
                
                return Err(WriteError::IncompatibleType {
                    got: value.to_string(),
                    expected: "u16".to_string()
                });
            }
            PrimitiveType::SInt16 => {
                if let Some(value) = value.as_i64() {
                    if value >= i16::MIN as i64 && value <= i16::MAX as i64 {
                        writer.write_i16(value as i16)?;
                        return Ok(());
                    }
                }
                
                return Err(WriteError::IncompatibleType {
                    got: value.to_string(),
                    expected: "i16".to_string()
                });
            }
            PrimitiveType::UInt32 => {
                if let Some(value) = value.as_u64() {
                    if value <= u32::MAX as u64 {
                        writer.write_u32(value as u32)?;
                        return Ok(());
                    }
                }
                
                return Err(WriteError::IncompatibleType {
                    got: value.to_string(),
                    expected: "u32".to_string()
                });
            }
            PrimitiveType::SInt32 => {
                if let Some(value) = value.as_i64() {
                    if value >= i32::MIN as i64 && value <= i32::MAX as i64 {
                        writer.write_i32(value as i32)?;
                        return Ok(());
                    }
                }
                
                return Err(WriteError::IncompatibleType {
                    got: value.to_string(),
                    expected: "i32".to_string()
                });
            }
            PrimitiveType::UInt64 => {
                if let Some(value) = value.as_u64() {
                    writer.write_u64(value)?;
                } else {
                    return Err(WriteError::IncompatibleType {
                        got: value.to_string(),
                        expected: "u64".to_string()
                    });
                }
            }
            PrimitiveType::SInt64 => {
                if let Some(value) = value.as_i64() {
                    writer.write_i64(value)?;
                } else {
                    return Err(WriteError::IncompatibleType {
                        got: value.to_string(),
                        expected: "i64".to_string()
                    });
                }
            }
            PrimitiveType::Float32 => {
                if let Some(value) = value.as_f64() {
                    writer.write_f32(value as f32)?;
                } else if let Some(value) = value.as_str() {
                    if value == "NaN" {
                        writer.write_f32(f32::NAN)?;
                    } else if value == "+inf" {
                        writer.write_f32(f32::INFINITY)?;
                    } else if value == "-inf" {
                        writer.write_f32(f32::NEG_INFINITY)?;
                    } else {
                        return Err(WriteError::IncompatibleType {
                            got: value.to_string(),
                            expected: "f32".to_string()
                        });
                    }
                } else {
                    return Err(WriteError::IncompatibleType {
                        got: value.to_string(),
                        expected: "f32".to_string()
                    });
                }
            }
            PrimitiveType::Float64 => {
                if let Some(value) = value.as_f64() {
                    writer.write_f64(value)?;
                } else if let Some(value) = value.as_str() {
                    if value == "NaN" {
                        writer.write_f64(f64::NAN)?;
                    } else if value == "+inf" {
                        writer.write_f64(f64::INFINITY)?;
                    } else if value == "-inf" {
                        writer.write_f64(f64::NEG_INFINITY)?;
                    } else {
                        return Err(WriteError::IncompatibleType {
                            got: value.to_string(),
                            expected: "f64".to_string()
                        });
                    }
                } else {
                    return Err(WriteError::IncompatibleType {
                        got: value.to_string(),
                        expected: "f64".to_string()
                    });
                }
            }
            PrimitiveType::Enum => {
                if let Some(value) = value.as_str() {
                    let enum_value = self.resolve_enum_value(type_info, value)?;
                    let enum_value_type = self.get_inner_type(type_info);

                    match enum_value_type.primitive_type {
                        PrimitiveType::SInt8 => writer.write_i8(enum_value as i8)?,
                        PrimitiveType::SInt16 => writer.write_i16(enum_value as i16)?,
                        PrimitiveType::SInt32 => writer.write_i32(enum_value as i32)?,
                        PrimitiveType::SInt64 => writer.write_i64(enum_value as i64)?,
                        PrimitiveType::UInt8 => writer.write_u8(enum_value as u8)?,
                        PrimitiveType::UInt16 => writer.write_u16(enum_value as u16)?,
                        PrimitiveType::UInt32 => writer.write_u32(enum_value as u32)?,
                        PrimitiveType::UInt64 => writer.write_u64(enum_value)?,
                        _ => panic!("Unsupported enum value type: {:?}", enum_value_type)
                    }
                } else {
                    return Err(WriteError::IncompatibleType {
                        got: value.to_string(),
                        expected: "str".to_string()
                    });
                }
            }
            PrimitiveType::Bitmask8 => {
                writer.write_u8(self.parse_bitmask_value(type_info, value)? as u8)?;
            }
            PrimitiveType::Bitmask16 => {
                writer.write_u16(self.parse_bitmask_value(type_info, value)? as u16)?;
            }
            PrimitiveType::Bitmask32 => {
                writer.write_u32(self.parse_bitmask_value(type_info, value)? as u32)?;
            }
            PrimitiveType::Bitmask64 => {
                writer.write_u64(self.parse_bitmask_value(type_info, value)?)?;
            }
            PrimitiveType::Typedef => {
                let inner_type = self.get_inner_type(type_info);

                return Self::write_type(
                    self,
                    inner_type,
                    writer,
                    value,
                    blob_offset,
                    base_offset,
                );
            }
            PrimitiveType::Struct => {
                if let JsonValue::Object(fields) = value {
                    if let Some(parent_type) = self.get_inner_type_opt(type_info) {
                        Self::write_type(
                            self,
                            parent_type,
                            writer,
                            value,
                            blob_offset,
                            base_offset,
                        )?;
                    }

                    for field in &type_info.struct_fields {
                        let (_, field_value) = fields.iter()
                            .find(|(name, _)| name == &&field.name)
                            .ok_or_else(|| WriteError::MissingField(field.name.clone()))?;
                        
                        let field_type = self.get_type_by_qualified_hash(field.r#type.hash)
                            .expect("invalid field type");

                        Self::write_type(
                            self,
                            field_type,
                            writer,
                            field_value,
                            blob_offset,
                            base_offset + field.data_offset,
                        )?;
                    }
                } else {
                    return Err(WriteError::IncompatibleType {
                        got: value.to_string(),
                        expected: "object".to_string()
                    });
                }
            }
            PrimitiveType::StaticArray => {
                let component_type = self.get_inner_type(type_info);

                if let JsonValue::Array(values) = value {
                    for (i, value) in values.iter().enumerate() {
                        let offset = (i * component_type.size as usize) as u64 + base_offset;

                        Self::write_type(
                            self,
                            component_type,
                            writer,
                            value,
                            blob_offset,
                            offset,
                        )?;
                    }
                } else {
                    return Err(WriteError::IncompatibleType {
                        got: value.to_string(),
                        expected: "array".to_string()
                    });
                }
            }
            PrimitiveType::DsArray => unreachable!(),
            PrimitiveType::DsString => unreachable!(),
            PrimitiveType::DsOptional => unreachable!(),
            PrimitiveType::DsVariant => unreachable!(),
            PrimitiveType::BlobArray => {
                let component_type = self.get_inner_type(type_info);

                if let JsonValue::Array(values) = value {
                    let stream_pos = writer.stream_position()?;
                    let total_size = values.len() as u64 * component_type.size as u64;
                    let alignment = component_type.alignment as u64;

                    if values.is_empty() {
                        writer.write_u32(0)?;
                        writer.write_u32(0)?;
                    } else {
                        *blob_offset += (alignment - (*blob_offset % alignment)) % alignment;

                        writer.write_u32((*blob_offset - stream_pos) as u32)?;
                        writer.write_u32(values.len() as u32)?;

                        let mut offset = *blob_offset;

                        *blob_offset += total_size;
                        *blob_offset += (alignment - (*blob_offset % alignment)) % alignment;

                        for value in values {
                            Self::write_type(
                                self,
                                component_type,
                                writer,
                                value,
                                blob_offset,
                                offset,
                            )?;
                            offset += component_type.size as u64;
                        }
                    }
                } else {
                    return Err(WriteError::IncompatibleType {
                        got: value.to_string(),
                        expected: "array".to_string()
                    });
                }
            }
            PrimitiveType::BlobString => {
                if value.is_null() {
                    writer.write_u32(0)?;
                    writer.write_u32(0)?;
                } else if let Some(value) = value.as_str() {
                    let stream_pos = writer.stream_position()?;
                    let total_size = value.len() as u64;

                    writer.write_u32((*blob_offset - stream_pos) as u32)?;
                    writer.write_u32(value.len() as u32)?;

                    let offset = *blob_offset;

                    *blob_offset += total_size;

                    writer.seek(SeekFrom::Start(offset))?;
                    writer.write_all(value.as_bytes())?;

                    return Ok(());
                } else {
                    return Err(WriteError::IncompatibleType {
                        got: value.to_string(),
                        expected: "str".to_string()
                    });
                }
            }
            PrimitiveType::BlobOptional => {
                let component_type = self.get_inner_type(type_info);

                if !value.is_null() {
                    let stream_pos = writer.stream_position()?;
                    let total_size = component_type.size as u64;
                    let alignment = component_type.alignment as u64;

                    *blob_offset += (alignment - (*blob_offset % alignment)) % alignment;

                    writer.write_u32((*blob_offset - stream_pos) as u32)?;

                    let offset = *blob_offset;

                    *blob_offset += total_size;
                    *blob_offset += (alignment - (*blob_offset % alignment)) % alignment;

                    Self::write_type(
                        self,
                        component_type,
                        writer,
                        value,
                        blob_offset,
                        offset,
                    )?;
                } else {
                    writer.write_u32(0)?;
                }
            }
            PrimitiveType::BlobVariant => {
                if value.is_null() {
                    writer.write_u32(0)?;
                    writer.write_u32(0)?;
                    writer.write_u32(0)?;
                } else if let JsonValue::Object(fields) = value {
                    // TODO: Validate that the given type is a valid sub-type of type_entry.inner_type
                    let variant_name = fields.get("$type")
                        .and_then(|v| v.as_str())
                        .ok_or(WriteError::MissingFieldType)?;

                    let value = fields.get("$value")
                        .ok_or(WriteError::MissingFieldValue)?;

                    let variant_type = self.get_type_by_qualified_name(variant_name)
                        .ok_or_else(|| WriteError::InvalidType(variant_name.to_string()))?;

                    writer.write_u32(variant_type.qualified_hash)?;

                    let stream_pos = writer.stream_position()?;
                    let total_size = variant_type.size as u64;
                    let alignment = variant_type.alignment as u64;

                    *blob_offset += (alignment - (*blob_offset % alignment)) % alignment;

                    writer.write_u32((*blob_offset - stream_pos) as u32)?;
                    writer.write_u32(variant_type.size)?;

                    let offset = *blob_offset;

                    *blob_offset += total_size;
                    *blob_offset += (alignment - (*blob_offset % alignment)) % alignment;

                    Self::write_type(
                        self,
                        variant_type,
                        writer,
                        value,
                        blob_offset,
                        offset,
                    )?;
                } else {
                    return Err(WriteError::IncompatibleType {
                        got: value.to_string(),
                        expected: "object".to_string()
                    });
                }
            }
            PrimitiveType::ObjectReference => {
                if let Some(value) = value.as_str() {
                    BlobGuid::from_str(value)
                        .map_err(WriteError::MalformedBlobGuid)?
                        .write(writer)?;
                } else {
                    return Err(WriteError::IncompatibleType {
                        got: value.to_string(),
                        expected: "str".to_string()
                    });
                }
            }
            PrimitiveType::Guid => {
                if let Some(value) = value.as_str() {
                    BlobGuid::from_str(value)
                        .map_err(WriteError::MalformedBlobGuid)?
                        .write(writer)?;
                } else {
                    return Err(WriteError::IncompatibleType {
                        got: value.to_string(),
                        expected: "str".to_string()
                    });
                }
            }
        }

        // fill remaining space with zeroes
        let end_offset = base_offset + type_info.size as u64;

        if writer.stream_position()? < end_offset {
            writer.seek(SeekFrom::Start(end_offset - 1))?;
            writer.write_u8(0)?;
        }

        Ok(())
    }

    fn parse_bitmask_value(&self, type_entry: &TypeInfo, value: &JsonValue) -> Result<u64, WriteError> {
        if let JsonValue::Array(values) = value {
            let mut bitmask = 0u64;
            let bitmask_type = self.get_inner_type(type_entry);

            for value in values {
                if let Some(value) = value.as_str() {
                    bitmask |= 1 << self.resolve_enum_value(bitmask_type, value)?;
                } else if let Some(value) = value.as_u64() {
                    bitmask |= 1 << value;
                } else {
                    return Err(WriteError::IncompatibleType {
                        got: value.to_string(),
                        expected: "str".to_string()
                    });
                }
            }

            Ok(bitmask)
        } else {
            Err(WriteError::IncompatibleType {
                got: value.to_string(),
                expected: "array".to_string()
            })
        }
    }

    fn resolve_enum_value(&self, type_entry: &TypeInfo, value: &str) -> Result<u64, WriteError> {
        let enum_value = type_entry.enum_fields.iter()
            .find(|field| field.name.as_str() == value);

        if let Some(field) = enum_value {
            Ok(field.value)
        } else {
            Err(WriteError::InvalidEnumValue {
                got: value.to_string(),
                expected: type_entry.enum_fields
                    .iter()
                    .map(|field| field.name.clone())
                    .collect()
            })
        }
    }

}

