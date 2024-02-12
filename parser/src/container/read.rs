use std::io::Read;
use shared::io::Reader;
use crate::container::{KFC_DIR_MAGIC, KFCDir, KFCDirEntry, KFCHeader};
use crate::error::ParseError;

impl KFCDir {
    pub fn read<T: Read>(reader: &mut Reader<T>) -> Result<Self, ParseError> {
        let header = KFCHeader::read(reader)?;
        let mut entries = Vec::new();

        for _ in 0..header.entry_count {
            entries.push(KFCDirEntry::read_name(reader)?);
        }

        for entry in &mut entries {
            entry.read_info(reader)?;
        }

        for entry in &mut entries {
            entry.read_offset(reader)?;
        }

        Ok(Self {
            header,
            entries,
        })
    }
}

impl KFCHeader {
    fn read<T: Read>(reader: &mut Reader<T>) -> Result<Self, ParseError> {
        let magic = reader.read_u32()?;

        if magic != KFC_DIR_MAGIC {
            return Err(ParseError::InvalidMagic(magic));
        }

        let entry_count = reader.read_u32()?;
        let entry_count2 = reader.read_u32()?;

        if entry_count != entry_count2 {
            return Err(ParseError::InvalidEntryCount(entry_count));
        }

        let reserved = reader.read_u32()?;
        let data_size = reader.read_u64()?;

        Ok(Self {
            magic,
            entry_count,
            reserved,
            data_size,
        })
    }
}

impl KFCDirEntry {
    fn read_name<T: Read>(reader: &mut Reader<T>) -> Result<Self, ParseError> {
        let name_hash = reader.read_u64()?;

        Ok(Self {
            name_hash,
            decompressed_size: 0,
            compressed_size: 0,
            index: 0,
            flags: 0,
            offset: 0,
        })
    }

    fn read_info<T: Read>(&mut self, reader: &mut Reader<T>) -> Result<(), ParseError> {
        self.decompressed_size = reader.read_u32()?;
        self.compressed_size = reader.read_u32()?;
        self.index = reader.read_u32()?;
        self.flags = reader.read_u32()?;

        Ok(())
    }

    fn read_offset<T: Read>(&mut self, reader: &mut Reader<T>) -> Result<(), ParseError> {
        self.offset = reader.read_u64()?;

        Ok(())
    }
}
