use thiserror::Error;

pub type Result<T> = std::result::Result<T, BufferError>;

#[derive(Debug, Clone, Error)]
pub enum BufferError {
    #[error("buffer overflow: head {head} exceeds tail {tail}")]
    Overflow { head: usize, tail: usize },
    #[error("buffer overflow: tail {tail} exceeds capacity {capacity}")]
    TailOverflow { tail: usize, capacity: usize },
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
