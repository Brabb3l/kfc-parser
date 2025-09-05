use thiserror::Error;

pub type Result<T> = std::result::Result<T, BufferError>;

#[derive(Debug, Clone, Error)]
pub enum BufferError {
    #[error("buffer overflow: position {position} exceeds limit {limit}")]
    Overflow { position: usize, limit: usize },
    #[error("buffer overflow: limit {limit} exceeds capacity {capacity}")]
    LimitOverflow { limit: usize, capacity: usize },
    #[error("capacity exceeds maximum possible size")]
    CapacityOverflow,

    #[error("buffer is not readable")]
    NotReadable,
    #[error("buffer is not writable")]
    NotWritable,
    #[error("buffer is closed")]
    Closed,
}

impl From<BufferError> for mlua::Error {
    fn from(val: BufferError) -> Self {
        Self::external(val)
    }
}
