use std::io::{Read, Seek, SeekFrom};

use shared::io::{ReadExt, ReadSeekExt};

use crate::error::ParseError;
use super::*;

impl KFCFile {
    pub fn read<R: Read + Seek>(reader: &mut R) -> Result<Self, ParseError> {
        let header = KFCHeader::read(reader)?;
        
        // KFCFile::version
        reader.seek(SeekFrom::Start(header.version.offset))?;
        let version = reader.read_string(header.version.count as usize)?;
        
        // KFCFile::dat_infos
        reader.seek(SeekFrom::Start(header.dat_infos.offset))?;
        let dat_infos = (0..header.dat_infos.count)
            .map(|_| DatInfo::read(reader))
            .collect::<Result<Vec<_>, _>>()?;
        
        // KFCFile::descriptor_locations
        reader.seek(SeekFrom::Start(header.descriptor_locations.offset))?;
        let descriptor_locations = (0..header.descriptor_locations.count)
            .map(|_| DescriptorLocation::read(reader))
            .collect::<Result<Vec<_>, _>>()?;
        
        // KFCFile::descriptor_indices
        reader.seek(SeekFrom::Start(header.descriptor_indices.offset))?;
        let descriptor_indices = (0..header.descriptor_indices.count)
            .map(|_| reader.read_u32())
            .collect::<Result<Vec<_>, _>>()?;
        
        // KFCFile::blob_ids
        reader.seek(SeekFrom::Start(header.blob_buckets.offset))?;
        let blob_ids = (0..header.blob_buckets.count)
            .map(|_| BlobBucket::read(reader))
            .collect::<Result<Vec<_>, _>>()?;
        
        // KFCFile::blob_guids
        reader.seek(SeekFrom::Start(header.blob_guids.offset))?;
        let blob_guids = (0..header.blob_guids.count)
            .map(|_| BlobGuid::read(reader))
            .collect::<Result<Vec<_>, _>>()?;
        
        // KFCFile::blob_links
        reader.seek(SeekFrom::Start(header.blob_links.offset))?;
        let blob_links = (0..header.blob_links.count)
            .map(|_| BlobLink::read(reader))
            .collect::<Result<Vec<_>, _>>()?;
        
        // KFCFile::descriptor_ids
        reader.seek(SeekFrom::Start(header.descriptor_buckets.offset))?;
        let descriptor_ids = (0..header.descriptor_buckets.count)
            .map(|_| DescriptorBucket::read(reader))
            .collect::<Result<Vec<_>, _>>()?;
        
        // KFCFile::descriptor_guids
        reader.seek(SeekFrom::Start(header.descriptor_guids.offset))?;
        let descriptor_guids = (0..header.descriptor_guids.count)
            .map(|_| DescriptorGuid::read(reader))
            .collect::<Result<Vec<_>, _>>()?;
        
        // KFCFile::descriptor_links
        reader.seek(SeekFrom::Start(header.descriptor_links.offset))?;
        let descriptor_links = (0..header.descriptor_links.count)
            .map(|_| DescriptorLink::read(reader))
            .collect::<Result<Vec<_>, _>>()?;
        
        // KFCFile::preload_ids
        reader.seek(SeekFrom::Start(header.preload_buckets.offset))?;
        let preload_ids = (0..header.preload_buckets.count)
            .map(|_| PreloadBucket::read(reader))
            .collect::<Result<Vec<_>, _>>()?;
        
        // KFCFile::preload_guids
        reader.seek(SeekFrom::Start(header.preload_guids.offset))?;
        let preload_guids = (0..header.preload_guids.count)
            .map(|_| PreloadGuid::read(reader))
            .collect::<Result<Vec<_>, _>>()?;
        
        // KFCFile::preload_links
        reader.seek(SeekFrom::Start(header.preload_links.offset))?;
        let preload_links = (0..header.preload_links.count)
            .map(|_| PreloadLink::read(reader))
            .collect::<Result<Vec<_>, _>>()?;
        
        Ok(Self {
            version,
            dat_infos,
            
            descriptor_locations,
            descriptor_indices,

            blob_buckets: blob_ids,
            blob_guids,
            blob_links,

            descriptor_buckets: descriptor_ids,
            descriptor_guids,
            descriptor_links,

            preload_buckets: preload_ids,
            preload_guids,
            preload_links,
        })
    }
}

impl KFCHeader {
    pub fn read<R: Read + Seek>(reader: &mut R) -> Result<Self, ParseError> {
        let magic = reader.read_u32()?;

        if magic != KFC_DIR_MAGIC {
            return Err(ParseError::InvalidMagic(magic));
        }

        let size = reader.read_u32()?;
        let unk0 = reader.read_u32()?;
        
        reader.padding(4)?;

        let version = KFCLocation::read(reader)?;
        let dat_infos = KFCLocation::read(reader)?;
        
        let unused0 = KFCLocation::read(reader)?;
        let unused1 = KFCLocation::read(reader)?;
        
        let descriptor_locations = KFCLocation::read(reader)?;
        let descriptor_indices = KFCLocation::read(reader)?;
        
        let blob_ids = KFCLocation::read(reader)?;
        let blob_guids = KFCLocation::read(reader)?;
        let blob_links = KFCLocation::read(reader)?;
        
        let descriptor_ids = KFCLocation::read(reader)?;
        let descriptor_guids = KFCLocation::read(reader)?;
        let descriptor_links = KFCLocation::read(reader)?;
        
        let preload_ids = KFCLocation::read(reader)?;
        let preload_guids = KFCLocation::read(reader)?;
        let preload_links = KFCLocation::read(reader)?;
        
        Ok(Self {
            magic,
            size,
            unk0,
            
            version,
            dat_infos,
            
            unused0,
            unused1,
            
            descriptor_locations,
            descriptor_indices,

            blob_buckets: blob_ids,
            blob_guids,
            blob_links,

            descriptor_buckets: descriptor_ids,
            descriptor_guids,
            descriptor_links,

            preload_buckets: preload_ids,
            preload_guids,
            preload_links,
        })
    }
}

impl KFCLocation {
    #[inline]
    pub fn read<R: Read + Seek>(reader: &mut R) -> Result<Self, ParseError> {
        let offset = reader.read_u32_offset()?;
        let count = reader.read_u32()?;

        Ok(Self {
            offset,
            count,
        })
    }
}

impl DatInfo {
    #[inline]
    pub fn read<R: Read>(reader: &mut R) -> Result<Self, ParseError> {
        let largest_chunk_size = reader.read_u16()?;
        let unk0 = reader.read_u32()?;
        reader.padding(2)?;
        let count = reader.read_u32()?;
        reader.padding(4)?;

        Ok(Self {
            largest_chunk_size,
            unk0,
            count,
        })
    }
}

impl DescriptorLocation {
    #[inline]
    pub fn read<R: Read>(reader: &mut R) -> Result<Self, ParseError> {
        let offset = reader.read_u32()?;
        let size = reader.read_u32()?;
        let count = reader.read_u32()?;

        Ok(Self {
            offset,
            size,
            count,
        })
    }
}

impl BlobBucket {
    #[inline]
    pub fn read<R: Read>(reader: &mut R) -> Result<Self, ParseError> {
        let index = reader.read_u32()?;
        let count = reader.read_u32()?;

        Ok(Self {
            index,
            count,
        })
    }
}

impl BlobLink {
    #[inline]
    pub fn read<R: Read>(reader: &mut R) -> Result<Self, ParseError> {
        let offset = reader.read_u32()?;
        let flags = reader.read_u16()?;
        let dat_index = reader.read_u16()?;
        reader.padding(8)?;

        Ok(Self {
            offset,
            flags,
            dat_index,
        })
    }
}

impl DescriptorBucket {
    #[inline]
    pub fn read<R: Read>(reader: &mut R) -> Result<Self, ParseError> {
        let index = reader.read_u32()?;
        let count = reader.read_u32()?;

        Ok(Self {
            index,
            count,
        })
    }
}

impl DescriptorLink {
    #[inline]
    pub fn read<R: Read>(reader: &mut R) -> Result<Self, ParseError> {
        let offset = reader.read_u32()?;
        let size = reader.read_u32()?;

        Ok(Self {
            offset,
            size,
        })
    }
}

impl PreloadBucket {
    #[inline]
    pub fn read<R: Read>(reader: &mut R) -> Result<Self, ParseError> {
        let index = reader.read_u32()?;
        let count = reader.read_u32()?;

        Ok(Self {
            index,
            count,
        })
    }
}

impl PreloadGuid {
    #[inline]
    pub fn read<R: Read>(reader: &mut R) -> Result<Self, ParseError> {
        let hash = reader.read_u32()?;

        Ok(Self {
            hash,
        })
    }
}

impl PreloadLink {
    #[inline]
    pub fn read<R: Read>(reader: &mut R) -> Result<Self, ParseError> {
        let type_hash_2 = reader.read_u32()?;
        let descriptor_index = reader.read_u32()?;
        let unk0 = reader.read_u32()?;
        
        Ok(Self {
            type_hash_2,
            descriptor_index,
            unk0,
        })
    }
}
