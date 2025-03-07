# KFC Parser (WIP)

Parser for unpacking and repacking Enshrouded game files.

## Features

- Unpacking and repacking (1:1) all types of descriptor files.
- Extracting reflection data from the enshrouded executable. (Only windows x64)
- Disassembling and assembling of impact programs.

## Usage

### Unpacking and Repacking

To unpack game files, use the `unpack` command.

```sh
kfc-parser.exe unpack -g <game-dir> -o <output-dir> [OPTIONS]
```

To repack unpacked files, use the `repack` command.

It will repack all `.json` files in the input directory which have a
qualified guid name (e.g. `82706b40-61b1-4b8f-8b23-dcec6971bda1_9398e747_0.json`).
The hash between the two underscores (`9398e747` in this case) is used to determine the file type.

```sh
kfc-parser.exe repack -g <game-dir> -i <input-dir> [OPTIONS]
```

### Restoring Original Game Files

To restore the original game files, use the `restore` command.

```sh
kfc-parser.exe restore -g <game-dir>
```

### Impact CLI

The `impact` sub command can be used to convert an impact program into
a more manageable format and vice versa.

The `disassemble` command will convert an impact program into a `.impact` and `.shutdown.impact` 
file which will contain the program's bytecode in text format and a `.data.json` file which will
contain the program's data such as variables, etc.

```sh
kfc-parser.exe impact disassemble -i <input-file-name>
```

To convert the disassembled files back into an impact program, use the `assemble` command.

The `input-file-name` should be the shared name of the disassembled files as follows:
- `<input-file-name>.impact`
- `<input-file-name>.shutdown.impact`
- `<input-file-name>.data.json`

```sh
kfc-parser.exe impact assemble -i <input-file-name> [OPTIONS]
```

### Extracting Reflection Data

To extract reflection data from the enshrouded executable, use the `extract-types` command.

**Note:** This is automatically executed when unpacking or repacking files.

```sh
kfc-parser.exe extract-types [OPTIONS]
```

## TODO

- Implement unpacking/repacking of blob files such as images, sounds, etc.
