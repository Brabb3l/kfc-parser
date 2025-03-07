use thiserror::Error;

#[derive(Debug, Error)]
pub enum KFCWriteError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Size too large: {0}")]
    SizeTooLarge(u64),
}

#[derive(Debug, Error)]
pub enum KFCReadError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("StaticMap error: {0}")]
    StaticMap(#[from] StaticMapError),

    #[error("Invalid magic number: {0:X}")]
    InvalidMagic(u32),
}

#[derive(Debug, Error)]
pub enum StaticMapError {
    #[error("Keys and values must have the same length: {0} != {1}")]
    LengthMismatch(usize, usize),
    #[error("Bucket reference count does not match key count: {0} != {1}")]
    BucketCountMismatch(usize, usize),
    #[error("Bucket size must be a power of 2: {0}")]
    InvalidBucketSize(usize),
}
