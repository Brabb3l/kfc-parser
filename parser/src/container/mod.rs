mod read;

const KFC_DIR_MAGIC: u32 = 0x3043464B;

#[derive(Debug, Clone)]
pub struct KFCDir {
    pub header: KFCHeader,
    pub entries: Vec<KFCDirEntry>,
}

#[derive(Debug, Clone)]
pub struct KFCHeader {
    pub magic: u32,
    pub entry_count: u32,
    pub reserved: u32,
    pub data_size: u64,
}

#[derive(Debug, Clone)]
pub struct KFCDirEntry {
    pub name_hash: u64,
    pub decompressed_size: u32,
    pub compressed_size: u32,
    pub index: u32,
    pub flags: u32,
    pub offset: u64,
}
