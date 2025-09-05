---@meta

--- TODO: add documentation
--- TODO: implement `Type::flags`
--- TODO: implement `Type::default_value`

---@class TypeRegistry
local TypeRegistry = {}

--- @param qualified_hash u32
--- @return Type
function TypeRegistry.get(qualified_hash) end

--- @param qualified_name string
--- @return Type
function TypeRegistry.get(qualified_name) end

--- @param qualified_hash u32
--- @return Type
function TypeRegistry.get_by_qualified_hash(qualified_hash) end

--- @param impact_hash u32
--- @return Type
function TypeRegistry.get_by_impact_hash(impact_hash) end

--- @param qualified_name string
--- @return Type
function TypeRegistry.get_by_qualified_name(qualified_name) end

--- @param impact_name string
--- @return Type
function TypeRegistry.get_by_impact_name(impact_name) end

--- @return Type[]
function TypeRegistry.get_all() end

--- @param value any
--- @return Type
function TypeRegistry.of(value) end

--- @class Type
---
--- @field name string
--- @field impact_name string
--- @field qualified_name string
---
--- @field name_hash u32
--- @field impact_hash u32
--- @field qualified_hash u32
--- @field internal_hash u32
---
--- @field namespace string[]
--- @field inner_type Type?
--- @field size u32
--- @field alignment u32
--- @field element_alignment u32
--- @field field_count u32
--- @field primitive_type PrimitiveType
--- @field flags TypeFlag[]
---
--- @field struct_fields table<string, StructField>
--- @field enum_fields table<string, EnumField>
--- @field attributes table<string, Attribute>
---
--- @field default_value unknown -- TODO: This needs a custom implementation and must be an ObjectValue
local Type = {}

--- @class StructField
--- @field name string
--- @field type Type
--- @field data_offset u32
--- @field attributes table<string, Attribute>

--- @class EnumField
--- @field name string
--- @field value u64

--- @class Attribute
--- @field name string
--- @field namespace string[]
--- @field type Type?
--- @field value string

---@alias PrimitiveType
---| "None"
---| "Bool"
---| "UInt8"
---| "SInt8"
---| "UInt16"
---| "SInt16"
---| "UInt32"
---| "SInt32"
---| "UInt64"
---| "SInt64"
---| "Float32"
---| "Float64"
---| "Enum"
---| "Bitmask8"
---| "Bitmask16"
---| "Bitmask32"
---| "Bitmask64"
---| "Typedef"
---| "Struct"
---| "StaticArray"
---| "DsArray"
---| "DsString"
---| "DsOptional"
---| "DsVariant"
---| "BlobArray"
---| "BlobString"
---| "BlobOptional"
---| "BlobVariant"
---| "ObjectReference"
---| "Guid"

---@alias TypeFlag
---| "None"
---| "IsDs"
---| "HasBlobArray"
---| "HasBlobString"
---| "HasBlobOptional"
---| "HasBlobVariant"
---| "IsGpuUniform"
---| "IsGpuStorage"
---| "IsGpuConstant"
