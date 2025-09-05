# Usage Instructions for Enshrouded Mod Loader (EML)

This document provides instructions on how to use the Enshrouded Mod Loader
to load and manage mods for the game Enshrouded.

There are currently two ways to use EML:

1. Using the [Enshrouded Mod Loader Proxy DLL](#using-the-proxy-dll-recommended) (recommended)
2. Using the [Enshrouded Mod Manager CLI](#using-the-cli)

**Note:** The `export` feature can only be used with the CLI method.

## Using the Proxy DLL (Recommended)

This is the recommended way to use EML as it provides a seamless experience
for loading mods directly when launching the game.

### Prerequisites

- Ensure you have a legal copy of the game Enshrouded installed on your system.
- Download the latest version of the Enshrouded Mod Loader Proxy DLL from **WIP**

### Installation

1. Extract the contents of the downloaded archive.
2. Copy the `dinput8.dll` file to the root directory of your Enshrouded installation.
3. Create a `mods` directory in the root directory of your Enshrouded installation
   if it doesn't already exist already.
4. Place the mods you want to use in the `mods` directory.
5. (Optional) If you want to enable the console for debugging purposes,
   change the Launch Options of the game in Steam to:

   ```bash
   EML_CONSOLE=1 %command%
   ```

6. Launch the game.

## Using the CLI

This method allows you to load mods using the command line interface.
Additionally, it can also be used to run mods with the `export` capability.

### Prerequisites

- Ensure you have a legal copy of the game Enshrouded installed on your system.
- Download the latest version of the Enshrouded Mod Manager CLI from **WIP**
- A terminal or command prompt of your choice.
- Basic knowledge of command line usage.

### Loading Mods

To load mods using the CLI, follow these steps:

1. Open a terminal or command prompt.
2. Navigate to the directory where you extracted the Enshrouded Mod Manager CLI.
   (or put it in your PATH)
3. Run the following command to run the mods:

   ```bash
   eml.exe run -g <game-dir> [OPTIONS]
   ```

   By default, this will only validate the mods and not actually run them.
   To actually run the mods, you need to pass feature flags for the capabilities
   you want to enable.

   For example, to enable the `patch` and `export` capabilities, you would run:

   ```bash
   eml.exe run -g <game-dir> --export --patch
   ```

4. Launch the game.
5. (Optional) If you want to enable the console for debugging purposes,
   change the Launch Options of the game in Steam to:

   ```bash
   EML_CONSOLE=1 %command%
   ```
