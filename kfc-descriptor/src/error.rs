use thiserror::Error;

#[derive(Debug, Error)]
pub enum ReadError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("FromUtf8 error: {0}")]
    FromUtf8(#[from] std::string::FromUtf8Error),

    #[error("Invalid type hash: {0}")]
    InvalidTypeHash(u32),
}

#[derive(Debug, Error)]
pub enum WriteError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

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
    #[error("Malformed descriptor GUID: {0}")]
    MalformedDescriptorGuid(String),
}
