use std::io::{Seek, SeekFrom, Write};

use shared::io::{WriteExt, WriterSeekExt};

use super::*;
use crate::error::ParseError;

impl KFCFile {
    pub fn write<W: Write + Seek>(&self, writer: &mut W) -> Result<(), ParseError> {
        KFCHeader::default().write(writer)?;
        
        // KFCFile::version
        let version_offset = writer.stream_position()?;
        writer.write_string(&self.version, self.version.len())?;
        writer.align(8)?;
        
        // KFCFile::dat_infos
        let dat_infos_offset = writer.stream_position()?;
        for dat_info in &self.dat_infos {
            dat_info.write(writer)?;
        }
        
        // KFCFile::descriptor_locations
        let descriptor_locations_offset = writer.stream_position()?;
        DescriptorLocation::default().write(writer)?;
        
        // KFCFile::descriptor_indices
        let descriptor_indices_offset = writer.stream_position()?;
        for descriptor_index in &self.descriptor_indices {
            writer.write_u32(*descriptor_index)?;
        }
        
        // KFCFile::blob_ids
        let blob_ids_offset = writer.stream_position()?;
        for blob_bucket in &self.blob_buckets {
            blob_bucket.write(writer)?;
        }
        
        // KFCFile::blob_guids
        let blob_guids_offset = writer.stream_position()?;
        for blob_guid in &self.blob_guids {
            blob_guid.write(writer)?;
        }
        writer.align(8)?;
        
        // KFCFile::blob_links
        let blob_links_offset = writer.stream_position()?;
        for blob_link in &self.blob_links {
            blob_link.write(writer)?;
        }
        
        // KFCFile::descriptor_ids
        let descriptor_ids_offset = writer.stream_position()?;
        for descriptor_bucket in &self.descriptor_buckets {
            descriptor_bucket.write(writer)?;
        }
        
        // KFCFile::descriptor_guids
        let descriptor_guids_offset = writer.stream_position()?;
        for descriptor_guid in &self.descriptor_guids {
            descriptor_guid.write(writer)?;
        }
        writer.align(8)?;
        
        // KFCFile::descriptor_links
        let descriptor_links_offset = writer.stream_position()?;
        for descriptor_link in &self.descriptor_links {
            descriptor_link.write(writer)?;
        }
        
        // KFCFile::preload_ids
        let preload_ids_offset = writer.stream_position()?;
        for preload_bucket in &self.preload_buckets {
            preload_bucket.write(writer)?;
        }
        
        // KFCFile::preload_guids
        let preload_guids_offset = writer.stream_position()?;
        for preload_guid in &self.preload_guids {
            preload_guid.write(writer)?;
        }
        writer.align(8)?;
        
        // KFCFile::preload_links
        let preload_links_offset = writer.stream_position()?;
        for preload_link in &self.preload_links {
            preload_link.write(writer)?;
        }
        
        let file_size = writer.stream_position()? as u32;
        
        // DescriptorLocation
        writer.seek(SeekFrom::Start(descriptor_locations_offset))?;
        let descriptor_location = DescriptorLocation {
            offset: file_size,
            size: self.descriptor_locations[0].size,
            count: self.descriptor_locations[0].count,
        };
        descriptor_location.write(writer)?;
        
        // KFCHeader
        let header = KFCHeader {
            size: file_size,
            version: KFCLocation {
                offset: version_offset,
                count: self.version.len() as u32,
            },
            dat_infos: KFCLocation {
                offset: dat_infos_offset,
                count: self.dat_infos.len() as u32,
            },
            descriptor_locations: KFCLocation {
                offset: descriptor_locations_offset,
                count: self.descriptor_locations.len() as u32,
            },
            descriptor_indices: KFCLocation {
                offset: descriptor_indices_offset,
                count: self.descriptor_indices.len() as u32,
            },
            blob_buckets: KFCLocation {
                offset: blob_ids_offset,
                count: self.blob_buckets.len() as u32,
            },
            blob_guids: KFCLocation {
                offset: blob_guids_offset,
                count: self.blob_guids.len() as u32,
            },
            blob_links: KFCLocation {
                offset: blob_links_offset,
                count: self.blob_links.len() as u32,
            },
            descriptor_buckets: KFCLocation {
                offset: descriptor_ids_offset,
                count: self.descriptor_buckets.len() as u32,
            },
            descriptor_guids: KFCLocation {
                offset: descriptor_guids_offset,
                count: self.descriptor_guids.len() as u32,
            },
            descriptor_links: KFCLocation {
                offset: descriptor_links_offset,
                count: self.descriptor_links.len() as u32,
            },
            preload_buckets: KFCLocation {
                offset: preload_ids_offset,
                count: self.preload_buckets.len() as u32,
            },
            preload_guids: KFCLocation {
                offset: preload_guids_offset,
                count: self.preload_guids.len() as u32,
            },
            preload_links: KFCLocation {
                offset: preload_links_offset,
                count: self.preload_links.len() as u32,
            },
            ..Default::default()
        };
        
        writer.seek(SeekFrom::Start(0))?;
        header.write(writer)?;
        
        writer.seek(SeekFrom::Start(file_size as u64))?;
        
        Ok(())
    }
}

impl KFCHeader {
    pub fn write<W: Write + Seek>(&self, writer: &mut W) -> Result<(), ParseError> {
        writer.write_u32(KFC_DIR_MAGIC)?;
        writer.write_u32(self.size)?;
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
        
        self.preload_buckets.write(writer)?;
        self.preload_guids.write(writer)?;
        self.preload_links.write(writer)?;
        
        Ok(())
    }
}

impl KFCLocation {
    #[inline]
    pub fn write<W: Write + Seek>(&self, writer: &mut W) -> Result<(), ParseError> {
        writer.write_offset(self.offset)?;
        writer.write_u32(self.count)?;
        
        Ok(())
    }
}

impl DatInfo {
    #[inline]
    pub fn write<W: Write + Seek>(&self, writer: &mut W) -> Result<(), ParseError> {
        writer.write_u16(self.largest_chunk_size)?;
        writer.write_u32(self.unk0)?;
        writer.padding(2)?;
        writer.write_u32(self.count)?;
        writer.padding(4)?;
        
        Ok(())
    }
}

impl DescriptorLocation {
    #[inline]
    pub fn write<W: Write + Seek>(&self, writer: &mut W) -> Result<(), ParseError> {
        writer.write_u32(self.offset)?;
        writer.write_u32(self.size)?;
        writer.write_u32(self.count)?;
        
        Ok(())
    }
}

impl BlobBucket {
    #[inline]
    pub fn write<W: Write + Seek>(&self, writer: &mut W) -> Result<(), ParseError> {
        writer.write_u32(self.index)?;
        writer.write_u32(self.count)?;
        
        Ok(())
    }
}

impl BlobLink {
    #[inline]
    pub fn write<W: Write + Seek>(&self, writer: &mut W) -> Result<(), ParseError> {
        writer.write_u32(self.offset)?;
        writer.write_u16(self.flags)?;
        writer.write_u16(self.dat_index)?;
        writer.padding(8)?;
        
        Ok(())
    }
}

impl DescriptorBucket {
    #[inline]
    pub fn write<W: Write + Seek>(&self, writer: &mut W) -> Result<(), ParseError> {
        writer.write_u32(self.index)?;
        writer.write_u32(self.count)?;
        
        Ok(())
    }
}

impl DescriptorLink {
    #[inline]
    pub fn write<W: Write + Seek>(&self, writer: &mut W) -> Result<(), ParseError> {
        writer.write_u32(self.offset)?;
        writer.write_u32(self.size)?;
        
        Ok(())
    }
}

impl PreloadBucket {
    #[inline]
    pub fn write<W: Write + Seek>(&self, writer: &mut W) -> Result<(), ParseError> {
        writer.write_u32(self.index)?;
        writer.write_u32(self.count)?;
        
        Ok(())
    }
}

impl PreloadGuid {
    #[inline]
    pub fn write<W: Write + Seek>(&self, writer: &mut W) -> Result<(), ParseError> {
        writer.write_u32(self.hash)?;
        
        Ok(())
    }
}

impl PreloadLink {
    #[inline]
    pub fn write<W: Write + Seek>(&self, writer: &mut W) -> Result<(), ParseError> {
        writer.write_u32(self.type_hash_2)?;
        writer.write_u32(self.descriptor_index)?;
        writer.write_u32(self.unk0)?;
        
        Ok(())
    }
}
