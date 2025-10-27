# Getting Started

Before you begin, make sure to setup your development environment as described [here](./setup.md).

This guide only covers the `patch` and `export` capabilities. The `runtime` capability is still a work in progress and cannot be used yet.

## Hello World

Now that you have your environment set up, you should see a `mod.lua` file in the `src` directory of your mod.
By default it contains a simple Hello World example like this:

```lua
print("Hello from the default mod!")
```

To run your mod refer to the [usage guide](../usage.md).
As soon as you run EML either via the CLI or the proxy DLL, you should see the message in the console output.  
To see a console window when using the proxy DLL, you need to enable it in the `eml.json` configuration file as described [here](../usage.md#emljson-configuration).

## Definition Files

As soon as you run EML for the first time, it will generate definition files for the Lua language server to provide autocompletion and type checking,
but they may also be used to just lookup available functions and types (especially for the game data structures).
When either the game or EML is updated, it will regenerate the definition files on the next run to ensure they are up to date.

These files are located in the `.cache/lua` directory of your game installation and there are currently two sets of definition files:
- `base.lua`: Contains definitions (including documentation) for the EML API.
- `types.lua`: Contains auto generated type definitions for game data structures.

If you generated a new mod using the CLI, it will also create a `.luarc.json` file in your mod directory to configure the Lua language server to use these definition files.
If you want to develop your mod in a different directory, make sure to adjust the paths in the `.luarc.json` file accordingly or else you won't get any autocompletion or type checking.

## EML API Structure

The EML API is divided into several globally accessable modules.

The most important ones are:
- `game`: The main module to access and modify game data and assets.
- `loader`: Used to access information about the mod loader and loaded mods.
  For example, you can check which mods are currently loaded or what capabilities are enabled.
- `buffer`: Provides a buffer type to work with binary data.
  Since content assets are stored in an abitrary binary format, this module is essential for reading and modifying them.
  There are also some modules (more coming soon) that do the parsing for you, like the `image` module for reading and writing images.

Helpful modules when working with game assets:
- `image`: A module for reading and writing images in various formats.
  It provides an image type that can be used to manipulate image data.
  You can convert game textures to PNG (or some other format) and vice versa.
- `integer`: Provides functions for integers of various sizes.
  There are a lot of game data structures that use fixed size integers (e.g., `u8`, `u16`, etc.) and this module provides functions to work with them more comfortably.
  For example, adding two `u8` values together while ensuring the result is still a `u8`.
  If you're unfamiliar with these types, see the [Type Aliases](#type-aliases) section below.
- `hasher`: Provides functions to compute hashes using various algorithms that are used by the game.
  This is mainly useful when working with game assets, as they are often identified by their hash values.
  There is also the `game.guid.hash` function which computes the hash of a GUID.

Other useful modules include:
- `io`: Provides functions for file system operations.
  This is especially useful when using the `export` capability to save data to the export directory.
  All operations are confined to either the `export` directory or the mod's own directory to prevent mods from accessing arbitrary files on the user's system.

For more detailed information about the available modules and their functions, refer to the [API Reference](../api/index.md).

### Type Aliases

If you have already looked at the definition files, you may have noticed that there are a lot of these `u8`, `i32`, `f32`, etc. types.
These are type aliases for fixed size integers and floating point numbers.

You should always make sure that the values match the boundaries of the specified type, or else it will raise an error.
For example, a `u8` can only hold values from `0` to `255`, so trying to assign a value of `300` to a `u8` variable will result in an error.

The naming convention for these integer types is as follows:
- `u`: Unsigned integer followed by the number of bits (e.g., `u8`, `u16`, `u32`, `u64`).
- `i`: Signed integer followed by the number of bits (e.g., `i8`, `i16`, `i32`, `i64`).
- `f`: Floating point number followed by the number of bits (e.g., `f16`, `f32`, `f64`).

## Accessing Game Data

To access game data, you will primarily use the `game.assets` module.
But before we dive into the details, let's first understand how game data is organized.

### Game Data

There are two distinct types of game data:

- **Resources**: These are rather small data files that contain typed binary data.
  They are referenced by a GUID, a qualified type name and a part index.
  The part index of a resource is used to distinguish between multiple resources with the same guid and type that belong together.
  The way how part indices are used is specific to the type of resource.
  For example, the voxel chunks of a scene are made up of multiple parts where all have the same type and guid, but have different part indices that specify their position.  

  They are provided as lua table-like userdata objects.
  The structure of these objects is defined in the `types.lua` definition file.
- **Content Assets**: These are blobs that contain arbitrary binary data like images, audio, models, voxels, etc.
  They are always referenced by resources via a field with the `keen::ContentHash` type.
  Content assets are not parsed by EML, but instead provided as raw binary data as a [`Buffer`](../api/buffer.md) object.
  There are also some helper modules like the `image` module to work with specific content types.

### Accessing Resources

To access resources, you can use one of the `game.assets.get_resource*` functions.

If you know the exact guid, type and part index of a resource, you can use the `game.assets.get_resource` function like this:

```lua
local game_scene = game.assets.get_resource("509feadb-4c60-425f-9c7c-deeefd9b6920", "keen::SceneResource", 0)

print("Guid: " .. game_scene.guid)
print("Type: " .. game_scene.type)
print("Part Index: " .. game_scene.part_index)
print("Data: " .. tostring(game_scene.data))
```

As you can see, it doesn't return the actual data directly, but instead a `Resource` object that contains some metadata about the resource (like its guid, type, part index, etc.) and the actual data is stored in the `data` field of the resource object.

**Note:** The `Resource` object can be assigned everywhere where a `Guid` or `ObjectReference` is expected, so you can pass it directly to functions and fields that expect these types without having to extract the guid manually.

But there are also cases where you don't know the exact guid or type of a resource or are just unsure if it changes between game versions.
In these cases, you can use the either `game.assets.get_resources_by_type` or `game.assets.get_all_resources` functions to get a list of resources that match certain criteria.

As an example, to get all resources of a certain type, you can use the `game.assets.get_resources_by_type` function like this:

```lua
local items = game.assets.get_resources_by_type("keen::ItemInfo")

for _, item in ipairs(items) do
    local item_data = item.data  -- Access the actual data of the resource

    print(item_data.itemId.value, item_data.debugName)
end
```

And if you don't know what types of resources are available, you can use the `game.assets.get_resource_types` function to get a list of all available resource types.
It is primarily useful for exploration and debugging purposes.

```lua
local resource_types = game.assets.get_resource_types()

for _, resource_type in ipairs(resource_types) do
    print(resource_type)
end
```

As a last resort, there is also the `game.assets.get_all_resources` function that returns all resources in the game.
It is not recommended to use this function to filter resources by type or guid, as it is less efficient than using the other functions.

### Modifying Resources

To modify game data, you can simply change the fields of the resource's `data` object or the `data` field itself to overwrite the entire resource.
Since resources are provided as lua table-like userdata objects, you can access and modify their fields just like you would with a regular lua table.
Modifications to resources are checked for validity, so make sure to only assign valid values to fields.
If you try to assign an invalid value, it will raise an error.

**Important:** If you assign a reference value (i.e., a table or a userdata) to a field, it will create a deep copy of the value.
So subsequent modifications to the original value will **not** affect the resource's data.

## Creating a Simple Patch

Now that you have a basic understanding of how to set up your environment and run your mod, let's create a simple patch that modifies some game data.
Open the `mod.lua` file in your mod's `src` directory and replace the existing code with the following:

```lua
---@type keen.BalancingTable
local BalancingTable = game.assets.get_resources_by_type("keen::BalancingTable")[1].data

BalancingTable.playerBaseStamina = 500
```

The first line (the `---@type` annotation) is a type hint that tells the Lua language server what type the `BalancingTable` variable has.
This is not strictly necessary, but it helps with autocompletion and type checking.

The second line retrieves the first resource of type `keen::BalancingTable` and accesses its `data` field to get the actual balancing table data.

And in the last line, we modify the `playerBaseStamina` field to set the player's base stamina to `500`.

## Exporting Data

To export data from the game, you'll have to use the `export` capability.
Make sure to enable it in your `eml.json` configuration file if you're using the proxy DLL.

Now, let's modify our `mod.lua` file to export the string representation of the balancing table to a file.

```lua
---@type keen.BalancingTable
local BalancingTable = game.assets.get_resources_by_type("keen::BalancingTable")[1].data

io.export("balancing_table.txt", tostring(BalancingTable))
```

When you run your mod now, it will create a file named `balancing_table.txt` in the `export` directory of your game installation (unless you changed it).
If you specify a subdirectory in the file name, it will create the necessary directories automatically.

You can also export a [`Buffer`](../api/buffer.md) object directly to export its binary data.

**Important:** The `export` function can only write files to the `export` directory. Files that already exist will be overwritten without warning!

## Next Steps

Now that you have created a simple patch, you start looking into all the resource types that are available.
But most things have not been documented yet, so you'll need to figure stuff out by yourself what certain things do.
The best way is to just try things out and see what happens.

When it comes to content assets, i highly suggest exporting the binary data with the `export` capability and inspecting it with external tools such as [ImHex](https://github.com/WerWolv/ImHex) for binary analysis.
Be aware that this requires some knowledge about binary formats and reverse engineering, so it may not be suitable for everyone.

Check out the **References** section for stuff that has already been documented.

Everything that has not been covered in this guide yet is documented in the [API Reference](../api/index.md).
