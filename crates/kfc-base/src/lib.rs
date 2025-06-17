use container::StaticHash;

pub mod container;
pub mod guid;
pub mod reflection;
pub mod hash;
pub mod io;

/// Represent an fnv1a32 hash
pub type Hash32 = u32;

/// Represent a crc64 hash
pub type Hash64 = u64;

impl StaticHash for Hash32 {
    fn static_hash(&self) -> u32 {
        *self
    }
}
