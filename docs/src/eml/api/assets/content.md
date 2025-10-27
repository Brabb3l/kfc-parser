# Content

These are blobs that contain arbitrary binary data like images, audio, models, voxels, etc.
They are always referenced by resources via a field with the `keen::ContentHash` type.
Content assets are not parsed by EML, but instead provided as raw binary data as a `Buffer` object.
There are also some helper modules like the `image` module to work with specific content types.

## Accessing Content

To access content assets, you first need a `keen::ContentHash` value that references the content asset you want to access.
You can find these values in resource data structures.
But you can also just use the guid of a content asset if you know it.

Now, to actually get the content asset, you can use the `game.assets.get_content` function like this:

```lua
local content_hash = some_resource.data.textureHash  -- Assume this is a keen::ContentHash value
local content_asset = game.assets.get_content(content_hash)

print("Guid: " .. content_asset.guid)
print("Size: " .. content_asset.size)
print("Data: " .. tostring(content_asset:read_data()))
```

This function returns a [`Content`]() object that contains some metadata about the content asset (like its guid and size) and a method to read the actual binary data as a read-only `Buffer` object.

**Note:** The `Content` object can be assigned everywhere where a `Guid` or `keen::ContentHash` is expected, so you can pass it directly to functions and fields that expect these types without having to extract the guid manually.

## Content Object

A content object contains some metadata about the content asset such as its guid and size but also a method to read the actual binary data.
Calling the `read_data` method will return a read-only `Buffer` object that contains the binary data of the content asset.

In comparison to resources, content assets are immutable since content hashes are unique, so the data for a given content hash is always the same and never changes.
Because of that, you will have to instead create new content assets and update the old references.

## Creating Content

To create new content assets, you can use the `game.assets.create_content` function.
This function takes a `Buffer` object containing the binary data of the content asset and returns a new `Content` object that can be assigned to resource fields.

```lua
local buf = ... -- Assume this is a Buffer object containing the binary data of the content asset
local content_asset = game.assets.create_content(buf)

print("Guid: " .. content_asset.guid)
print("Size: " .. content_asset.size)
```

## Binary Format

Every content asset is stored differently depending on the context where it is used.
For example, image content are referenced in `keen::UiTextureResource` which has information about the format, size, etc. of the image data.
Without it, its very hard to interpret the raw binary data of a content asset.

Currently, EML does only provide the `image` module as a helper to work with image content assets.
That is, because most content formats have not been figured out just yet.
So you will have to reverse engineer the binary formats on your own if you want to work with other content types.
When doing so, i highly suggest exporting the binary data with the `export` capability and inspecting it with external tools such as [ImHex](https://github.com/WerWolv/ImHex) for binary analysis.
Be aware that this requires some knowledge about binary formats and reverse engineering, so it may not be suitable for everyone.
