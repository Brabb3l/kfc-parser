use std::io::Read;
use shared::io::ReadExt;

use super::{BlobGuid, DescriptorGuid};

impl BlobGuid {
    pub fn read<R: Read>(reader: &mut R) -> std::io::Result<Self> {
        let mut data = [0; 16];
        reader.read_exact(&mut data)?;

        Ok(Self {
            data
        })
    }
}

impl DescriptorGuid {
    pub fn read<R: Read>(reader: &mut R) -> std::io::Result<Self> {
        let mut data = [0; 16];
        reader.read_exact(&mut data)?;

        let type_hash = reader.read_u32()?;

        let part_number = reader.read_u32()?;
        let reserved = reader.read_u64()?;

        Ok(Self {
            data,
            type_hash,
            part_number,
            reserved
        })
    }
}

