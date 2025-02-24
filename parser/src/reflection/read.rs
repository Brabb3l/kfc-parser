use std::io::{Cursor, Read, Seek, SeekFrom};

use serde_json::{Map, Value as JsonValue};
use shared::io::{ReadExt, ReadSeekExt};

use crate::guid::BlobGuid;

use super::types::*;
use super::{ReadError, TypeCollection};

impl TypeCollection {
    pub fn deserialize_by_hash(
        &self,
        type_hash: u32,
        data: &[u8]
    ) -> Result<JsonValue, ReadError> {
        let type_entry = self.get_type_by_qualified_hash(type_hash)
            .ok_or(ReadError::UnknownType(type_hash))?;

        Self::deserialize(self, type_entry, data)
    }

    pub fn deserialize(
        &self,
        type_entry: &TypeInfo,
        data: &[u8]
    ) -> Result<JsonValue, ReadError> {
        let mut reader = Cursor::new(data);
        Self::read_type(self, type_entry, &mut reader, 0)
    }

    fn read_type<R: Read + Seek>(
        &self,
        type_entry: &TypeInfo,
        reader: &mut R,
        base_offset: u64
    ) -> Result<JsonValue, ReadError> {
        reader.seek(SeekFrom::Start(base_offset))?;

        let value = match type_entry.primitive_type {
            PrimitiveType::None => JsonValue::Null,
            PrimitiveType::Bool => JsonValue::Bool(reader.read_u8()? != 0),
            PrimitiveType::UInt8 => JsonValue::Number(reader.read_u8()?.into()),
            PrimitiveType::SInt8 => JsonValue::Number(reader.read_i8()?.into()),
            PrimitiveType::UInt16 => JsonValue::Number(reader.read_u16()?.into()),
            PrimitiveType::SInt16 => JsonValue::Number(reader.read_i16()?.into()),
            PrimitiveType::UInt32 => JsonValue::Number(reader.read_u32()?.into()),
            PrimitiveType::SInt32 => JsonValue::Number(reader.read_i32()?.into()),
            PrimitiveType::UInt64 => JsonValue::Number(reader.read_u64()?.into()),
            PrimitiveType::SInt64 => JsonValue::Number(reader.read_i64()?.into()),
            PrimitiveType::Float32 => {
                let value = reader.read_f32()?;

                if let Some(value) = serde_json::Number::from_f64(value as f64) {
                    JsonValue::Number(value)
                } else {
                    JsonValue::String(value.to_string())
                }
            },
            PrimitiveType::Float64 => {
                let value = reader.read_f64()?;

                if let Some(value) = serde_json::Number::from_f64(value) {
                    JsonValue::Number(value)
                } else {
                    JsonValue::String(value.to_string())
                }
            },
            PrimitiveType::Enum => {
                let enum_value_type = self.get_inner_type(type_entry);
                let enum_value_raw = match enum_value_type.primitive_type {
                    PrimitiveType::SInt8 => reader.read_i8()? as u64,
                    PrimitiveType::SInt16 => reader.read_i16()? as u64,
                    PrimitiveType::SInt32 => reader.read_i32()? as u64,
                    PrimitiveType::SInt64 => reader.read_i64()? as u64,
                    PrimitiveType::UInt8 => reader.read_u8()? as u64,
                    PrimitiveType::UInt16 => reader.read_u16()? as u64,
                    PrimitiveType::UInt32 => reader.read_u32()? as u64,
                    PrimitiveType::UInt64 => reader.read_u64()?,
                    _ => panic!("Unsupported enum value type: {:?}", enum_value_type)
                };
                
                let enum_name = type_entry.enum_fields
                    .iter()
                    .find(|f| f.value == enum_value_raw)
                    .map(|s| s.name.clone())
                    .unwrap_or_else(|| enum_value_raw.to_string());

                JsonValue::String(enum_name)
            },
            PrimitiveType::Bitmask8 => {
                let bitmask_type = self.get_inner_type(type_entry);
                let value = reader.read_u8()?;
                let mut bits = Vec::with_capacity(value.count_ones() as usize);
                let mut checked_bits = 0;
                
                for enum_field in &bitmask_type.enum_fields {
                    let enum_value = enum_field.value as u8;
                    let enum_name = &enum_field.name;
                    
                    checked_bits |= 1 << enum_value;

                    if value & (1 << enum_value) != 0 {
                        bits.push(JsonValue::String(enum_name.clone()));
                    }
                }
                
                // check for unknown bits
                if value.count_ones() != bits.len() as u32 {
                    for i in 0..8 {
                        if checked_bits & (1 << i) == 0 && value & (1 << i) != 0 {
                            bits.push(JsonValue::Number(i.into()));
                        }
                    }
                }
                
                JsonValue::Array(bits)
            },
            PrimitiveType::Bitmask16 => {
                let bitmask_type = self.get_inner_type(type_entry);
                let value = reader.read_u16()?;
                let mut bits = Vec::with_capacity(value.count_ones() as usize);
                
                for enum_field in &bitmask_type.enum_fields {
                    let enum_value = enum_field.value as u16;
                    let enum_name = &enum_field.name;

                    if value & (1 << enum_value) != 0 {
                        bits.push(JsonValue::String(enum_name.clone()));
                    }
                }
                
                JsonValue::Array(bits)
            }
            PrimitiveType::Bitmask32 => {
                let bitmask_type = self.get_inner_type(type_entry);
                let value = reader.read_u32()?;
                let mut bits = Vec::with_capacity(value.count_ones() as usize);
                
                for enum_field in &bitmask_type.enum_fields {
                    let enum_value = enum_field.value as u32;
                    let enum_name = &enum_field.name;

                    if value & (1 << enum_value) != 0 {
                        bits.push(JsonValue::String(enum_name.clone()));
                    }
                }
                
                JsonValue::Array(bits)
            }
            PrimitiveType::Bitmask64 => {
                let bitmask_type = self.get_inner_type(type_entry);
                let value = reader.read_u64()?;
                let mut bits = Vec::with_capacity(value.count_ones() as usize);
                
                for enum_field in &bitmask_type.enum_fields {
                    let enum_value = enum_field.value;
                    let enum_name = &enum_field.name;

                    if value & (1 << enum_value) != 0 {
                        bits.push(JsonValue::String(enum_name.clone()));
                    }
                }
                
                JsonValue::Array(bits)
            }
            PrimitiveType::Typedef => {
                let typedef_type = self.get_inner_type(type_entry);
                
                self.read_type(
                    typedef_type,
                    reader,
                    base_offset
                )?
            },
            PrimitiveType::Struct => {
                let mut map = Map::new();
                
                if let Some(parent_type) = self.get_inner_type_opt(type_entry) {
                    let parent_value = self.read_type(
                        parent_type,
                        reader,
                        base_offset
                    )?;
                    
                    if let JsonValue::Object(parent_map) = parent_value {
                        map.extend(parent_map);
                    } else {
                        unreachable!("expected object");
                    }
                }

                for field in &type_entry.struct_fields {
                    let field_type = self.get_type_by_qualified_hash(field.r#type.hash)
                        .expect("invalid field type");
                    
                    let field_value = self.read_type(
                        field_type,
                        reader,
                        base_offset + field.data_offset
                    )?;
                    
                    map.insert(field.name.to_string(), field_value);
                }
                
                JsonValue::Object(map)
            },
            PrimitiveType::StaticArray => {
                let component_type = self.get_inner_type(type_entry);
                let mut values = Vec::new();
                
                for i in 0..type_entry.field_count {
                    let offset = (i * component_type.size) as u64 + base_offset;
                    let value = self.read_type(
                        component_type,
                        reader,
                        offset
                    )?;
                    
                    values.push(value);
                }
                
                JsonValue::Array(values)
            },
            PrimitiveType::DsArray => unreachable!(),
            PrimitiveType::DsString => unreachable!(),
            PrimitiveType::DsOptional => unreachable!(),
            PrimitiveType::DsVariant => unreachable!(),
            PrimitiveType::BlobArray => {
                let component_type = self.get_inner_type(type_entry);
                let mut values = Vec::new();
                
                let mut offset = reader.read_u32_offset()?;
                let count = reader.read_u32()?;
                
                for _ in 0..count {
                    let value = self.read_type(
                        component_type,
                        reader,
                        offset
                    )?;
                    
                    values.push(value);
                    offset += component_type.size as u64;
                }
                
                JsonValue::Array(values)
            },
            PrimitiveType::BlobString => {
                let relative_offset = reader.read_u32()?;
                
                if relative_offset != 0 {
                    let offset = base_offset + relative_offset as u64;
                    let length = reader.read_u32()?;

                    reader.seek(SeekFrom::Start(offset))?;

                    let mut data = vec![0; length as usize];
                    reader.read_exact(&mut data)?;

                    let value = String::from_utf8(data)?;

                    JsonValue::String(value)
                } else {
                    JsonValue::Null
                }
            },
            PrimitiveType::BlobOptional => {
                let component_type = self.get_inner_type_opt(type_entry);
                let offset = reader.read_u32()? as u64;
                
                if offset == 0 {
                    JsonValue::Null
                } else if let Some(component_type) = component_type {
                    self.read_type(
                        component_type,
                        reader,
                        base_offset + offset
                    )?
                } else {
                    JsonValue::Null
                }
            },
            PrimitiveType::BlobVariant => {
                let variant_type_hash = reader.read_u32()?;
                let relative_offset = reader.read_u32()?;
                let _size = reader.read_u32()?;

                if relative_offset == 0 {
                    JsonValue::Null
                } else {
                    let offset = base_offset + relative_offset as u64 + 4;
                    let variant_type = self.get_type_by_qualified_hash(variant_type_hash)
                        .ok_or(ReadError::InvalidTypeHash(variant_type_hash))?;
                    
                    let mut object = Map::new();
                    
                    object.insert("$type".into(), JsonValue::String(variant_type.qualified_name.clone()));
                    object.insert("$value".into(), self.read_type(
                        variant_type,
                        reader,
                        offset
                    )?);
                    
                    JsonValue::Object(object)
                }
            },
            PrimitiveType::ObjectReference => {
                let guid = BlobGuid::read(reader)?;
                
                // I don't think this is really necessary to include
                // 
                // let inner_type = self.get_inner_type_opt(type_entry);
                // 
                // let guid = if let Some(inner_type) = &inner_type {
                //     guid.as_descriptor_guid(inner_type.qualified_hash, 0)
                // } else {
                //     guid.as_descriptor_guid(0, 0)
                // };
                
                JsonValue::String(guid.to_string())
            },
            PrimitiveType::Guid => {
                let value = BlobGuid::read(reader)?;
                
                JsonValue::String(value.to_string())
            },
        };

        Ok(value)
    }
}

