# kfc

This is a lightweight crate that re-exports the core functionality of:
- **`kfc-base`**: reading/writing `.kfc` files, type reflection, GUIDs, and hashing.
- **`kfc-descriptor`**: converting raw descriptor data into more usable formats such as json.
- **`kfc-blob`**: converting raw blob data into more usable formats such as images, audio and meshes.

### Features

- **`descriptor`** *(default)*: Enables the `kfc-descriptor` crate under the `kfc::descriptor` module.
- **`blob`**: Enables the `kfc-blob` crate under the `kfc::blob` module.
