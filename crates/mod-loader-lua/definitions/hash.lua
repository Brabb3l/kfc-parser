--- @meta

--- Provides functions to compute various hash values.
---
--- @class Hasher
hasher = {}

--- Computes a 32-bit FNV-1a hash of the given value.
---
--- @param value string|Buffer
--- @return u32
function hasher.fnv1a32(value) end

--- Computes the CRC32/ISO-HDLC checksum of the given value.
---
--- @param value string|Buffer
--- @return u32
function hasher.crc32(value) end

--- Computes the CRC64/ECMA-182 checksum of the given value.
---
--- @param value string|Buffer
--- @return u64
function hasher.crc64(value) end
