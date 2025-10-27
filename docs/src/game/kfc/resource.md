# Resource Format Specification

This document describes the format of **resources** stored inside the `.kfc` file.

Resources are typed binary objects. The concrete structure of types (fields, type hashes, field offsets, alignments, etc.) is defined in the [types section](./types.md).

## General Notes

- Everything is little-endian unless otherwise noted.
- Make sure to zero out any padding bytes when serializing.

## Primitive Types

| Name | Ordinal | Description |
| - | - | - |
| `none` | `0x00` | No data |
| `bool` | `0x01` | 1 byte, values: `0x00` (false), `0x01` (true) |
| `uint8` | `0x02` | 1 byte unsigned integer |
| `sint8` | `0x03` | 1 byte signed integer |
| `uint16` | `0x04` | 2 byte unsigned integer |
| `sint16` | `0x05` | 2 byte signed integer |
| `uint32` | `0x06` | 4 byte unsigned integer |
| `sint32` | `0x07` | 4 byte signed integer |
| `uint64` | `0x08` | 8 byte unsigned integer |
| `sint64` | `0x09` | 8 byte signed integer |
| `float32` | `0x0A` | 4 byte IEEE 754 floating point number |
| `float64` | `0x0B` | 8 byte IEEE 754 floating point number |
| `enum` | `0x0C` | stored using the enum's `inner_type` (see [Enum](#enum)) |
| `bitmask8` | `0x0D` | 1 byte bitmask (up to 8 flags) |
| `bitmask16` | `0x0E` | 2 byte bitmask (up to 16 flags) |
| `bitmask32` | `0x0F` | 4 byte bitmask (up to 32 flags) |
| `bitmask64` | `0x10` | 8 byte bitmask (up to 64 flags) |
| `typedef` | `0x11` | stored using the typedef's `inner_type` (see [Typedef](#typedef)) |
| `struct` | `0x12` | see [Struct](#struct) |
| `static_array` | `0x13` | a fixed-size array (see [Static Array](#static-array)) |
| `ds_array` | `0x14` | unknown/not used |
| `ds_string` | `0x15` | unknown/not used |
| `ds_optional` | `0x16` | unknown/not used |
| `ds_variant` | `0x17` | unknown/not used |
| `blob_array` | `0x18` | a variable-size array (see [Blob Array](#blob-array)) |
| `blob_string` | `0x19` | a variable-size string (see [Blob String](#blob-string)) |
| `blob_optional` | `0x1A` | an optional value (see [Blob Optional](#blob-optional)) |
| `blob_variant` | `0x1B` | a variant of the base type (see [Blob Variant](#blob-variant)) |
| `object_reference` | `0x1C` | 16 byte GUID referencing another resource |
| `guid` | `0x1D` | 16 byte ContentHash |

### Enum

- An `enum` is stored using its `inner_type` (in the type metadata).
- The `inner_type` is always a primitive integer type (`uint8`, `sint8`, `uint16`, `sint16`, `uint32`, `sint32`, `uint64`, `sint64`).

### Struct

- A `struct` is a composite type consisting of multiple fields which is essentially a concatenation of its fields' serialized bytes.
- If a struct inherits from a base struct (namely, has a `inner_type`), the base struct's fields are serialized **first** (parents recursively up the chain).
- Each field is serialized **without its keys**, just its value.

**Note:** Each field has a `field_offset` in the type metadata which can be used to locate the field's value inside the struct instead of recomputing padding/alignment yourself.

### Typedef

- A `typedef` is an alias for another type (the `inner_type`).
- It can be resolved by simply serializing the `inner_type` recursively until a non-typedef type is reached.

### Static Array

- A `static_array` is a fixed-size array of elements of the same type.
- The number of elements is `field_count` (in the type metadata).
- The element type is `inner_type` (in the type metadata).
- Elements are stored contiguously, directly inline.

### Blob Array

- Out-of-line, data is stored as a blob.
- Layout:
  - 4 byte `uint32` relative offset (0 if empty)
  - 4 byte `uint32` count (0 if empty)
- The element type is `inner_type` (in the type metadata).
- Elements are stored contiguously, at the given blob offset.
- **IMPORTANT:** blob rules apply, see [Blob Rules](#blob-rules).

### Blob String

- Out-of-line, data is stored as a blob (non-null-terminated).
- Layout:
  - 4 byte `uint32` relative offset (0 if empty)
  - 4 byte `uint32` length in bytes (0 if empty)
- Characters are stored as bytes at the given blob offset.
- **IMPORTANT:** blob rules apply, see [Blob Rules](#blob-rules).

### Blob Optional

- Out-of-line, data is stored as a blob.
- Layout:
  - 4 byte `uint32` relative offset (0 if null)
- The inner type is `inner_type` (in the type metadata).
- The value is stored at the given blob offset if not null.
- **IMPORTANT:** blob rules apply, see [Blob Rules](#blob-rules).

### Blob Variant

- Out-of-line, data is stored as a blob.
- Layout:
  - 4 byte `uint32` qualified type hash of the stored variant (0 if no variant is specified)
  - 4 byte `uint32` relative offset (0 if no variant is specified)
  - 4 byte `uint32` blob size in bytes (0 if no variant is specified)
- The base type is `inner_type` (in the type metadata).
- **IMPORTANT:** blob rules apply, see [Blob Rules](#blob-rules).

## Blob Rules

Blob types (`blob_array`, `blob_string`, `blob_optional`, `blob_variant`) are placed **out-of-line** after the fixed-size base struct.
They need to be properly spaced and aligned according to their type metadata.

Because of this, when **serializing** a blob value you must manage this process yourself.
Feel free to implement this in a way that makes sense for you, but here is how the game seems to do it:

1. Set a `blob_offset` to the size of the base struct.
2. Serialize everything **in order**. For each blob field, do the following:
    - **(BlobVariant only)** Write the qualified type hash.
    - **Align `blob_offset`** to the blob data's alignment specified by the type metadata for the blob's data.
    - Compute `relative_offset = blob_offset - stream.position`, where:
        - `stream.position` is the absolute position of the `relative_offset` field itself.
        (i.e. the position where the `relative_offset` will be written)
    - Write the `relative_offset`.
    - Write the `count`/`length`/`size` field if applicable. (`blob_array`, `blob_string`, `blob_variant`)
    - Then write the **blob data** at the current `blob_offset`.
    - After writing the blob data, increment `blob_offset` by the size of the written blob data.
    - And finally **align `blob_offset` again** to the blob data's alignment.
3. Continue with the next field until all fields are serialized.
