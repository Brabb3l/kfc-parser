# IO

The `io` module provides functions for reading and exporting files and data.

**Note:** All io operations (except `io.export`) are currently only supported for files within the mod's own directory.
Additionally, all operations are subject to io errors which may be raised if something goes wrong (e.g. file not found, permission denied, etc.).

## Reading Files

You can use the `io.read` or `io.read_to_string` functions to read a file and get its contents as a [`Buffer`](./buffer.md) object or a string respectively.
This can be useful to load configurations, binary data, or any other type of file your mod needs to work with.

```lua
local contents = io.read_to_string("path/to/file.txt")

-- Outputs the content of the file.
print(contents)
```

**Note:** Reading is currently only supported for files within the mod's own directory.

## Exporting Files

You can use the `io.export` function to write data to a file relative to the specified export directory.

The `export` capability must be enabled for this function to work, otherwise an error will be raised.

```lua
-- Exports "Hello, World!" to "<export-dir>/greetings/hello.txt"
io.export("greetings/hello.txt", "Hello, World!")
```

## File System

If you have multiple files or need to work with directories, you can use the following functions:

- `io.list_files`: Lists all files in a given directory. (non-recursive)
- `io.exists`: Checks if a file or directory exists.
- `io.is_file`: Checks if a given path is a file.
- `io.is_directory`: Checks if a given path is a directory.

There are also a few helper functions to work with file paths:

- `io.name`: Gets the name of a file (with extension) or directory from a given path.
- `io.name_without_extension`: Gets the name of a file without its extension from a given path.
- `io.extension`: Gets the extension of a file from a given path.
- `io.parent`: Gets the parent directory of a given path or nil if there is none.
- `io.join`: Joins multiple path segments into a single path.

Check the `base.lua` definition file for more information about these functions (including examples).
