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

```sh
kfc-parser.exe repack [OPTIONS]
```

## TODO

- Implement unpacking/repacking of blob files such as images, sounds, etc.
- Implement adding new files to the game archives.
- Provide a static and dynamic library for the parser.
- Use a scripting language such as lua to modify game files and also resolve collisions between mods.
