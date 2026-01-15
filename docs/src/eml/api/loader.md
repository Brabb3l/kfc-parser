# Loader

The `loader` module provides information about the current environment in which the mod is currently running in.

- `is_client`: A boolean value indicating whether the mod is running in a client environment.
- `is_server`: A boolean value indicating whether the mod is running in a server environment.
- `features`: See [Features](#features).
- `has_mod(mod_id)`: Checks if a mod with the specified ID is loaded.

## Features

The `features` table contains boolean flags indicating the availability of certain features in the current environment.

- `patch`: Indicates whether data patching is supported.
- `export`: Indicates whether data exporting is supported.
