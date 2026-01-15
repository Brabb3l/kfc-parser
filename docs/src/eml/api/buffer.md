# Buffer

A `Buffer` is a (mutable or immutable) sequence of bytes for reading and writing binary data.

Buffers act like a FIFO (first-in, first-out) stream where data is read from the front and written to the back.
To achieve this, it has two 0-based byte offsets:
- **head**: The position where data is read from (next byte to read).
- **tail**: The position where data is written to (next byte to write).

That means that bytes are read in the order they were written, unless you modify the **head** or **tail** positions manually.

Reading or writing bytes advances the respective offset by the number of bytes read or written.
If **head** should ever exceeds **tail**, a read error is raised.

Additionally, buffers have a **capacity** which is the total allocated size of the buffer in bytes.
It is automatically increased as needed when writing but the amount of growth may vary.
It is not to be confused with the actual size; it is simply the amount of space the buffer can work with without needing to reallocate memory.
The actual size is determined by the difference between **tail** and **head**.

Buffers are primarily used to work with raw binary data such as [`Content`](./assets/content.md) assets.
But they may also be used to work with embedded binary data in resources.

## Creating Buffers

There are two ways to create new buffers:
- `buffer.create`: Creates a new empty mutable buffer with an optional initial capacity.
- `buffer.wrap`: Creates a new mutable buffer from a given string.

```lua
local buf1 = buffer.create(128)  -- Create an empty buffer with an initial capacity
local buf2 = buffer.wrap("Hello, World!")  -- Create a buffer from a string
```

## Reading and Writing Data

Buffers provide various methods to read and write different types of data.
Refer to the `base.lua` definition file for a complete list of available methods.

Here are some examples of reading and writing data:

```lua
local buf = buffer.create()  -- Create a new empty buffer

buf:write_u8(42)             -- Write a byte (0x2A)
buf:write_string("Hello")    -- Write a string ("Hello")

assert(buf:tail() == 6)      -- Tail is now at position 6
assert(buf:head() == 0)      -- Head is still at position 0
```

```lua
local a = buf:read_u8()        -- Read a byte (0x2A)
local str = buf:read_string(5) -- Read a string ("Hello")

assert(a == 42)                -- a is 42
assert(str == "Hello")         -- str is "Hello"

assert(buf:head() == 6)        -- Head is now at position 6
assert(buf:tail() == 6)        -- Tail is still at position 6
```

## Reading and Writing raw Resources

In some cases, you may come across resources that are still in their raw binary format.
In such cases, you can use the `read_resource` and `write_resource` methods to read and write these resources directly from/to a buffer.

This is currently the case for translations which are resources stored as a content asset.
