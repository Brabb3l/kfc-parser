# KFC Parser (WIP)

Parser for unpacking and repacking Enshrouded game files.

## Features

- Capable of unpacking and repacking (1:1) all types of descriptor files.
- Extracting reflection data from the enshrouded executable. (Only windows x64)

## Usage

To unpack game files, use the `unpack` command.

```sh
kfc-parser.exe unpack [OPTIONS]
```

To repack unpacked files, use the `repack` command.

**Note: This tool is currently not capable of adding new files but only update existing ones**

By default, it will just append the input files to the existing game archive without
touching the original files and update the offsets accordingly. 
This can be used to update individual files without repacking the whole game.

```sh
kfc-parser.exe repack [OPTIONS]
```

You can use the `--all` argument to instead create a new game archive with only the input files.
It is recommended to unpack all files first since the game might crash if a needed file is missing.

```sh
kfc-parser.exe repack --all [OPTIONS]
```

To restore the original game files, use the `restore` command.

```sh
kfc-parser.exe restore [OPTIONS]
```

To extract reflection data from the enshrouded executable, use the `extract-types` command.

```sh
kfc-parser.exe extract-types [OPTIONS]
```

## TODO

- Implement unpacking/repacking of blob files such as images, sounds, etc.
- Implement adding new files to the game archives.
- Provide a static and dynamic library for the parser.
- Use a scripting language such as lua to modify game files and also resolve collisions between mods.
