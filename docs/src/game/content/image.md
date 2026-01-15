# Image

The game primarily uses gpu-ready packed/compressed texture formats for images.

A list of all supported formats can be found in the `keen::PixelFormat` enum in the type definitions.
All formats follow [Vulkan's Format Spec](https://docs.vulkan.org/spec/latest/chapters/formats.html)

The most commonly used formats are:
- `R8G8B8A8_UNORM`: 32-bit RGBA format for color textures with alpha (e.g. sprites, UI elements)
- `BC7_SRGB_BLOCK`: Compressed format for color textures with optional alpha (e.g. albedo/diffuse maps)
- `BC5_UNORM_BLOCK`: Compressed format for normal maps
- `BC1_RGB_UNORM_BLOCK`: Compressed format for material parameters (e.g. roughness, metallic, ambient occlusion)
- `BC4_UNORM_BLOCK`: Compressed format for single-channel textures (e.g. masks)
- `BC6H_UFLOAT_BLOCK`: Compressed format for HDR textures (e.g. emissive maps)
