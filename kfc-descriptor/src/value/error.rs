use std::fmt::Display;

use kfc::reflection::TypeIndex;
use thiserror::Error;

#[derive(Debug)]
pub struct WriteError {
    path: String,
    error: WriteErrorInfo,
}

impl WriteError {
    #[inline]
    pub(super) fn new(path: String, error: WriteErrorInfo) -> Self {
        Self { path, error }
    }

    #[inline]
    pub fn path(&self) -> &str {
        &self.path
    }

    #[inline]
    pub fn error(&self) -> &WriteErrorInfo {
        &self.error
    }
}

impl Display for WriteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "At {}: {}", self.path, self.error)
    }
}

impl std::error::Error for WriteError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.error)
    }
}

#[derive(Debug, Error)]
pub enum WriteErrorInfo {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Incompatible type: got {got}, expected {expected}")]
    IncompatibleType { got: String, expected: String },

    #[error("Invalid enum value: got {got}, expected one of {expected:?}")]
    InvalidEnumValue { got: String, expected: Vec<String> },

    #[error("Invalid field: {0}")]
    MissingField(String),

    #[error("Invalid type: {0}")]
    InvalidType(TypeIndex),
    #[error("Invalid type name: {0}")]
    InvalidTypeName(String),
    #[error("Variant type {0} is not a sub-type of {1}")]
    VariantTypeNotSubType(String, String),

    #[error("Malformed GUID: {0}")]
    MalformedGuid(String),
}
