use container::StaticHash;

pub mod container;
pub mod guid;
pub mod error;
pub mod reflection;
pub mod data;

/// Represent an fnv1a32 hash
type Hash32 = u32;

/// Represent a crc64 hash
type Hash64 = u64;

impl StaticHash for Hash32 {
    fn static_hash(&self) -> u32 {
        *self
    }
}
