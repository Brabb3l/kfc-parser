--- @meta

--- TODO: implement `image` module

--- @alias ImageFormat
--- | "png"
--- | "jpg" -- same as jpeg
--- | "jpeg" -- same as jpg
--- | "gif"
--- | "webp"
--- | "pnm"
--- | "tiff"
--- | "tga"
--- | "dds"
--- | "bmp"
--- | "ico"
--- | "hdr"
--- | "openexr"
--- | "farbfeld"
--- | "avif"
--- | "qoi"
--- | "pcx"

--- Provides utility functions for working with images.
---
--- @class ImageHelper
image = {}

--- Creates a new image with the specified width and height.
--- The image is initialized with #00000000 (transparent black) pixels.
---
--- @param width u32 -- The width of the image in pixels.
--- @param height u32 -- The height of the image in pixels.
--- @return Image -- The newly created image.
function image.create(width, height) end

--- Decodes a buffer to an image using the specified `ImageFormat`.
---
--- If no format is provided, the function will attempt to guess the format based on the buffer content.
--- This incldues all images in `ImageFormat`, except TGA.
---
--- @param buffer Buffer -- The buffer containing the image data.
--- @param format ImageFormat? -- The format of the input buffer, otherwise it will be guessed.
--- @return Image -- The decoded image.
function image.decode(buffer, format) end

--- Decodes a buffer to an image using the specified `PixelFormat`.
---
--- @param buffer Buffer -- The buffer containing the texture data.
--- @param format keen.PixelFormat -- The format of the input buffer.
--- @param width u32 -- The width of the image in pixels.
--- @param height u32 -- The height of the image in pixels.
--- @param mipmap_level u32? -- The mipmap level to decode, defaults to 0.
--- @return Image -- The decoded image.
function image.decode_texture(buffer, format, width, height, mipmap_level) end

--- Encodes an image to a buffer in the specified format.
---
--- @param image Image -- The image to encode.
--- @param format ImageFormat? -- The format to encode the image to, defaults to "png".
--- @return Buffer -- The encoded image data as a buffer.
function image.encode(image, format) end

--- Encode an image to a buffer in the specified `PixelFormat`.
---
--- @param image Image -- The image to encode.
--- @param format keen.PixelFormat -- The pixel format to encode the image to.
--- @return Buffer -- The encoded image data as a buffer.
function image.encode_texture(image, format) end

--- @class Image
--- @field width u32 -- The width of the image in pixels.
--- @field height u32 -- The height of the image in pixels.
local Image = {}

--- Returns the pixel color at the specified coordinates or `nil` if out of bounds.
---
--- @param x u32 -- The x-coordinate of the pixel.
--- @param y u32 -- The y-coordinate of the pixel.
--- @return u8, u8, u8, u8 -- The RGBA components of the pixel color, each in the range 0-255.
function Image:get_pixel(x, y) end

--- Sets the pixel color at the specified coordinates.
---
--- ### Errors
--- - If the coordinates are out of bounds.
---
--- @param x u32 -- The x-coordinate of the pixel.
--- @param y u32 -- The y-coordinate of the pixel.
--- @param r u8 -- The red component of the pixel color, in the range 0-255.
--- @param g u8 -- The green component of the pixel color, in the range 0-255.
--- @param b u8 -- The blue component of the pixel color, in the range 0-255.
--- @param a u8 -- The alpha component of the pixel color, in the range 0-255.
function Image:set_pixel(x, y, r, g, b, a) end

--- Returns the pixel color at the specified coordinates as a packed 32 bit integer in RGBA format, or `nil` if out of bounds.
--- The integer is packed as follows: `0xRRGGBBAA`.
---
--- @param x u32 -- The x-coordinate of the pixel.
--- @param y u32 -- The y-coordinate of the pixel.
--- @return u32 -- The packed RGBA color of the pixel.
function Image:get_pixel_packed(x, y) end

--- Sets the pixel color at the specified coordinates using a packed 32 bit integer in RGBA format.
--- The integer is packed as follows: `0xRRGGBBAA`.
---
--- ### Errors
--- - If the coordinates are out of bounds.
---
--- @param x u32 -- The x-coordinate of the pixel.
--- @param y u32 -- The y-coordinate of the pixel.
--- @param color u32 -- The packed RGBA color of the pixel.
function Image:set_pixel_packed(x, y, color) end
