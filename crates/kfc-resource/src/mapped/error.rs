use kfc::reflection::TypeIndex;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MappingError {
    #[error("utf-8 conversion error: {0}")]
    Utf8(#[from] std::str::Utf8Error),

    #[error("unexpected end of data")]
    UnexpectedEndOfData,
    #[error("invalid type index: {0}")]
    InvalidTypeIndex(TypeIndex),
    #[error("invalid type hash: {0}")]
    InvalidTypeHash(u32),
    #[error("{0}")]
    UnsupportedOperation(&'static str),
}
