---@meta

--- Provides utility functions for working with GUIDs.
---
--- @class GuidHelper
--- @field NONE Guid
local GuidHelper = {}

--- Creates a GUID from the given ContentHash.
--- @param content_hash keen.ContentHash
--- @return Guid
function GuidHelper.from_content_hash(content_hash) end

--- Creates a ContentHash from the given GUID.
---
--- @param guid Guid
--- @return keen.ContentHash
function GuidHelper.to_content_hash(guid) end

--- Hashes the given GUID to produce a Hash32 (fnv1a32) value.
---
--- @param guid Guid
--- @return u32
function GuidHelper.hash(guid) end
