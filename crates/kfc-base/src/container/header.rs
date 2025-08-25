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
///     KFCLocation containers;
///
///     KFCLocation unused0;
///     KFCLocation unused1;
///
///     KFCLocation resource_locations;
///     KFCLocation resource_indices;
///
///     KFCLocation content_buckets;
///     KFCLocation content_keys;
///     KFCLocation content_values;
///
///     KFCLocation resource_buckets;
///     KFCLocation resource_keys;
///     KFCLocation resource_values;
///
///     KFCLocation resource_bundle_buckets;
///     KFCLocation resource_bundle_keys;
///     KFCLocation resource_bundle_values;
/// };
/// ```
#[derive(Debug, Clone, Default)]
pub struct KFCHeader {
    pub size: u64,
    // pub unk0: u32,

    pub version: KFCLocation,
    pub containers: KFCLocation,

    pub unused0: KFCLocation,
    pub unused1: KFCLocation,

    pub resource_locations: KFCLocation,
    pub resource_indices: KFCLocation,

    pub content_buckets: KFCLocation,
    pub content_keys: KFCLocation,
    pub content_values: KFCLocation,

    pub resource_buckets: KFCLocation,
    pub resource_keys: KFCLocation,
    pub resource_values: KFCLocation,

    pub resource_bundle_buckets: KFCLocation,
    pub resource_bundle_keys: KFCLocation,
    pub resource_bundle_values: KFCLocation,
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
        let containers = KFCLocation::read(reader)?;

        let unused0 = KFCLocation::read(reader)?;
        let unused1 = KFCLocation::read(reader)?;

        let resource_locations = KFCLocation::read(reader)?;
        let resource_indices = KFCLocation::read(reader)?;

        let content_buckets = KFCLocation::read(reader)?;
        let content_keys = KFCLocation::read(reader)?;
        let content_values = KFCLocation::read(reader)?;

        let resource_buckets = KFCLocation::read(reader)?;
        let resource_keys = KFCLocation::read(reader)?;
        let resource_values = KFCLocation::read(reader)?;

        let resource_bundle_buckets = KFCLocation::read(reader)?;
        let resource_bundle_keys = KFCLocation::read(reader)?;
        let resource_bundle_values = KFCLocation::read(reader)?;

        Ok(Self {
            size,

            version,
            containers,

            unused0,
            unused1,

            resource_locations,
            resource_indices,

            content_buckets,
            content_keys,
            content_values,

            resource_buckets,
            resource_keys,
            resource_values,

            resource_bundle_buckets,
            resource_bundle_keys,
            resource_bundle_values,
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
        self.containers.write(writer)?;

        self.unused0.write(writer)?;
        self.unused1.write(writer)?;

        self.resource_locations.write(writer)?;
        self.resource_indices.write(writer)?;

        self.content_buckets.write(writer)?;
        self.content_keys.write(writer)?;
        self.content_values.write(writer)?;

        self.resource_buckets.write(writer)?;
        self.resource_keys.write(writer)?;
        self.resource_values.write(writer)?;

        self.resource_bundle_buckets.write(writer)?;
        self.resource_bundle_keys.write(writer)?;
        self.resource_bundle_values.write(writer)?;

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
/// struct ContainerInfo {
///     u64 size;
///     u64 count;
/// };
/// ```
#[derive(Debug, Clone, Default)]
pub struct ContainerInfo {
    pub size: u64,
    pub count: usize,
}

impl ContainerInfo {

    #[inline]
    pub fn read<R: Read>(reader: &mut R) -> Result<Self, KFCReadError> {
        let size = reader.read_u64()?;
        let count = reader.read_u64()? as usize;

        Ok(Self {
            size,
            count,
        })
    }

    #[inline]
    pub fn write<W: Write + Seek>(&self, writer: &mut W) -> Result<(), KFCWriteError> {
        writer.write_u64(self.size)?;
        writer.write_u64(self.count as u64)?;

        Ok(())
    }

}

/// # Layout
/// ```c
/// struct ResourceLocation {
///     u32 offset;
///     u32 size;
///     u32 count;
/// };
/// ```
#[derive(Debug, Clone, Default)]
pub struct ResourceLocation {
    pub offset: u64,
    pub size: u64,
    pub count: usize,
}

impl ResourceLocation {

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
/// struct ContentEntry {
///     u32 offset;
///     u16 flags;
///     u16 container_index;
///     u8 padding[8];
/// };
/// ```
#[derive(Debug, Clone, Default)]
pub struct ContentEntry {
    pub offset: u64,
    pub flags: u16,
    pub container_index: usize,
}

impl ContentEntry {

    #[inline]
    pub fn new(offset: u64, flags: u16, container_index: usize) -> Self {
        Self {
            offset,
            flags,
            container_index,
        }
    }

    #[inline]
    pub fn read<R: Read>(reader: &mut R) -> Result<Self, KFCReadError> {
        let offset = reader.read_u32()? as u64;
        let flags = reader.read_u16()?;
        let container_index = reader.read_u16()? as usize;
        reader.padding(8)?;

        Ok(Self {
            offset,
            flags,
            container_index,
        })
    }

    #[inline]
    pub fn write<W: Write + Seek>(&self, writer: &mut W) -> Result<(), KFCWriteError> {
        writer.write_u32(self.offset as u32)?;
        writer.write_u16(self.flags)?;
        writer.write_u16(self.container_index as u16)?;
        writer.padding(8)?;

        Ok(())
    }

}

/// # Layout
/// ```c
/// struct ResourceEntry {
///     u32 offset;
///     u32 size;
/// };
/// ```
#[derive(Debug, Clone, Default)]
pub struct ResourceEntry {
    pub offset: u64,
    pub size: u64,
}

impl ResourceEntry {

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
/// struct ResourceBundleEntry {
///     u32 internal_hash;
///     u32 index;
///     u32 count;
/// };
/// ```
#[derive(Debug, Clone, Default)]
pub struct ResourceBundleEntry {
    pub internal_hash: Hash32,
    pub index: usize,
    pub count: usize,
}

impl ResourceBundleEntry {

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
