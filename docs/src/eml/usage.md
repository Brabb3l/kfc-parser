# Usage Instructions

This document provides instructions on how to use the Enshrouded Mod Loader
to load and manage mods for the game Enshrouded.

There are currently two ways to use EML:

1. Using the [Enshrouded Mod Loader Proxy DLL](#using-the-proxy-dll-recommended) (recommended)
2. Using the [Enshrouded Mod Manager CLI](#using-the-cli)

## Using the Proxy DLL (Recommended)

This is the recommended way to use EML as it provides a seamless experience
for loading mods directly when launching the game.

### Prerequisites

- Ensure you have a legal copy of the game Enshrouded installed on your system.
- Download the latest version of the [dinput8.dll](https://github.com/Brabb3l/kfc-parser/actions/workflows/build_release.yml) binary.

### Installation

1. Extract the contents of the downloaded archive.
2. Copy the `dinput8.dll` file to the root directory of your Enshrouded installation.
3. Create a `mods` directory in the root directory of your Enshrouded installation
   if it doesn't already exist.
4. Place the mods you want to use in the `mods` directory.
5. (Optional) Modify the [`eml.json`](#emljson-configuration) configuration file.
    - Useful for enabling the console or export capabilities.
6. Launch the game.

### `eml.json` Configuration

This file is created automatically when you launch the game for the first time with the proxy DLL.

- `enable_console` (boolean, default: false): If set to `true`, a console window will be opened
  alongside the game for debugging purposes.
- `use_export_flag` (boolean, default: false): If set to `true`, the `export` capability will be
  enabled when launching the game. This is useful if you want to let mods export stuff when using the proxy DLL.
- `export_directory` (string, default: "export"): The directory where exported files will be saved.
  This path is relative to the game directory.

## Using the CLI

This method allows you to load mods using the command line interface.
Additionally, it can also be used to run mods with the `export` capability.

### Prerequisites

- Ensure you have a legal copy of the game Enshrouded installed on your system.
- Download the latest version of the [emm.exe](https://github.com/Brabb3l/kfc-parser/actions/workflows/build_release.yml) binary.
- A terminal or command prompt of your choice.
- Basic knowledge of command line usage.

### Loading Mods

To load mods using the CLI, follow these steps:

1. Open a terminal or command prompt.
2. Navigate to the directory where you extracted the `emm.exe` binary
   (or put it in your PATH)
3. Run the following command to run the mods:

   ```shell
   emm.exe run -g <game-dir> [OPTIONS]
   ```

   By default, this will only validate the mods and not actually run them.
   To actually run the mods, you need to pass feature flags for the capabilities
   you want to enable.

   For example, to enable the `patch` and `export` capabilities, you would run:

   ```shell
   emm.exe run -g <game-dir> --export --patch
   ```

4. Launch the game.
