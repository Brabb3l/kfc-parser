use thiserror::Error;

#[derive(Debug, Error)]
pub enum PEParseError {
    #[error("IO error: {0}")]
    IO(#[from] std::io::Error),
    
    #[error("Invalid DOS signature")]
    InvalidDosSignature,
    #[error("Invalid NT signature")]
    InvalidNTSignature,
    #[error("Unsupported PE type")]
    UnsupportedPEType,
    #[error("Malformed section name")]
    MalformedSectionName,
}

#[derive(Debug, Error)]
pub enum ReflectionParseError {
    #[error("IO error: {0}")]
    IO(#[from] std::io::Error),
    #[error("PE parse error: {0}")]
    PEParse(#[from] PEParseError),
    
    #[error("Missing .data section")]
    MissingDataSection,
    #[error("Missing .rdata section")]
    MissingRDataSection,
    #[error("Malformed pattern")]
    MalformedPattern,
}

#[derive(Debug, Error)]
pub enum TypeParseError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Debug, Error)]
pub enum ReadError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("FromUtf8 error: {0}")]
    FromUtf8(#[from] std::string::FromUtf8Error),

    #[error("Unknown type: {0}")]
    UnknownType(u32),

    #[error("Invalid type hash: {0}")]
    InvalidTypeHash(u32),
}

#[derive(Debug, Error)]
pub enum WriteError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Unknown type: {0}")]
    UnknownType(u32),

    #[error("Incompatible type: got {got}, expected {expected}")]
    IncompatibleType {
        got: String,
        expected: String
    },
    #[error("Invalid enum value: got {got}, expected one of {expected:?}")]
    InvalidEnumValue {
        got: String,
        expected: Vec<String>
    },
    #[error("Invalid field: {0}")]
    MissingField(String),
    #[error("Missing field type annotation")]
    MissingFieldType,
    #[error("Missing field value annotation")]
    MissingFieldValue,
    #[error("Invalid type: {0}")]
    InvalidType(String),
    #[error("Malformed blob GUID: {0}")]
    MalformedBlobGuid(String),
}
