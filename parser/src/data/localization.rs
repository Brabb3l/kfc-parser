use serde::{Deserialize, Serialize};

use super::HashKey32;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LocaTagCollectionResourceData {
    pub tags: Vec<LocaTagResource>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LocaTagResource {
    pub id: HashKey32,
    pub text: String,
    pub arguments: Vec<LocaTagArgument>,
    pub generic_arguments: u32,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LocaTagArgument {
    pub id: u32,
    pub r#type: LocaArgumentType,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum LocaArgumentType {
    Generic,
    Input,
    Config,
    Balancing,
}

// pub fn deserialize_localization<R: Read + Seek>(
//     reader: &mut R
// ) -> anyhow::Result<Vec<LocaEntry>> {
//     let _unk0 = reader.read_u32()?;
//     let count = reader.read_u32()?;
//     let mut result = Vec::with_capacity(count as usize);

//     for _ in 0..count {
//         let key = reader.read_u32()?;
//         let offset = reader.read_u32_offset()?;
//         let length = reader.read_u32()?;
//         let unk1 = reader.read_u32()?;
//         let unk2 = reader.read_u32()?;
//         let unk3 = reader.read_u32()?;

//         let pos = reader.stream_position()?;

//         reader.seek(SeekFrom::Start(offset))?;

//         let value = reader.read_string(length as usize)?;

//         result.push(LocaEntry {
//             key,
//             value,
//             unk1,
//             unk2,
//             unk3
//         });

//         reader.seek(SeekFrom::Start(pos))?;
//     }

//     Ok(result)
// }
