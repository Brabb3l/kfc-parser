# Enshrouded Modding Tools

A collection of unofficial tools for modding the game [Enshrouded](https://store.steampowered.com/app/1203620/Enshrouded/).

The main project in this repository is the **Enshrouded Mod Loader (EML)**, a Lua-based
mod loader and framework. EML currently supports asset modifications that are applied
before the game starts. Runtime modifications are planned for the future.

- Documentation: <https://brabb3l.github.io/kfc-parser>
- Discord: <https://discord.gg/HKKyeMsKfW>
- Releases: <https://github.com/Brabb3l/kfc-parser/releases>

## Installing EML

EML works for both the game client and the dedicated server.

1. Download `dbghelp.dll` from the [latest release](https://github.com/Brabb3l/kfc-parser/releases).
2. Copy it into the root of your Enshrouded installation, next to `enshrouded.exe`
   (or `enshrouded_server.exe`).
3. Create a `mods` directory in the same location and place your mods in it.
4. Launch the game.

On Linux (Wine/Proton), the DLL override has to be enabled, for example via the Steam
launch options:

```bash
WINEDLLOVERRIDES="dbghelp=native,builtin" %command%
```

If `dbghelp.dll` does not work on your client, try `dinput8.dll` instead. Do not install both.

On first launch EML creates an `eml.json` configuration file and writes its log to
`logs/<date>.eml.log`. See the [Usage Guide](https://brabb3l.github.io/kfc-parser/eml/usage.html)
for all options.

### Uninstalling

Removing the DLL alone does not undo changes that were already applied to the game data.
Remove `dbghelp.dll` (or `dinput8.dll`) from the game directory and then either let Steam
verify the game files, or restore the original game data with:

```bash
emm.exe restore -g <game-dir>
```

## Creating Mods

Mods are written in Lua. Each mod is a directory inside `mods/` with a `mod.json` manifest
and a `src/mod.lua` entry point. A new mod template can be generated with the mod manager CLI:

```bash
emm.exe create -g <game-dir>
```

A minimal mod looks like this:

```lua
---@type keen.BalancingTable
local BalancingTable = game.assets.get_resources_by_type("keen::BalancingTable")[1].data

BalancingTable.playerBaseStamina = 500
```

See the [Development Guide](https://brabb3l.github.io/kfc-parser/eml/develop/index.html),
the [API reference](https://brabb3l.github.io/kfc-parser/eml/api/index.html) and the
[examples](examples) to get started.

## Repository Overview

| Crate | Description |
| ----- | ----------- |
| `dbghelp-proxy`, `dinput8-proxy` | Proxy DLLs that load EML into the game process |
| `mod-loader`, `mod-loader-base`, `mod-loader-lua`, `mod-loader-runtime` | The mod loader and its Lua environment |
| `mod-manager-cli` | `emm.exe`, the command line mod manager |
| `kfc`, `kfc-base`, `kfc-resource`, `kfc-content` | Reading and writing the game's `.kfc` data files, resources and content |
| `cli-legacy` | `kfc-parser.exe`, the deprecated KFC Parser CLI |

## Building from Source

Requirements: Rust 1.88.0 or newer.

```bash
cargo build --release -p dbghelp-proxy    # dbghelp.dll
cargo build --release -p mod-manager-cli  # emm.exe
```

Official builds target `x86_64-pc-windows-gnu`. The tests in the `kfc` crate that read
real game data are ignored by default and need the `GAME_DIR` environment variable:

```bash
GAME_DIR="C:/Path/To/Enshrouded" cargo test -p kfc -- --include-ignored
```

## KFC Parser CLI (deprecated)

- Unpacking and repacking (1:1) all types of descriptor files.
- Extracting reflection data from the Enshrouded executable (Windows x64 only).
- Disassembling and assembling impact programs.

Refer to the [CLI documentation](crates/cli-legacy/README.md) for usage instructions.

## License

This project is licensed under the [GNU General Public License v3.0](LICENSE).
