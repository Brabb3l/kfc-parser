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
