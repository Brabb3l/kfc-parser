use std::collections::BTreeMap;
use std::io::{Read, Seek, SeekFrom};
use shared::io::Reader;
use crate::crpf::{Crpf, CRPF_MAGIC, CrpfHeader, Ctcb, CTCB_MAGIC, CtcbFieldInfo, CtcbHeader, CtcbNamespace, CtcbTypeEntry, Kbf, KBF_MAGIC, KbfHeader};
use crate::error::{CrpfError, ParseError};
use crate::types::{CrpfGuid, Guid, PrimitiveType};

impl Crpf {
    pub fn read<T: Read + Seek>(reader: &mut Reader<T>) -> Result<Self, ParseError> {
        let header = CrpfHeader::read(reader)?;

        reader.seek(SeekFrom::Start(header.guid_offset))?;

        let guid = CrpfGuid::read(reader)?;
        let r#type = reader.read_u32()?;
        let unk0 = reader.read_u32()?;

        let mut data_guids = Vec::new();
        let mut crpf_guids = Vec::new();

        if header.data_guids_count > 0 {
            reader.seek(SeekFrom::Start(header.data_guids_offset))?;

            for _ in 0..header.data_guids_count {
                data_guids.push(Guid::read(reader)?);
            }
        }

        if header.crpf_guids_count > 0 {
            reader.seek(SeekFrom::Start(header.crpf_guids_offset))?;

            for _ in 0..header.crpf_guids_count {
                crpf_guids.push(CrpfGuid::read(reader)?);
            }
        }

        reader.seek(SeekFrom::Start(header.name_offset))?;

        let name = reader.read_string(header.name_len as usize)?;

        reader.seek(SeekFrom::Start(header.kbf_offset))?;

        let kbf = Kbf::read(reader)?;
        let ctcb = Ctcb::read(reader)?;

        Ok(Self {
            header,
            guid,
            r#type,
            unk0,
            data_guids,
            crpf_guids,
            kbf,
            ctcb,
            name,
        })
    }
}

impl CrpfHeader {
    pub fn read<T: Read + Seek>(reader: &mut Reader<T>) -> Result<Self, ParseError> {
        let magic = reader.read_u32()?;

        if magic != CRPF_MAGIC {
            return Err(CrpfError::InvalidMagic(magic).into());
        }

        let unk0 = reader.read_u32()?;
        let unk1 = reader.read_u32()?;
        let unk2 = reader.read_u32()?;

        let guid_offset = reader.read_u32_offset()?;
        let unk3 = reader.read_u32()?;
        let crpf_guids_offset = reader.read_u32_offset()?;
        let crpf_guids_count = reader.read_u32()?;
        let data_guids_offset = reader.read_u32_offset()?;
        let data_guids_count = reader.read_u32()?;
        let kbf_offset = reader.read_u32_offset()?;
        let kbf_size = reader.read_u32()?;
        let name_data_offset = reader.read_u32_offset()?;
        let name_data_unk = reader.read_u32()?;
        let name_offset = reader.read_u32_offset()?;
        let name_len = reader.read_u32()?;

        Ok(Self {
            magic,
            unk0,
            unk1,
            unk2,
            guid_offset,
            unk3,
            crpf_guids_offset,
            crpf_guids_count,
            data_guids_offset,
            data_guids_count,
            kbf_offset,
            kbf_size,
            name_data_offset,
            name_data_unk,
            name_offset,
            name_len,
        })
    }
}

impl Kbf {
    pub fn read<T: Read + Seek>(reader: &mut Reader<T>) -> Result<Self, ParseError> {
        let header = KbfHeader::read(reader)?;

        let mut data = vec![0; header.kbf_content_size as usize];
        reader.read_exact(&mut data)?;

        // align to 8 bytes
        let padding = 8 - (reader.stream_position()? % 8);

        if padding != 8 {
            reader.seek(SeekFrom::Current(padding as i64))?;
        }

        Ok(Self {
            header,
            data,
        })
    }
}

impl KbfHeader {
    pub fn read<T: Read>(reader: &mut Reader<T>) -> Result<Self, ParseError> {
        let magic = reader.read_u32()?;

        if magic != KBF_MAGIC {
            return Err(CrpfError::InvalidKBFMagic(magic).into());
        }

        let name_offset = reader.read_u32()?;
        let crpf_type = reader.read_u32()?;
        let r#type = reader.read_u32()?;
        let kbf_content_size = reader.read_u32()?;
        let ctcb_size = reader.read_u32()?;
        let unk1 = reader.read_u64()?;
        let unk2 = reader.read_u64()?;
        let unk3 = reader.read_u64()?;

        Ok(Self {
            magic,
            name_offset,
            crpf_type,
            r#type,
            kbf_content_size,
            ctcb_size,
            unk1,
            unk2,
            unk3,
        })
    }
}

impl Ctcb {
    pub fn read<T: Read + Seek>(reader: &mut Reader<T>) -> Result<Self, ParseError> {
        let header = CtcbHeader::read(reader)?;
        let mut namespaces = Vec::new();
        let mut type_entries = Vec::new();
        let mut name_entries = BTreeMap::new();
        let mut value_entries = Vec::new();

        reader.seek(SeekFrom::Start(header.namespace_offset))?;

        for _ in 0..header.namespace_count {
            namespaces.push(CtcbNamespace::read(reader)?);
        }

        reader.seek(SeekFrom::Start(header.type_table_offset))?;

        for _ in 0..header.type_table_count {
            type_entries.push(CtcbTypeEntry::read(reader)?);
        }

        reader.seek(SeekFrom::Start(header.name_table_offset))?;

        let mut pos = 0;

        while reader.stream_position()? < header.name_table_offset + header.name_table_size as u64 {
            let byte = reader.read_u8()? + 1;
            pos += 1;
            name_entries.insert(pos, reader.read_string(byte as usize)?);
            pos += byte as u16;
        }

        // padding of 4-byte alignment //

        reader.seek(SeekFrom::Start(header.value_table_offset))?;

        for _ in 0..header.value_table_count {
            value_entries.push(CtcbFieldInfo::read(reader)?);
        }

        for type_entry in &mut type_entries {
            if type_entry.primitive_type == PrimitiveType::Enum {
                let enum_offset = (type_entry.enum_index as u64 - 1) * 16;

                reader.seek(SeekFrom::Start(header.enum_table_offset + enum_offset))?;

                for _ in 0..type_entry.field_count {
                    let name_offset = reader.read_u64()? as u16;
                    let value = reader.read_u64()?;

                    if let Some(name) = name_entries.get(&name_offset) {
                        type_entry.enum_values.insert(value, name.clone());
                    } else {
                        println!("Name not found: {:#?}", name_offset);
                    }
                }
            }
        }

        Ok(Self {
            header,
            namespaces,
            type_entries,
            name_entries,
            value_entries,
        })
    }
}

impl CtcbHeader {
    pub fn read<T: Read + Seek>(reader: &mut Reader<T>) -> Result<Self, ParseError> {
        let magic = reader.read_u32()?;

        if magic != CTCB_MAGIC {
            return Err(CrpfError::InvalidCTCBMagic(magic).into())
        }

        let unk0 = reader.read_u32()?;
        let namespace_offset = reader.read_u32_offset()?;
        let namespace_count = reader.read_u32()?;
        let type_table_offset = reader.read_u32_offset()?;
        let type_table_count = reader.read_u32()?;
        let name_table_offset = reader.read_u32_offset()?;
        let name_table_size = reader.read_u32()?;
        let value_table_offset = reader.read_u32_offset()?;
        let value_table_count = reader.read_u32()?;
        let enum_table_offset = reader.read_u32_offset()?;
        let enum_table_count = reader.read_u32()?;

        Ok(Self {
            magic,
            unk0,
            namespace_offset,
            namespace_count,
            type_table_offset,
            type_table_count,
            name_table_offset,
            name_table_size,
            value_table_offset,
            value_table_count,
            enum_table_offset,
            enum_table_count,
        })
    }
}

impl CtcbNamespace {
    pub fn read<T: Read>(reader: &mut Reader<T>) -> Result<Self, ParseError> {
        let name_offset = reader.read_u16()?;
        let unk1 = reader.read_u16()?;

        Ok(Self {
            name_offset,
            unk1,
        })
    }
}

impl CtcbTypeEntry {
    pub fn read<T: Read>(reader: &mut Reader<T>) -> Result<Self, ParseError> {
        let name0_offset = reader.read_u16()?;
        let name1_offset = reader.read_u16()?;
        let namespace_index = reader.read_u16()?;
        let ref_type_index = reader.read_u16()?;
        let size = reader.read_u32()?;
        let field_count = reader.read_u32()?;
        let primitive_type = PrimitiveType::from_u8(reader.read_u8()?);
        let unk0 = reader.read_u8()?;
        let unk1 = reader.read_u16()?;
        let type_hash1 = reader.read_u32()?;
        let type_hash2 = reader.read_u32()?;
        let field_info_start_index = reader.read_u16()?;
        let enum_index = reader.read_u16()?;

        Ok(Self {
            name0_offset,
            name1_offset,
            namespace_index,
            ref_type_index,
            size,
            field_count,
            primitive_type,
            unk0,
            unk1,
            type_hash1,
            type_hash2,
            field_info_start_index,
            enum_index,
            enum_values: BTreeMap::new(),
        })
    }
}

impl CtcbFieldInfo {
    pub fn read<T: Read>(
        reader: &mut Reader<T>,
    ) -> Result<Self, ParseError> {
        let key_offset = reader.read_u16()?;
        let type_index = reader.read_u16()?;
        let offset = reader.read_u32()?;

        Ok(Self {
            key_offset,
            type_index,
            offset,
        })
    }
}


