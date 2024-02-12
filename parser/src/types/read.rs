use std::collections::HashMap;
use std::io::{Cursor, Read, Seek, SeekFrom};

use crate::error::ParseError;
use shared::io::Reader;
use crate::crpf::{Crpf, CtcbTypeEntry};
use crate::types::{CrpfNode, CrpfGuid, Guid, PrimitiveType, CrpfStruct, CrpfEnum, CrpfTypedef, Bitmask8, Bitmask16, Bitmask32, Bitmask64, BlobOptional, BlobVariant, ObjectReference};

impl Guid {
    pub fn new(data: [u8; 16]) -> Self {
        Self {
            data
        }
    }

    pub fn read<T: Read>(reader: &mut T) -> Result<Self, ParseError> {
        let mut data = [0; 16];
        reader.read_exact(&mut data)?;

        Ok(Self {
            data
        })
    }
}

impl CrpfGuid {
    pub fn read<T: Read>(reader: &mut Reader<T>) -> Result<Self, ParseError> {
        let mut data = [0; 16];
        reader.read_exact(&mut data)?;

        let mut data2 = [0; 4];
        reader.read_exact(&mut data2)?;

        let data3 = reader.read_u32()?;
        let data4 = reader.read_u64()?;

        Ok(Self {
            data,
            data2,
            data3,
            data4
        })
    }
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
}

impl Crpf {
    pub fn parse(&self) -> anyhow::Result<CrpfNode> {
        let type_entry = &self.ctcb.type_entries[0];
        let mut reader = Reader::new(Cursor::new(&self.kbf.data));

        Self::convert_type(self, type_entry, &mut reader, 0)
    }

    fn convert_type<T: Read + Seek>(
        crpf: &Crpf,
        type_entry: &CtcbTypeEntry,
        reader: &mut Reader<T>,
        base_offset: u64
    ) -> anyhow::Result<CrpfNode> {
        reader.seek(SeekFrom::Start(base_offset))?;

        let value = match type_entry.primitive_type {
            PrimitiveType::None => CrpfNode::None,
            PrimitiveType::Bool => CrpfNode::Bool(reader.read_u8()? == 1),
            PrimitiveType::UInt8 => CrpfNode::UInt8(reader.read_u8()?),
            PrimitiveType::SInt8 => CrpfNode::SInt8(reader.read_i8()?),
            PrimitiveType::UInt16 => CrpfNode::UInt16(reader.read_u16()?),
            PrimitiveType::SInt16 => CrpfNode::SInt16(reader.read_i16()?),
            PrimitiveType::UInt32 => CrpfNode::UInt32(reader.read_u32()?),
            PrimitiveType::SInt32 => CrpfNode::SInt32(reader.read_i32()?),
            PrimitiveType::UInt64 => CrpfNode::UInt64(reader.read_u64()?),
            PrimitiveType::SInt64 => CrpfNode::SInt64(reader.read_i64()?),
            PrimitiveType::Float32 => CrpfNode::Float32(reader.read_f32()?),
            PrimitiveType::Float64 => CrpfNode::Float64(reader.read_f64()?),
            PrimitiveType::Enum => {
                let enum_type = crpf.get_type(type_entry.ref_type_index)?;
                let type_name = crpf.get_name(type_entry.name1_offset)?.clone();
                let enum_value = Self::convert_type(crpf, enum_type, reader, base_offset)?;

                let enum_name = enum_value.as_u64()
                    .and_then(|v| type_entry.enum_values.get(&v).cloned())
                    .unwrap_or_else(|| "".to_string());

                CrpfNode::Enum(CrpfEnum {
                    type_name,
                    value: Box::new(enum_value),
                    value_name: enum_name,
                })
            },
            PrimitiveType::Bitmask8 => CrpfNode::Bitmask8(Bitmask8(reader.read_u8()?)),
            PrimitiveType::Bitmask16 => CrpfNode::Bitmask16(Bitmask16(reader.read_u16()?)),
            PrimitiveType::Bitmask32 => CrpfNode::Bitmask32(Bitmask32(reader.read_u32()?)),
            PrimitiveType::Bitmask64 => CrpfNode::Bitmask64(Bitmask64(reader.read_u64()?)),
            PrimitiveType::Typedef => {
                let typedef_type = crpf.get_type(type_entry.ref_type_index)?;
                let type_name = crpf.get_name(type_entry.name1_offset)?.clone();
                let typedef_value = Self::convert_type(crpf, typedef_type, reader, base_offset)?;

                CrpfNode::Typedef(CrpfTypedef {
                    type_name,
                    value: Box::new(typedef_value),
                })
            },
            PrimitiveType::Struct => {
                let mut fields = HashMap::new();
                let field_info_base_index = type_entry.field_info_start_index as u32;
                let type_name = crpf.get_name(type_entry.name1_offset)?.clone();

                let parent = if type_entry.ref_type_index != 0 {
                    let parent_type = crpf.get_type(type_entry.ref_type_index)?;
                    let parent_value = Self::convert_type(crpf, parent_type, reader, base_offset)?;

                    Some(Box::new(parent_value))
                } else {
                    None
                };

                for i in 0..type_entry.field_count {
                    let field_info_index = field_info_base_index + i;
                    let field_info = crpf.get_field_info(field_info_index)?;
                    let field_type = crpf.get_type(field_info.type_index)?;
                    let field_name = crpf.get_name(field_info.key_offset)?;

                    fields.insert(field_name.clone(), Self::convert_type(crpf, field_type, reader, base_offset + field_info.offset as u64)?);
                }

                CrpfNode::Struct(CrpfStruct {
                    type_name,
                    fields,
                    parent,
                })
            },
            PrimitiveType::StaticArray => CrpfNode::StaticArray,
            PrimitiveType::DsArray => CrpfNode::DsArray,
            PrimitiveType::DsString => CrpfNode::DsString,
            PrimitiveType::DsOptional => CrpfNode::DsOptional,
            PrimitiveType::DsVariant => CrpfNode::DsVariant,
            PrimitiveType::BlobArray => {
                // format: [u32: offset] [u32: count] [[T; count]: data]
                let component_type = crpf.get_opt_type(type_entry.ref_type_index);
                let mut values = Vec::new();

                if let Some(component_type) = component_type {
                    let mut offset = reader.read_u32()? as u64;
                    let count = reader.read_u32()?;

                    offset += reader.stream_position()? - 8;

                    for _ in 0..count {
                        values.push(Self::convert_type(crpf, component_type, reader, offset)?);
                        offset += component_type.size as u64;
                    }
                }

                CrpfNode::BlobArray(values)
            }
            PrimitiveType::BlobString => {
                // format: [u32: offset] [u32: length] [[u8; length]: data]
                let offset = reader.read_u32_offset()?;
                let length = reader.read_u32()?;

                reader.seek(SeekFrom::Start(offset))?;

                let mut data = vec![0; length as usize];
                reader.read_exact(&mut data)?;

                CrpfNode::BlobString(String::from_utf8_lossy(&data).to_string())
            }
            PrimitiveType::BlobOptional => {
                let component_type = crpf.get_opt_type(type_entry.ref_type_index);
                let offset = reader.read_u32()? as u64;

                if let Some(component_type) = component_type {
                    if offset != 0 {
                        CrpfNode::BlobOptional(BlobOptional(Some(Box::new(Self::convert_type(crpf, component_type, reader, base_offset + offset)?))))
                    } else {
                        panic!("BlobOptional with offset 0 but a type has been specified");
                    }
                } else {
                    if offset != 0 {
                        panic!("BlobOptional with offset {} but no type has been specified", offset);
                    }

                    CrpfNode::BlobOptional(BlobOptional(None))
                }
            }
            PrimitiveType::BlobVariant => {
                // format: [u32: type_hash1] [u32: offset] [u32: size] [[u8; size]: data]

                let variant_type_hash = reader.read_u32()?;
                let offset = reader.read_u32_offset()?;
                let size = reader.read_u32()?;

                if size == 0 { // TODO: might be a wrong guess
                    return Ok(CrpfNode::BlobVariant(BlobVariant(Box::new(CrpfNode::None))));
                }

                let component_type = crpf.get_type_by_hash(variant_type_hash)?;

                reader.seek(SeekFrom::Start(offset))?;

                let value = Self::convert_type(crpf, component_type, reader, offset)?;

                CrpfNode::BlobVariant(BlobVariant(Box::new(value)))
            },
            PrimitiveType::ObjectReference => {
                CrpfNode::ObjectReference(ObjectReference(Guid::read(reader)?))
            }
            PrimitiveType::Guid => {
                CrpfNode::Guid(Guid::read(reader)?)
            }
        };

        Ok(value)
    }
}

