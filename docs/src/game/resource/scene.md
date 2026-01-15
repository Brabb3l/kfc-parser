# Scene

A Scene is a container for all the game objects, lights, and other elements that make up an environment in the game.

Each scene is composed of multiple different objects with the **same** GUID (this list may not be complete):
- `keen::SceneResource`
- `keen::FogVoxelMappingResource`
- `keen::RenderModelChunkGridResource`
- `keen::RenderModelChunkModelResource`
- `keen::SceneCinematicList`
- `keen::SceneEntityChunkResource`
- `keen::SceneRandomLootResource`
- `keen::VolumetricFog3ModelResource`
- `keen::VoxelTemperatureResource`
- `keen::VoxelWorldChunkResource`
- `keen::VoxelWorldFog3Resource`
- `keen::VoxelWorldResource`
- `keen::WaterChunkResource`

### `keen::SceneResource`

This object contains mostly static information about the scene, such as models, lights, bounds, and other elements.
Check the type definition for more details.

### Chunks

**Note:** This section is incomplete and may contain inaccurate information.

The scene contains several chunked resources that describe the terrain/fog voxels, water or entities.
There are 4 chunked resources in total:

- `keen::VoxelWorldChunkResource`: Contains the terrain and fog voxel data.
- `keen::RenderModelChunkModelResource`: Most likely contains the low-detail model for the terrain.
- `keen::WaterChunkResource`: Contains water data.
- `keen::SceneEntityChunkResource`: Contains entities placed in the scene.

#### Voxels

Each chunk of voxels is stored in a `keen::VoxelWorldChunkResource` object.
And for both the terrain and fog voxels, there is a separate `keen::VoxelWorldResource` object that contains:
- `type` (terrain or fog): Indicates whether the voxel data represents terrain or fog.
- `tileCount` (x, z): The number of `keen::VoxelWorldChunkResource` chunks in the x and z dimensions.
  There is no y dimension, as the height seems to be determined by the voxel data itself.
- `origin` (x, y, z): The position of the chunk grid's origin in world space.
- `lowLODData`: Most likely a lower-resolution representation of the voxel data for rendering at a distance.
- `materialGuids` (max 256): Most likely a list of material guids used for the voxels.

There are other fields, but their purpose is currently unknown. Check the type definition for more details.

To access the respective `keen::VoxelWorldChunkResource` objects, you will need to know the part indices of the `keen::VoxelWorldChunkResource` you want to access.
All `keen::VoxelWorldChunkResource` objects in the scene will have the same GUID,
where the part index corresponds to `flattened chunk coordinates + amount of chunks of previous keen::VoxelWorldResources`.

For example, if you have a terrain and fog voxel world with 2x2 chunks,
the part indices for the terrain chunks will be 0, 1, 2, and 3,
while the part indices for the fog chunks will be 4, 5, 6, and 7.

Now, to access the voxel data of a chunk, you will need to access the content that is referenced by the `highLODData` field in the `keen::VoxelWorldChunkResource`.
The binary format of the voxel data is currently unknown, but zeroing out the data will result in an empty chunk.

#### Water

The water data is stored in `keen::WaterChunkResource` objects.

#### Entities

The entities placed in the scene are stored in `keen::SceneEntityChunkResource` objects.
Each chunk contains a list of template references, models and entity spawns.
The amount of chunks is determined by the `entityChunkCount` (x, z) field in the `keen::SceneResource`.
