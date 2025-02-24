use std::io::{Read, Seek, SeekFrom};
use serde::{Deserialize, Serialize};
use shared::io::{ReadExt, ReadSeekExt};

#[derive(Debug, Serialize, Deserialize)]
pub struct LocaEntry {
    pub key: u32,
    pub value: String,
    pub unk1: u32,
    pub unk2: u32,
    pub unk3: u32
}

pub fn deserialize_localization<R: Read + Seek>(
    reader: &mut R
) -> anyhow::Result<Vec<LocaEntry>> {
    let _unk0 = reader.read_u32()?;
    let count = reader.read_u32()?;
    let mut result = Vec::with_capacity(count as usize);

    for _ in 0..count {
        let key = reader.read_u32()?;
        let offset = reader.read_u32_offset()?;
        let length = reader.read_u32()?;
        let unk1 = reader.read_u32()?;
        let unk2 = reader.read_u32()?;
        let unk3 = reader.read_u32()?;

        let pos = reader.stream_position()?;

        reader.seek(SeekFrom::Start(offset))?;

        let value = reader.read_string(length as usize)?;

        result.push(LocaEntry {
            key,
            value,
            unk1,
            unk2,
            unk3
        });

        reader.seek(SeekFrom::Start(pos))?;
    }

    Ok(result)
}
