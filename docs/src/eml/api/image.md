# Image

The `image` module provides functions for encoding and decoding image data in various formats.
It supports common image formats such as PNG and JPEG, but also gpu image formats like R8G8B8A8_UNORM, BCn, etc.

## Decoding Images

There are two separate functions to decode image data for different use cases:
- `image.decode`: Decodes regular image data such as PNG or JPEG into an [`Image`](#image-object) object.
- `image.decode_texture`: Decodes image data in a GPU format into an [`Image`](#image-object) object.

Both functions take a [`Buffer`](./buffer.md) object containing the binary image data as input and return an [`Image`](#image-object) object representing the decoded image.
However `decode_texture` also requires the width, height, format and mip level of the image to decode it properly.

```lua
local png_buffer = io.read("image.png")
local png_image = image.decode(png_buffer)

local texture_buffer = io.read("texture.dat")
local texture_image = image.decode_texture(texture_buffer, 256, 256, "R8G8B8A8_UNORM")
```

## Encoding Images

Similar to decoding, there are two separate functions to encode image data for different use cases:
- `image.encode`: Encodes an [`Image`](#image-object) object into regular image formats such as PNG or JPEG.
- `image.encode_texture`: Encodes an [`Image`](#image-object) object into a GPU image format.

Both functions take an [`Image`](#image-object) object as input and return a [`Buffer`](./buffer.md) object containing the encoded binary image data.

**Note:** Encoding to GPU formats may be a slow process depending on the format and size of the image. Especially for block-compressed formats, it may take several seconds to encode a single image.

```lua
local image = ... -- Assume this is an Image object
local png_buffer = image.encode(image, "PNG")

io.export("output.png", png_buffer) -- Export the PNG image

local texture_buffer = image.encode_texture(image, "R8G8B8A8_UNORM")
```

## Converting Images

With encoding and decoding functions available, you can easily convert images between different formats.

For example, to extract a texture from a resource and convert it to a PNG image, you can do the following:

```lua
local texture_resource = ... -- Assume this is a keen::UiTextureResource
local content = game.assets.get_content(texture_resource.data)

-- Decode the texture data into an Image object
local texture_image = image.decode_texture(
    content:read_data(),
    texture_resource.width,
    texture_resource.height,
    texture_resource.format
)

-- Encode the Image object into a PNG image
local png_buffer = image.encode(texture_image, "PNG")

io.export("texture.png", png_buffer) -- Export as PNG
```

This also works the other way around, so you can convert a PNG image into a GPU texture format:

```lua
local png_buffer = io.read("input.png")
local image = image.decode(png_buffer)
local texture_buffer = image.encode_texture(image, "R8G8B8A8_UNORM")

local texture_resource = ... -- Assume this is a keen::UiTextureResource

texture_resource.width = image.width
texture_resource.height = image.height
texture_resource.format = "R8G8B8A8_UNORM"
texture_resource.data = game.assets.create_content(texture_buffer)
```

## Creating Images

You can also create new [`Image`](#image-object) objects from scratch by using the `image.create` function.
It takes the width and height of the image and returns a new empty [`Image`](#image-object) object where all pixels are initialized to transparent black.

```lua
local img = image.create(128, 128)  -- Create a new 128x128 image
```

## Image Object

An `Image` object represents a 2D image with pixel data stored in RGBA format.
It provides some basic methods to manipulate the image data, such as getting and setting pixel colors.

Check the `Image` definition in the `base.lua` definition file for a complete list of available methods and properties.
