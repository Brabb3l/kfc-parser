use thiserror::Error;
use crate::types::PrimitiveType;

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("UTF-8 error: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),
    #[error("ParseInt error: {0}")]
    ParseInt(#[from] std::num::ParseIntError),
    #[error("CRPF error: {0}")]
    Crpf(#[from] CrpfError),

    #[error("Invalid magic number: {0:X}")]
    InvalidMagic(u32),
    #[error("Invalid entry count: {0}")]
    InvalidEntryCount(u32),
    #[error("Invalid index entry: {0}")]
    InvalidIndexEntry(String),
}

#[derive(Debug, Error)]
pub enum CrpfError {
    #[error("Invalid magic number: {0:X}")]
    InvalidMagic(u32),
    #[error("Invalid kbf magic number: {0:X}")]
    InvalidKBFMagic(u32),
    #[error("Invalid CTCB magic number: {0:X}")]
    InvalidCTCBMagic(u32),
}

#[derive(Debug, Error)]
pub enum CrpfNodeParseError {
    #[error("Missing field: {0} in struct {1}")]
    MissingField(String, String),
    #[error("Field type mismatch: got {0:?}, expected {1:?} in struct {2}")]
    InvalidFieldType(PrimitiveType, PrimitiveType, String),
    #[error("Node type mismatch: got {0:?}, expected {1:?}")]
    TypeMismatch(PrimitiveType, &'static [PrimitiveType]),
    #[error("Invalid enum value: {0}")]
    InvalidEnumValue(String),
    
    // Specific to ContentHash
    #[error("Could not find content hash")]
    MissingContentHash, // TODO: make more verbose
    
    // Specific to SoundResource
    #[error("Invalid channel configuration: {0}")]
    InvalidChannelConfig(String),
}