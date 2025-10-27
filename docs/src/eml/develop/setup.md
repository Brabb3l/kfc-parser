# Setup

This document provides instructions on how to set up a modding environment for
the game Enshrouded using the Enshrouded Mod Loader (EML).

## Prerequisites

- Ensure you have a legal copy of the game Enshrouded installed on your system.
- Download the latest version of the Enshrouded Mod Manager CLI from **WIP**
- A text editor or IDE of your choice, preferably with [lua_ls](https://luals.github.io/) (e.g., Visual Studio Code).
- Basic knowledge of Lua programming.

## Creating a Mod

The mod manager cli can be used to create a new mod template.

```bash
emm.exe create -g <game-dir>
```

You will be prompted to enter a few details about your mod which will be placed
within the `mod.json` [manifest file](#mod-manifest).

After the command completes, a new directory named after your mod id will be
created in the mods directory of your game installation.
It will also generate definition files for the Lua language server to provide
autocompletion and type checking. They are located in `<game-dir>/.cache/lua`.

### Mod Structure

```
<mod-id>/
├── mod.json
├── README.md
├── .luarc.json
└── src/
    └── mod.lua
```

- `mod.json`: The manifest file containing metadata about your mod.
- `README.md`: A markdown file where you can provide a detailed description of your mod.
- `.luarc.json`: Configuration file for the Lua language server.
  If you don't use lua_ls or use another language server, you can safely delete this file.
- `src/mod.lua`: The main Lua script file where you will write your mod's code

### Mod Manifest

The `mod.json` file contains metadata about your mod and is essential for EML to
recognize and load it. Below is a detailed explanation of each field in the manifest:

- `id` (string, required): A **unique** identifier for your mod. The id may only
  contain lowercase letters, numbers, hyphens (`-`), and underscores (`_`).
  It must start with a letter.
- `name` (string, required): The display name of your mod.
  This is what users will see in the mod manager.
- `version` (string, required): The version of your mod, following [semantic versioning](https://semver.org/).
- `capabilities` (string list, required): A list of capabilities that your mod requires.
  Valid capabilities include:
  - `patch` (lua): Allows the mod to patch game data.
  - `export` (lua): Allows the mod to export arbitrary data into an export directory.
  - `runtime` (WIP): Allows the mod to run code at runtime.

- `description` (string, optional): A brief description of what your mod does.
  A detailed description can be provided in a separate `README.md` file.
- `authors` (string list, optional): A list of authors who contributed to the mod.
- `license` (string, optional): The license under which your mod is released (e.g., MIT, GPL-3.0).
  You may also include a `LICENSE` file in your mod directory.
- `icon` (string, optional): Path to an icon file (e.g., `icon.png`) that represents your mod.
  This icon will be displayed in the mod manager.
- `dependencies` (WIP): A list of other mods that your mod depends on.

## Writing Your Mod

The main script file for your mod is located at `src/mod.lua`. This is the
entry point for your mod's code.  

Refer to the definition files in `<game-dir>/.cache/lua` for available
functions and types provided by EML and the game or check the examples
in the `examples` directory of the repository.
