use crate::guid::{BlobGuid, DescriptorGuid};

mod read;
mod write;
mod util;

const KFC_DIR_MAGIC: u32 = 0x3243464B;

#[derive(Debug, Clone)]
pub struct KFCFile {
    pub version: String,
    pub dat_infos: Vec<DatInfo>,
    
    pub descriptor_locations: Vec<DescriptorLocation>,
    pub descriptor_indices: Vec<u32>,
    
    pub blob_buckets: Vec<BlobBucket>,
    pub blob_guids: Vec<BlobGuid>,
    pub blob_links: Vec<BlobLink>,
    
    pub descriptor_buckets: Vec<DescriptorBucket>,
    pub descriptor_guids: Vec<DescriptorGuid>,
    pub descriptor_links: Vec<DescriptorLink>,
    
    pub preload_buckets: Vec<PreloadBucket>,
    pub preload_guids: Vec<PreloadGuid>,
    pub preload_links: Vec<PreloadLink>,
}

#[derive(Debug, Clone, Default)]
pub struct KFCHeader {
    pub magic: u32, // KFC_DIR_MAGIC
    pub size: u32,
    pub unk0: u32, // 12
    // padding[4]
    
    pub version: KFCLocation, // char[count]
    pub dat_infos: KFCLocation, // DatInfo[count]
    
    pub unused0: KFCLocation,
    pub unused1: KFCLocation,
    
    pub descriptor_locations: KFCLocation, // DescriptorLocation[count]
    pub descriptor_indices: KFCLocation, // u32[count]
    
    pub blob_buckets: KFCLocation, // BlobBucket[count]
    pub blob_guids: KFCLocation, // BlobGuid[count]
    pub blob_links: KFCLocation, // BlobLink[count]
    
    pub descriptor_buckets: KFCLocation, // DescriptorBucket[count]
    pub descriptor_guids: KFCLocation, // DescriptorGuid[count]
    pub descriptor_links: KFCLocation, // DescriptorLink[count]
    
    pub preload_buckets: KFCLocation, // PreloadBucket[count]
    pub preload_guids: KFCLocation, // PreloadGuid[count]
    pub preload_links: KFCLocation, // PreloadLink[count]
}

#[derive(Debug, Clone, Default)]
pub struct KFCLocation {
    pub offset: u64, // u32
    pub count: u32,
}

#[derive(Debug, Clone, Default)]
pub struct DatInfo {
    pub largest_chunk_size: u16,
    pub unk0: u32,
    // padding[2]
    pub count: u32,
    // padding[4]
}

#[derive(Debug, Clone, Default)]
pub struct DescriptorLocation {
    pub offset: u32,
    pub size: u32,
    pub count: u32,
}

#[derive(Debug, Clone, Default)]
pub struct BlobBucket {
    pub index: u32,
    pub count: u32,
}

#[derive(Debug, Clone, Default)]
pub struct BlobLink {
    pub offset: u32,
    pub flags: u16,
    pub dat_index: u16,
    // padding[8]
}

#[derive(Debug, Clone, Default)]
pub struct DescriptorBucket {
    pub index: u32,
    pub count: u32,
}

#[derive(Debug, Clone, Default)]
pub struct DescriptorLink {
    pub offset: u32,
    pub size: u32,
}

#[derive(Debug, Clone, Default)]
pub struct PreloadBucket {
    pub index: u32,
    pub count: u32,
}

#[derive(Debug, Clone, Default)]
pub struct PreloadGuid {
    pub hash: u32,
}

#[derive(Debug, Clone, Default)]
pub struct PreloadLink {
    pub type_hash_2: u32,
    pub descriptor_index: u32,
    pub unk0: u32,
}
