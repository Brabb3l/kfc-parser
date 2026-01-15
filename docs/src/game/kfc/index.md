# Game File Documentation

The game consists of:
- **One** `.kfc` file
- **One** `.kfc_resources` file
- **Multiple** `.dat` files (containers). The number of `.dat` files is always a **power of two** and they always need to be present even if empty.

There are two different kinds of assets:

- **Resource**: typed binary data (type metadata defined in the [types section](./types.md) and bundled inside the executable).
  Resources can be referenced by `ObjectReference<T>` typed fields.
  They are stored in the `.kfc_resources` file (after the header) and are **16-byte aligned**.
- **Content**: opaque blobs (images, audio, models, voxels, etc.).
  Content assets are referenced by resources via a `keen::ContentHash` typed field.
  They are stored in the `.dat` files and are **4096-byte aligned**.
