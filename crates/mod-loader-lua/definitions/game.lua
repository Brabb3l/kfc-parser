---@meta

--- This is the primary object for accessing or modifying anything in the game.
---
--- @class Game
---
--- @field version string -- Represents the version of this game which does not follow any specific format and is purely extracted from the game's kfc file.
--- @field assets AssetManager -- A reference to the mod loader's asset manager.
--- @field guid GuidHelper -- A helper for creating GUIDs from content hashes.
--- @field types TypeRegistry
game = {}
