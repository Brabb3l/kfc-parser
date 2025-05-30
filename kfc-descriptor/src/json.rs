use std::io::{Cursor, Read, Write, Seek, SeekFrom};
use std::str::FromStr;

use kfc::reflection::{PrimitiveType, TypeInfo};
use serde_json::{Map, Value as JsonValue};

use kfc::{guid::{BlobGuid, DescriptorGuid}, io::{ReadExt, ReadSeekExt, WriteExt}, reflection::TypeCollection};
use crate::error::{ReadError, WriteError};

pub fn serialize(
    type_collection: &TypeCollection,
    type_info: &TypeInfo,
    value: &JsonValue
) -> Result<Vec<u8>, WriteError> {
    let mut result = Vec::new();

    serialize_into(type_collection, type_info, value, &mut result)?;

    Ok(result)
}

pub fn serialize_into(
    type_collection: &TypeCollection,
    type_info: &TypeInfo,
    value: &JsonValue,
    dst: &mut Vec<u8>
) -> Result<(), WriteError> {
    dst.resize(type_info.size as usize, 0);

    let mut writer = Cursor::new(dst);
    let mut blob_offset = type_info.size as u64;

    write_type(type_collection, type_info, &mut writer, value, &mut blob_offset, 0)?;

    Ok(())
}

pub fn deserialize(
    type_collection: &TypeCollection,
    type_entry: &TypeInfo,
    data: &[u8]
) -> Result<JsonValue, ReadError> {
    let mut reader = Cursor::new(data);

    read_type(type_collection, type_entry, &mut reader, 0)
}

// TODO: Record path for better error messages

fn write_type<W: Write + Seek>(
    type_collection: &TypeCollection,
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
                match value.to_ascii_lowercase().as_str() {
                    "nan" => writer.write_f32(f32::NAN)?,
                    "+inf" | "+infinity" | "inf" | "infinity" => writer.write_f32(f32::INFINITY)?,
                    "-inf" | "-infinity" => writer.write_f32(f32::NEG_INFINITY)?,
                    _ => return Err(WriteError::IncompatibleType {
                        got: value.to_string(),
                        expected: "f32".to_string()
                    })
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
                match value.to_ascii_lowercase().as_str() {
                    "nan" => writer.write_f64(f64::NAN)?,
                    "+inf" | "+infinity" | "inf" | "infinity" => writer.write_f64(f64::INFINITY)?,
                    "-inf" | "-infinity" => writer.write_f64(f64::NEG_INFINITY)?,
                    _ => return Err(WriteError::IncompatibleType {
                        got: value.to_string(),
                        expected: "f64".to_string()
                    })
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
                let enum_value = resolve_enum_value(type_info, value)?;
                let enum_value_type = get_inner_type(type_collection, type_info);

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
            writer.write_u8(parse_bitmask_value(type_collection, type_info, value)? as u8)?;
        }
        PrimitiveType::Bitmask16 => {
            writer.write_u16(parse_bitmask_value(type_collection, type_info, value)? as u16)?;
        }
        PrimitiveType::Bitmask32 => {
            writer.write_u32(parse_bitmask_value(type_collection, type_info, value)? as u32)?;
        }
        PrimitiveType::Bitmask64 => {
            writer.write_u64(parse_bitmask_value(type_collection, type_info, value)?)?;
        }
        PrimitiveType::Typedef => {
            let inner_type = get_inner_type(type_collection, type_info);

            return write_type(
                type_collection,
                inner_type,
                writer,
                value,
                blob_offset,
                base_offset,
            );
        }
        PrimitiveType::Struct => {
            if let JsonValue::Object(fields) = value {
                if let Some(parent_type) = get_inner_type_opt(type_collection, type_info) {
                    write_type(
                        type_collection,
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

                    let field_type = type_collection.get_type(field.r#type)
                        .expect("invalid field type");

                    write_type(
                        type_collection,
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
            let component_type = get_inner_type(type_collection, type_info);

            if let JsonValue::Array(values) = value {
                for (i, value) in values.iter().enumerate() {
                    let offset = (i * component_type.size as usize) as u64 + base_offset;

                    write_type(
                        type_collection,
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
            let component_type = get_inner_type(type_collection, type_info);

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
                        write_type(
                            type_collection,
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
            let component_type = get_inner_type(type_collection, type_info);

            if !value.is_null() {
                let stream_pos = writer.stream_position()?;
                let total_size = component_type.size as u64;
                let alignment = component_type.alignment as u64;

                *blob_offset += (alignment - (*blob_offset % alignment)) % alignment;

                writer.write_u32((*blob_offset - stream_pos) as u32)?;

                let offset = *blob_offset;

                *blob_offset += total_size;
                *blob_offset += (alignment - (*blob_offset % alignment)) % alignment;

                write_type(
                    type_collection,
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

                let variant_type = type_collection.get_type_by_qualified_name(variant_name)
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

                write_type(
                    type_collection,
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
            if value.is_null() {
                BlobGuid::NONE.write(writer)?;
            } else if let Some(value) = value.as_str() {
                DescriptorGuid::from_str(value, 0, 0)
                    .ok_or_else(|| WriteError::MalformedDescriptorGuid(value.to_string()))?
                    .as_blob_guid()
                    .write(writer)?;
            } else {
                return Err(WriteError::IncompatibleType {
                    got: value.to_string(),
                    expected: "str".to_string()
                });
            }
        }
        PrimitiveType::Guid => {
            if value.is_null() {
                BlobGuid::NONE.write(writer)?;
            } else if let Some(value) = value.as_str() {
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

fn read_type<R: Read + Seek>(
    type_collection: &TypeCollection,
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
            let enum_value_type = get_inner_type(type_collection, type_entry);
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
            let bitmask_type = get_inner_type(type_collection, type_entry);
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
            let bitmask_type = get_inner_type(type_collection, type_entry);
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
            let bitmask_type = get_inner_type(type_collection, type_entry);
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
            let bitmask_type = get_inner_type(type_collection, type_entry);
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
            let typedef_type = get_inner_type(type_collection, type_entry);

            read_type(
                type_collection,
                typedef_type,
                reader,
                base_offset
            )?
        },
        PrimitiveType::Struct => {
            let mut map = Map::new();

            if let Some(parent_type) = get_inner_type_opt(type_collection, type_entry) {
                let parent_value = read_type(
                    type_collection,
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
                let field_type = type_collection.get_type(field.r#type)
                    .expect("invalid field type");

                let field_value = read_type(
                    type_collection,
                    field_type,
                    reader,
                    base_offset + field.data_offset
                )?;

                map.insert(field.name.to_string(), field_value);
            }

            JsonValue::Object(map)
        },
        PrimitiveType::StaticArray => {
            let component_type = get_inner_type(type_collection, type_entry);
            let mut values = Vec::new();

            for i in 0..type_entry.field_count {
                let offset = (i * component_type.size) as u64 + base_offset;
                let value = read_type(
                    type_collection,
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
            let component_type = get_inner_type(type_collection, type_entry);
            let mut values = Vec::new();

            let mut offset = reader.read_u32_offset()?;
            let count = reader.read_u32()?;

            for _ in 0..count {
                let value = read_type(
                    type_collection,
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
                JsonValue::String(String::new())
            }
        },
        PrimitiveType::BlobOptional => {
            let component_type = get_inner_type_opt(type_collection, type_entry);
            let offset = reader.read_u32()? as u64;

            if offset == 0 {
                JsonValue::Null
            } else if let Some(component_type) = component_type {
                read_type(
                    type_collection,
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
                let variant_type = type_collection.get_type_by_qualified_hash(variant_type_hash)
                    .ok_or(ReadError::InvalidTypeHash(variant_type_hash))?;

                let mut object = Map::new();

                object.insert("$type".into(), JsonValue::String(variant_type.qualified_name.clone()));
                object.insert("$value".into(), read_type(
                    type_collection,
                    variant_type,
                    reader,
                    offset
                )?);

                JsonValue::Object(object)
            }
        },
        PrimitiveType::ObjectReference => {
            let guid = BlobGuid::read(reader)?;

            if guid.is_none() {
                JsonValue::Null
            } else {
                JsonValue::String(guid.as_descriptor_guid(0, 0).to_string())
            }
        },
        PrimitiveType::Guid => {
            let guid = BlobGuid::read(reader)?;

            if guid.is_none() {
                JsonValue::Null
            } else {
                JsonValue::String(guid.to_string())
            }
        },
    };

    Ok(value)
}

fn resolve_enum_value(
    type_entry: &TypeInfo,
    value: &str
) -> Result<u64, WriteError> {
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

fn parse_bitmask_value(
    type_collection: &TypeCollection,
    type_entry: &TypeInfo,
    value: &JsonValue
) -> Result<u64, WriteError> {
    if let JsonValue::Array(values) = value {
        let mut bitmask = 0u64;
        let bitmask_type = get_inner_type(type_collection, type_entry);

        for value in values {
            if let Some(value) = value.as_str() {
                bitmask |= 1 << resolve_enum_value(bitmask_type, value)?;
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

#[inline]
fn get_inner_type<'a>(type_collection: &'a TypeCollection, type_entry: &'a TypeInfo) -> &'a TypeInfo {
    type_entry.inner_type
        .and_then(|t| type_collection.get_type(t))
        .map(|t| resolve_typedef(type_collection, t))
        .expect("invalid inner type")
}

#[inline]
fn get_inner_type_opt<'a>(type_collection: &'a TypeCollection, type_entry: &'a TypeInfo) -> Option<&'a TypeInfo> {
    type_entry.inner_type
        .and_then(|t| type_collection.get_type(t))
        .map(|t| resolve_typedef(type_collection, t))
}

#[inline]
fn resolve_typedef<'a>(type_collection: &'a TypeCollection, type_entry: &'a TypeInfo) -> &'a TypeInfo {
    match &type_entry.primitive_type {
        PrimitiveType::Typedef => get_inner_type(type_collection, type_entry),
        _ => type_entry
    }
}
