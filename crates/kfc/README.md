# kfc

This is a lightweight crate that re-exports the core functionality of:
- **`kfc-base`**: reading/writing `.kfc` files, type reflection, GUIDs, and hashing.
- **`kfc-resource`**: converting raw resource data into more usable formats such as json.
- **`kfc-content`**: converting raw content data into more usable formats such as images, audio and meshes.

### Features

- **`resource`** *(default)*: Enables the `kfc-resource` crate under the `kfc::resource` module.
- **`content`**: Enables the `kfc-content` crate under the `kfc::content` module.
