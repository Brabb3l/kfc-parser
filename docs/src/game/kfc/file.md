# KFC File Format Specification

Everything is little-endian unless otherwise noted.

The format of resources is defined in the [resource section](./resource.md).

## Header

| Name                    | Type        | Size (bytes) | Description                                                                                                                               |
| ------------            | ----------- | ------------ | ------------------------------------------------------------------------------------------------------------------------------------------|
| magic                   | uint32      |            4 | File signature: `4B 46 43 32` ("KFC2")                                                                                                    |
| size                    | uint32      |            4 | Size in bytes of the header area.                                                                                                         |
| unknown                 | uint32      |            4 | Always 12                                                                                                                                 |
| padding                 | uint32      |            4 | Padding? Always 0                                                                                                                         |
| version                 | KFCLocation |            8 | Points to `uint8[count]` containing the version string.                                                                                   |
| containers              | KFCLocation |            8 | Points to [`ContainerInfo[count]`](#containerinfo) describing `.dat` files.                                                               |
| unused0                 | KFCLocation |            8 | Unused, always null location.                                                                                                             |
| unused1                 | KFCLocation |            8 | Unused, always null location.                                                                                                             |
| resource_locations      | KFCLocation |            8 | Points to [`ResourceLocation[count]`](#resourcelocation) describing where resources are stored within this file.                          |
| resource_indices        | KFCLocation |            8 | Points to `uint32[count]` mapping `ResourceBundleEntry::index` to an index in `resource_keys`. See [Resource Bundles](#resource-bundles). |
| content_buckets         | KFCLocation |            8 | Points to [`StaticMapBucket[count]`](#staticmapbucket) for content [static map](#static-map).                                             |
| content_keys            | KFCLocation |            8 | Points to [`ContentHash[count]`](#contenthash) for content [static map](#static-map).                                                     |
| content_values          | KFCLocation |            8 | Points to [`ContentEntry[count]`](#contententry) for content [static map](#static-map).                                                   |
| resource_buckets        | KFCLocation |            8 | Points to [`StaticMapBucket[count]`](#staticmapbucket) for resources [static map](#static-map).                                           |
| resource_keys           | KFCLocation |            8 | Points to [`ResourceId[count]`](#resourceid) for resources [static map](#static-map).                                                     |
| resource_values         | KFCLocation |            8 | Points to [`ResourceEntry[count]`](#resourceentry) for resources [static map](#static-map).                                               |
| resource_bundle_buckets | KFCLocation |            8 | Points to [`StaticMapBucket[count]`](#staticmapbucket) for resource bundles [static map](#static-map).                                    |
| resource_bundle_keys    | KFCLocation |            8 | Points to `uint32[count]` for resource bundles [static map](#static-map). (the internal hash of the resource type)                        |
| resource_bundle_values  | KFCLocation |            8 | Points to [`ResourceBundleEntry[count]`](#resourcebundleentry) for resource bundles [static map](#static-map).                            |

Addtional notes:
- `version` is a non-null-terminated ASCII string.
- `resource_indices.count` == `resource_keys.count` == `resource_values.count`

### KFCLocation

| Name            | Type   | Size (bytes) | Description                                                                                       |
| --------------- | ------ | ------------ | ------------------------------------------------------------------------------------------------- |
| relative_offset | uint32 |            4 | The amount of bytes between the offset of this field and the start of the data it is pointing to. |
| count           | uint32 |            4 | The number of entries of the type being pointed to.                                               |

To get the absolute file offset of the data, add `relative_offset` to the file offset of the `relative_offset` field itself.
For example, if the `relative_offset` field is at file offset 0x20 and its value is 0x100, the data starts at file offset 0x120.

### ContainerInfo

| Name   | Type   | Size (bytes) | Description                                       |
| ------ | ------ | ------------ | ------------------------------------------------- |
| size   | uint64 |            8 | Total size of the `.dat` container file in bytes. |
| count  | uint64 |            8 | Number of contents in this container.             |

While the size is a `uint64`, [ContentEntry](#contententry) uses a `uint32` for the offset and size of each content,
so no single content can be larger than 4 GiB.

### ResourceLocation

| Name         | Type   | Size (bytes) | Description                                                  |
| ------------ | ------ | ------------ | ------------------------------------------------------------ |
| offset       | uint32 |            4 | Offset to where the resources start in this file. (absolute) |
| size         | uint32 |            4 | Total size of all resources in bytes.                        |
| count        | uint32 |            4 | Number of resources.                                         |

There is currently always exactly one `ResourceLocation` entry.
It may work with multiple entries, but this has not been tested yet.

### StaticMapBucket

| Name     | Type   | Size (bytes) | Description                                            |
| -------- | ------ | ------------ | ------------------------------------------------------ |
| index    | uint32 |            4 | Start index into the map's key/value arrays.           |
| count    | uint32 |            4 | Number of entries in this bucket. (linear probe range) |

See [Static Map](#static-map) for details.

### ContentHash

A content hash is used to reference content assets within `.dat` files.

Here is how it is structured:

| Name  | Type   | Size (bytes) | Description               |
| ----- | ------ | ------------ | ------------------------- |
| size  | uint32 |            4 | The size of the content.  |
| hash0 | uint32 |            4 | First part of the hash.   |
| hash1 | uint32 |            4 | Second part of the hash.  |
| hash2 | uint32 |            4 | Third part of the hash.   |

The hash of the content is computed with a [custom algorithm](https://github.com/Brabb3l/kfc-parser/blob/a7f4e26f33644316664646cd4cb6f1f21a223eb7/crates/kfc-base/src/hash/content.rs) which produces a 128-bit hash.
The first 4 bytes of the result is then replaced with the size of the content.

The `size` field is used for determining the size of the content and there is no other way to get it.

Since content with the same data have the same `ContentHash`, you don't need to store the same content multiple times.

### ContentEntry

| Name                          | Type        | Size (bytes) | Description                                                        |
| ----------------------------- | ----------- | ------------ | ------------------------------------------------------------------ |
| offset                        | uint32      |            4 | Offset inside the referenced `.dat` file where the content starts. |
| flags                         | uint16      |            2 | Currently unused, always 0.                                        |
| container_index               | uint16      |            2 | Index into the containers array.                                   |
| padding                       | uint8\[8]   |            8 | Padding to make the struct 16 bytes long. Always 0.                |

Content is always aligned to **4096 bytes** inside the `.dat` files.
The size of the content is not stored here, but in the [ContentHash](#contenthash).

### ResourceId

A resource ID is used to reference resources within the `.kfc` file.
It is a completely unique identifier for each resource.
Here is how it is structured:

| Name       | Type      | Size (bytes) | Description                                                             |
| ---------- | --------- | ------------ | ----------------------------------------------------------------------- |
| hash       | uint32[4] |           16 | The hash of the resource.                                               |
| type_hash  | uint32    |            4 | The qualified hash of the resource type.                                |
| part_index | uint32    |            4 | The index of the part if there are multiple instances of this resource. |
| reserved0  | uint32    |            4 | Reserved, always 0.                                                     |
| reserved1  | uint32    |            4 | Reserved, always 0.                                                     |

The hash of the resource id is a randomly generated UUIDv4.

### ResourceEntry

| Name            | Type        | Size (bytes) | Description                                                    |
| --------------- | ----------- | ------------ | -------------------------------------------------------------- |
| offset          | uint32      |            4 | Offset inside the resource location where the resource starts. |
| size            | uint32      |            4 | Size of the resource in bytes.                                 |

The absolute offset of the resource can be calculated by adding the `offset` to the `offset` of the [ResourceLocation](#resourcelocation).

### ResourceBundleEntry

| Name          | Type        | Size (bytes) | Description                             |
| ------------- | ----------- | ------------ | --------------------------------------- |
| internal_hash | uint32      |            4 | The internal hash of the resource type. |
| index         | uint32      |            4 | Index into the `resource_keys` array.   |
| count         | uint32      |            4 | Number of resources of this type.       |

## Static Map

Static maps consists of three arrays: buckets, keys and values.
There are always `N` buckets, where `N` is a power of two, and the key and value arrays must always have the same length.

Here is how to look up a key in a static map in pseudocode:

```pseudo
hash = hash_function(key) % buckets.count
bucket = buckets[hash]

for i in 0 until bucket.count:
    idx = bucket.index + i

    if keys[idx] == key:
        return values[idx]

return not_found
```

### Hash Functions

| Key Type       | Hash Function                                                                                                                                       |
| -------------- | --------------------------------------------------------------------------------------------------------------------------------------------------- |
| ContentHash    | `hash0` field of the `ContentHash` (precomputed)                                                                                                    |
| ResourceId     | A seeded [FNV-1a32](#fnv-1a32) over a 64-bit word formed from `type_hash` (low 32 bits) and `part_index` (high 32 bits) with `hash[0]` as the seed. |
| uint32         | The uint32 value itself.                                                                                                                            |

**Note:** The 64-bit word `data = type_hash | (part_index << 32)` is serialized in little-endian byte order and then fed, byte-by-byte, into FNV-1a32.

### FNV-1a32

See [FNV hash on Wikipedia](https://en.wikipedia.org/wiki/Fowler%E2%80%93Noll%E2%80%93Vo_hash_function).

Here is the pseudocode for FNV-1a32 used by the game:

```pseudo
function fnv1a32(data: byte[]) -> uint32:
    return fnv1a32_with_seed(data, 0x811C9DC5)

function fnv1a32_with_seed(data: byte[], seed: uint32) -> uint32:
    hash = seed

    for byte in data:
        hash = hash XOR byte
        hash = hash * 0x01000193

    return hash
```

## Resource Bundles

Resource bundles are a collection of resources of the same type.
You can use them to efficiently look up all resources of a specific type.

Here is how to get all resources of a specific type in pseudocode:

```pseudo
bundle = resource_bundles.get(type_hash)
bundle_resources = []

for i in 0 until bundle.count:
    idx = bundle.index + i

    resource_index = resource_indices[idx]
    resource_id = resource_keys[resource_index]
    resource_entry = resource_values[resource_index]

    bundle_resources.append((resource_id, resource_entry))

return bundle_resources
```
