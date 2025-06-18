use std::io::{Cursor, Read, Seek, SeekFrom};
use std::path::Path;

use crate::io::ReadExt;

use super::error::PEParseError;

pub const DOS_SIGNATURE: u16 = 0x5A4D;
pub const NT_SIGNATURE: u32 = 0x00004550;

pub const PE32_MAGIC: u16 = 0x010B;
pub const PE32_PLUS_MAGIC: u16 = 0x020B;

#[derive(Debug)]
struct Section {
    name: String,
    // virtual_size: u32,
    virtual_address: u32,
    size_of_raw_data: u32,
    pointer_to_raw_data: u32,
}

pub struct PEFile {
    data: Vec<u8>,
    sections: Vec<Section>,
    image_base: u64,
}

impl PEFile {

    pub fn load_from_file<P: AsRef<Path>>(
        path: P
    ) -> Result<Self, PEParseError> {
        // load things in memory since we need to read through the file multiple times
        let data = std::fs::read(path)?;
        let mut reader = Cursor::new(&data);

        reader.seek(SeekFrom::Start(0))?;

        let dos_signature = reader.read_u16()?;
        if dos_signature != DOS_SIGNATURE {
            return Err(PEParseError::InvalidDosSignature);
        }

        reader.seek(SeekFrom::Start(60))?; // jump to e_lfanew
        let pe_offset = reader.read_u32()?;

        // coff header
        reader.seek(SeekFrom::Start(pe_offset as u64))?;
        let pe_signature = reader.read_u32()?;

        if pe_signature != NT_SIGNATURE {
            return Err(PEParseError::InvalidNTSignature);
        }

        let _machine = reader.read_u16()?;
        let number_of_sections = reader.read_u16()?;
        let _time_date_stamp = reader.read_u32()?;
        let _pointer_to_symbol_table = reader.read_u32()?;
        let _number_of_symbols = reader.read_u32()?;
        let size_of_optional_header = reader.read_u16()?;
        let _characteristics = reader.read_u16()?;

        // optional header
        let opt_header_start = reader.stream_position()?;
        let magic = reader.read_u16()?;
        let image_base = match magic {
            PE32_MAGIC => {
                panic!("PE32 not supported");
                // reader.seek(SeekFrom::Current(26))?; // skip to image base
                // reader.read_u32()? as u64
            }
            PE32_PLUS_MAGIC => {
                reader.seek(SeekFrom::Current(22))?; // skip to image base
                reader.read_u64()?
            }
            _ => return Err(PEParseError::UnsupportedPEType),
        };

        let section_start = opt_header_start + size_of_optional_header as u64;
        reader.seek(SeekFrom::Start(section_start))?; // skip optional header

        // section headers
        let mut sections = Vec::new();

        for _ in 0..number_of_sections {
            let mut name = [0; 8];
            reader.read_exact(&mut name)?;
            let index_of_nul = name.iter()
                .position(|&c| c == 0)
                .ok_or(PEParseError::MalformedSectionName)?;
            let name = String::from_utf8(name[..index_of_nul].to_vec())
                .map_err(|_| PEParseError::MalformedSectionName)?;

            let _virtual_size = reader.read_u32()?;
            let virtual_address = reader.read_u32()?;
            let size_of_raw_data = reader.read_u32()?;
            let pointer_to_raw_data = reader.read_u32()?;
            let _pointer_to_relocations = reader.read_u32()?;
            let _pointer_to_line_numbers = reader.read_u32()?;
            let _number_of_relocations = reader.read_u16()?;
            let _number_of_line_numbers = reader.read_u16()?;
            let _characteristics = reader.read_u32()?;

            sections.push(Section {
                name,
                // virtual_size,
                virtual_address,
                size_of_raw_data,
                pointer_to_raw_data,
            });
        }

        Ok(Self {
            data,
            sections,
            image_base,
        })
    }

    pub fn va_to_fo(&self, va: u64) -> Option<u64> {
        if va < self.image_base {
            return None;
        }

        let va = (va - self.image_base) as u32;

        for section in &self.sections {
            let start = section.virtual_address;
            let end = start + section.size_of_raw_data;

            if va >= start && va < end {
                return Some((section.pointer_to_raw_data + (va - start)) as u64);
            }
        }

        None
    }

    pub fn fo_to_va(&self, offset: u64) -> Option<u64> {
        let offset = offset as u32;

        for section in &self.sections {
            let start = section.pointer_to_raw_data;
            let end = start + section.size_of_raw_data;

            if offset >= start && offset < end {
                return Some((section.virtual_address + (offset - start)) as u64 + self.image_base);
            }
        }

        None
    }

    pub fn find<const N: usize>(
        &self,
        from_offset: u64,
        needle: [u8; N],
        alignment: usize,
    ) -> Option<u64> {
        let haystack = &self.data[from_offset as usize..];
        let mut ptr = 0;

        while ptr < haystack.len() - N {
            if haystack[ptr..].starts_with(&needle) {
                return Some(ptr as u64 + from_offset);
            }

            ptr += alignment;
        }

        // try again without alignment
        if alignment > 1 {
            return self.find(from_offset, needle, 1);
        }

        None
    }

    pub fn offset_to_section(&self, name: &str) -> Option<u64> {
        self.sections.iter()
            .find(|s| s.name == name)
            .map(|s| s.pointer_to_raw_data as u64)
    }

    pub fn find_pointer_to_0va(
        &self,
        from_offset: u64,
        va: u64,
    ) -> Option<u64> {
        let needle = super::util::prefix_pattern::<8, 9>(
            va.to_le_bytes(),
            0x00
        );

        self.find(from_offset - 1, needle, 8)
            .map(|offset| offset + 1)
    }

    pub fn get_cursor_at(&self, offset: u64) -> std::io::Result<Cursor<&[u8]>> {
        self.data.get(offset as usize..)
            .map(Cursor::new)
            .ok_or_else(|| std::io::Error::other("Out of bounds"))
    }

}

pub trait ReadPEExt {

    fn read_pointee<'a>(
        &mut self,
        file: &'a PEFile
    ) -> std::io::Result<Cursor<&'a [u8]>>;

    fn read_pointee_opt<'a>(
        &mut self,
        file: &'a PEFile
    ) -> std::io::Result<Option<Cursor<&'a [u8]>>>;

}

impl<R: Read + Seek> ReadPEExt for R {

    fn read_pointee<'a>(
        &mut self,
        file: &'a PEFile
    ) -> std::io::Result<Cursor<&'a [u8]>> {
        file.va_to_fo(self.read_u64()?)
            .and_then(|offset| file.data.get(offset as usize..))
            .map(Cursor::new)
            .ok_or_else(|| std::io::Error::other("Could not read pointee"))
    }

    fn read_pointee_opt<'a>(
        &mut self,
        file: &'a PEFile
    ) -> std::io::Result<Option<Cursor<&'a [u8]>>> {
        let va = self.read_u64()?;

        if va == 0 {
            return Ok(None);
        }

        file.va_to_fo(va)
            .and_then(|offset| file.data.get(offset as usize..))
            .map(Cursor::new)
            .map(Some)
            .ok_or_else(|| std::io::Error::other("Could not read pointee"))
    }

}
