use std::io::{Read, Seek, Write};

use crate::{io::{ReadExt, ReadSeekExt, WriteExt, WriteSeekExt}, Hash32};

use super::{KFCReadError, KFCWriteError};

const KFC_DIR_MAGIC: u32 = 0x3243464B; // KFC2

/// # Layout
/// ```c
/// struct KFCHeader {
///     u32 magic; // KFC_DIR_MAGIC
///     u32 size;
///     u32 unk0; // 12
///     u8 padding[4];
///
///     KFCLocation version;
///     KFCLocation dat_infos;
///
///     KFCLocation unused0;
///     KFCLocation unused1;
///
///     KFCLocation descriptor_locations;
///     KFCLocation descriptor_indices;
///
///     KFCLocation blob_buckets;
///     KFCLocation blob_guids;
///     KFCLocation blob_links;
///
///     KFCLocation descriptor_buckets;
///     KFCLocation descriptor_guids;
///     KFCLocation descriptor_links;
///
///     KFCLocation group_buckets;
///     KFCLocation group_hashes;
///     KFCLocation group_infos;
/// };
/// ```
#[derive(Debug, Clone, Default)]
pub struct KFCHeader {
    pub size: u64,
    // pub unk0: u32,

    pub version: KFCLocation,
    pub dat_infos: KFCLocation,

    pub unused0: KFCLocation,
    pub unused1: KFCLocation,

    pub descriptor_locations: KFCLocation,
    pub descriptor_indices: KFCLocation,

    pub blob_buckets: KFCLocation,
    pub blob_guids: KFCLocation,
    pub blob_links: KFCLocation,

    pub descriptor_buckets: KFCLocation,
    pub descriptor_guids: KFCLocation,
    pub descriptor_links: KFCLocation,

    pub group_buckets: KFCLocation,
    pub group_hashes: KFCLocation,
    pub group_infos: KFCLocation,
}

impl KFCHeader {

    pub fn read<R: Read + Seek>(reader: &mut R) -> Result<Self, KFCReadError> {
        let magic = reader.read_u32()?;

        if magic != KFC_DIR_MAGIC {
            return Err(KFCReadError::InvalidMagic(magic));
        }

        let size = reader.read_u32()? as u64;
        let _unk0 = reader.read_u32()?;

        reader.padding(4)?;

        let version = KFCLocation::read(reader)?;
        let dat_infos = KFCLocation::read(reader)?;

        let unused0 = KFCLocation::read(reader)?;
        let unused1 = KFCLocation::read(reader)?;

        let descriptor_locations = KFCLocation::read(reader)?;
        let descriptor_indices = KFCLocation::read(reader)?;

        let blob_buckets = KFCLocation::read(reader)?;
        let blob_guids = KFCLocation::read(reader)?;
        let blob_links = KFCLocation::read(reader)?;

        let descriptor_buckets = KFCLocation::read(reader)?;
        let descriptor_guids = KFCLocation::read(reader)?;
        let descriptor_links = KFCLocation::read(reader)?;

        let group_buckets = KFCLocation::read(reader)?;
        let group_hashes = KFCLocation::read(reader)?;
        let group_infos = KFCLocation::read(reader)?;

        Ok(Self {
            size,

            version,
            dat_infos,

            unused0,
            unused1,

            descriptor_locations,
            descriptor_indices,

            blob_buckets,
            blob_guids,
            blob_links,

            descriptor_buckets,
            descriptor_guids,
            descriptor_links,

            group_buckets,
            group_hashes,
            group_infos,
        })
    }

    pub fn write<W: Write + Seek>(&self, writer: &mut W) -> Result<(), KFCWriteError> {
        if self.size > u32::MAX as u64 {
            return Err(KFCWriteError::SizeTooLarge(self.size));
        }

        writer.write_u32(KFC_DIR_MAGIC)?;
        writer.write_u32(self.size as u32)?;
        writer.write_u32(12)?;
        writer.padding(4)?;

        self.version.write(writer)?;
        self.dat_infos.write(writer)?;

        self.unused0.write(writer)?;
        self.unused1.write(writer)?;

        self.descriptor_locations.write(writer)?;
        self.descriptor_indices.write(writer)?;

        self.blob_buckets.write(writer)?;
        self.blob_guids.write(writer)?;
        self.blob_links.write(writer)?;

        self.descriptor_buckets.write(writer)?;
        self.descriptor_guids.write(writer)?;
        self.descriptor_links.write(writer)?;

        self.group_buckets.write(writer)?;
        self.group_hashes.write(writer)?;
        self.group_infos.write(writer)?;

        Ok(())
    }

}

/// # Layout
/// ```c
/// struct KFCLocation {
///     u32 offset;
///     u32 count;
/// };
/// ```
#[derive(Debug, Clone, Default)]
pub struct KFCLocation {
    pub offset: u64,
    pub count: usize,
}

impl KFCLocation {

    #[inline]
    pub fn new(offset: u64, count: usize) -> Self {
        Self {
            offset,
            count,
        }
    }

    #[inline]
    pub fn read<R: Read + Seek>(reader: &mut R) -> Result<Self, KFCReadError> {
        let offset = reader.read_u32_offset()?;
        let count = reader.read_u32()? as usize;

        Ok(Self {
            offset,
            count,
        })
    }

    #[inline]
    pub fn write<W: Write + Seek>(&self, writer: &mut W) -> Result<(), KFCWriteError> {
        writer.write_offset(self.offset)?;
        writer.write_u32(self.count as u32)?;

        Ok(())
    }

}

/// # Layout
/// ```c
/// struct DatInfo {
///     u64 size;
///     u32 count;
///     u8 padding[4];
/// };
/// ```
#[derive(Debug, Clone, Default)]
pub struct DatInfo {
    pub size: u64,
    pub count: usize,
}

impl DatInfo {

    #[inline]
    pub fn read<R: Read>(reader: &mut R) -> Result<Self, KFCReadError> {
        let size = reader.read_u64()?;
        let count = reader.read_u32()? as usize;
        reader.padding(4)?;

        Ok(Self {
            size,
            count,
        })
    }

    #[inline]
    pub fn write<W: Write + Seek>(&self, writer: &mut W) -> Result<(), KFCWriteError> {
        writer.write_u64(self.size)?;
        writer.write_u32(self.count as u32)?;
        writer.padding(4)?;

        Ok(())
    }

}

/// # Layout
/// ```c
/// struct DescriptorLocation {
///     u32 offset;
///     u32 size;
///     u32 count;
/// };
/// ```
#[derive(Debug, Clone, Default)]
pub struct DescriptorLocation {
    pub offset: u64,
    pub size: u64,
    pub count: usize,
}

impl DescriptorLocation {

    #[inline]
    pub fn read<R: Read>(reader: &mut R) -> Result<Self, KFCReadError> {
        let offset = reader.read_u32()? as u64;
        let size = reader.read_u32()? as u64;
        let count = reader.read_u32()? as usize;

        Ok(Self {
            offset,
            size,
            count,
        })
    }

    #[inline]
    pub fn write<W: Write + Seek>(&self, writer: &mut W) -> Result<(), KFCWriteError> {
        writer.write_u32(self.offset as u32)?;
        writer.write_u32(self.size as u32)?;
        writer.write_u32(self.count as u32)?;

        Ok(())
    }

}

/// # Layout
/// ```c
/// struct BlobLink {
///     u32 offset;
///     u16 flags;
///     u16 dat_index;
///     u8 padding[8];
/// };
/// ```
#[derive(Debug, Clone, Default)]
pub struct BlobLink {
    pub offset: u64,
    pub flags: u16,
    pub dat_index: usize,
}

impl BlobLink {

    #[inline]
    pub fn new(offset: u64, flags: u16, dat_index: usize) -> Self {
        Self {
            offset,
            flags,
            dat_index,
        }
    }

    #[inline]
    pub fn read<R: Read>(reader: &mut R) -> Result<Self, KFCReadError> {
        let offset = reader.read_u32()? as u64;
        let flags = reader.read_u16()?;
        let dat_index = reader.read_u16()? as usize;
        reader.padding(8)?;

        Ok(Self {
            offset,
            flags,
            dat_index,
        })
    }

    #[inline]
    pub fn write<W: Write + Seek>(&self, writer: &mut W) -> Result<(), KFCWriteError> {
        writer.write_u32(self.offset as u32)?;
        writer.write_u16(self.flags)?;
        writer.write_u16(self.dat_index as u16)?;
        writer.padding(8)?;

        Ok(())
    }

}

/// # Layout
/// ```c
/// struct DescriptorLink {
///     u32 offset;
///     u32 size;
/// };
/// ```
#[derive(Debug, Clone, Default)]
pub struct DescriptorLink {
    pub offset: u64,
    pub size: u64,
}

impl DescriptorLink {

    #[inline]
    pub fn new(offset: u64, size: u64) -> Self {
        Self {
            offset,
            size,
        }
    }

    #[inline]
    pub fn read<R: Read>(reader: &mut R) -> Result<Self, KFCReadError> {
        let offset = reader.read_u32()? as u64;
        let size = reader.read_u32()? as u64;

        Ok(Self {
            offset,
            size,
        })
    }

    #[inline]
    pub fn write<W: Write + Seek>(&self, writer: &mut W) -> Result<(), KFCWriteError> {
        writer.write_u32(self.offset as u32)?;
        writer.write_u32(self.size as u32)?;

        Ok(())
    }

}

/// # Layout
/// ```c
/// struct GroupInfo {
///     u32 internal_hash;
///     u32 index;
///     u32 count;
/// };
/// ```
#[derive(Debug, Clone, Default)]
pub struct GroupInfo {
    pub internal_hash: Hash32,
    pub index: usize,
    pub count: usize,
}

impl GroupInfo {

    #[inline]
    pub fn read<R: Read>(reader: &mut R) -> Result<Self, KFCReadError> {
        let internal_hash = reader.read_u32()?;
        let index = reader.read_u32()? as usize;
        let count = reader.read_u32()? as usize;

        Ok(Self {
            internal_hash,
            index,
            count,
        })
    }

    #[inline]
    pub fn write<W: Write + Seek>(&self, writer: &mut W) -> Result<(), KFCWriteError> {
        writer.write_u32(self.internal_hash)?;
        writer.write_u32(self.index as u32)?;
        writer.write_u32(self.count as u32)?;

        Ok(())
    }

}
