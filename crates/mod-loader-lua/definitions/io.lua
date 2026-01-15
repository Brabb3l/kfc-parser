--- @meta

--- @class IO
io = {}

--- Provides the contents of a file as a `Buffer`.
---
--- # Errors
--- - If the file does not exist or cannot be read.
---
--- # Example
--- ```lua
--- local buffer = io.read("path/to/file.txt")
---
--- -- Outputs the content of the file as a string.
--- print(buffer:to_string())
--- ```
---
--- @param path string
--- @return Buffer
function io.read(path) end

--- Reads the entire content of a file and returns it as a string.
---
--- This is a convenience function that combines `io.read` and `Buffer:to_string()`.
---
--- # Errors
--- - If the file does not exist or cannot be read.
---
--- # Example
--- ```lua
--- local content = io.read_to_string("path/to/file.txt")
---
--- print(content)
--- ```
---
--- @param path string
--- @return string
function io.read_to_string(path) end

--- Returns an array of paths within the specified directory.
---
--- # Errors
--- - If the directory does not exist or cannot be read.
---
--- # Example
--- ```lua
--- local files = io.list_files("path/to/directory")
---
--- for _, file in ipairs(files) do
---     print(file.name)
--- end
--- ```
---
--- @param path string
--- @return string[]
function io.list_files(path) end

--- Returns `true` if the file or directory at the given path exists.
---
--- If an error occurs while checking (e.g., due to permission issues), it will return `false`.
---
--- # Example
--- ```lua
--- if io.file_exists("path/to/file.txt") then
---     print("File exists!")
--- end
--- ```
---
--- @param path string
--- @return boolean
function io.exists(path) end

--- Returns `true` if the path is a file.
---
--- If an error occurs while checking (e.g., due to permission issues), it will return `false`.
---
--- # Example
--- ```lua
--- if io.is_file("path/to/file.txt") then
---     print("It's a file!")
--- end
--- ```
---
--- @param path string
--- @return boolean
function io.is_file(path) end

--- Returns `true` if the path is a directory.
---
--- If an error occurs while checking (e.g., due to permission issues), it will return `false`.
---
--- # Example
--- ```lua
--- if io.is_directory("path/to/directory") then
---     print("It's a directory!")
--- end
--- ```
---
--- @param path string
--- @return boolean
function io.is_directory(path) end

--- Returns the name (including the extension) from a given path.
---
--- If the path is a file, it returns the file name.
--- If the path is a directory, it returns the directory name.
---
--- Returns an empty string if the path terminates in `..`.
---
--- # Example
--- ```lua
--- local file_name = io.name("path/to/file.txt")
---
--- assert(file_name == "file.txt")
--- ```
---
--- @param path string
--- @return string
function io.name(path) end

--- Returns the name of the file without its extension.
---
--- If the path has no name, it returns an empty string.
---
--- # Example
--- ```lua
--- local name_without_ext = io.name_without_extension("path/to/file.txt")
---
--- assert(name_without_ext == "file")
--- ```
---
--- @param path string
--- @return string
function io.name_without_extension(path) end

--- Returns the extension (without the dot) of the file name.
---
--- If the path has no name or no extension, it returns an empty string.
---
--- # Example
--- ```lua
--- local ext = io.extension("path/to/file.txt")
---
--- assert(ext == "txt")
--- ```
---
--- @param path string
--- @return string
function io.extension(path) end

--- Returns the parent directory of the given path.
---
--- If the path has no parent (e.g., it's a root directory), it returns `nil`.
---
--- # Example
--- ```lua
--- local parent_dir = io.parent_directory("path/to/file.txt")
---
--- assert(parent_dir == "path/to")
--- ```
---
--- @param path string
--- @return string|nil
function io.parent(path) end

--- Joins multiple path segments into a single path.
---
--- # Example
--- ```lua
--- local full_path = io.join("path", "to", "file.txt")
---
--- assert(full_path == "path/to/file.txt")
--- ```
---
--- @param ... string
--- @return string
--- @nodiscard
function io.join(...) end

--- Writes the contents of a string or `Buffer` to a file at the specified path relative to the configured export directory.
---
--- [loader.features.export](lua://loader.features.export) must be enabled for this function to work, otherwise it will throw an error.
---
--- # Behavior
--- If the file already exists, it will be overwritten.
--- If the file does not exist, it will be created along with any necessary parent directories.
---
--- # Errors
--- - If the [loader.features.export](lua://loader.features.export) feature is not enabled.
--- - If the file cannot be written (e.g., due to permission issues).
--- - If the path points to a directory.
--- - If the path points outside the configured export directory.
---
--- # Example
--- ```lua
--- -- Exports "Hello, World!" to "<export-dir>/greetings/hello.txt"
--- io.export("greetings/hello.txt", "Hello, World!")
--- ```
---
--- @param path string
--- @param bytes string|Buffer
--- @return Buffer
function io.export(path, bytes) end
