---@meta

-- TODO: extensions such as reading/writing vectors, matrices, objects, guids and other common types.

--- Factory for creating and wrapping [Buffer](lua://Buffer) objects.
---
--- @class BufferFactory
buffer = {}

--- Creates a new [Buffer](lua://Buffer) with an optional initial capacity.
---
--- @param initial_capacity integer? -- initial capacity in bytes (defaults to 0)
--- @return Buffer
function buffer.create(initial_capacity) end

--- Creates a new [Buffer](lua://Buffer) containing the bytes of the given string.
--- The **capacity** and **limit** are set to the length of the string, and the **position** is set to 0.
---
--- @param str string
--- @return Buffer
function buffer.wrap(str) end

--- A [Buffer](lua://Buffer) is a (mutable or immutable) sequence of bytes for reading and writing binary data.
---
--- @see BufferFactory
--- @class Buffer
local Buffer = {}

--- Returns this buffer's current **position**.
---
--- ### Errors
--- - When this buffer is closed.
---
--- @return u64
function Buffer:position() end

--- Sets this buffer's current **position** to the specified value.
---
--- ### Errors
--- - When `position` is less than 0 or greater than this buffer's **limit**.
--- - When this buffer is closed.
---
--- @param position u64
function Buffer:position(position) end

--- Returns this buffer's **limit**.
---
--- ### Errors
--- - When this buffer is closed.
---
--- @return u64
function Buffer:limit() end

--- Sets this buffer's **limit** to the specified value.
---
--- ### Errors
--- - When `limit` is less than 0 or greater than this buffer's **capacity**.
--- - When this buffer is closed.
---
--- @param limit u64
function Buffer:limit(limit) end

--- Returns the number of bytes remaining in this buffer, which is the difference between the **limit** and the current **position**.
---
--- ### Errors
--- - When this buffer is closed.
---
--- @return u64
function Buffer:remaining() end

--- Returns this buffer's **capacity**.
---
--- ### Errors
--- - When this buffer is closed.
---
--- @return u64
function Buffer:capacity() end

--- Flips this buffer, setting the **limit** to the current **position** and the **position** to 0.
--- This is typically used after writing data to the buffer to prepare it for reading.
---
--- ### Errors
--- - When this buffer is closed.
function Buffer:flip() end

--- Rewinds this buffer, setting the **position** to 0.
---
--- ### Errors
--- - When this buffer is closed.
function Buffer:rewind() end

--- Resets this buffer, setting both the **position** and **limit** to 0.
---
--- ### Errors
--- - When this buffer is closed.
function Buffer:reset() end

--- Reserves at least `length` more bytes in this buffer at the current **position**.
--- This will only increase the **capacity** of this buffer if it is not sufficient.
---
--- ### Errors
--- - When this buffer is not writable.
--- - When this buffer is closed.
---
--- @param length integer
function Buffer:reserve(length) end

--- The [ByteOrder](lua://ByteOrder) determines how multi-byte values are laid out in a buffer.
---
--- This affects methods that read or write multiple bytes at once, such as `read_i16`, `read_f32`, etc.
---
--- ### Values
--- - `little`: The least significant byte is stored at the lowest address (first).
--- - `big`: The most significant byte is stored at the lowest address (first).
--- - `default`: Currently refers to `little`, which is what the game primarily uses.
---
--- ### Example
--- Given a buffer containing the bytes `0x01 0x02`:
--- - If the byte order is `little`, reading a 16-bit integer would yield `0x0201` (513 in decimal).
--- - If the byte order is `big`, reading a 16-bit integer would yield `0x0102` (258 in decimal).
---
--- @alias ByteOrder "default" | "little" | "big"

--- Returns this buffer's [ByteOrder](lua://ByteOrder).
---
--- ### Errors
--- - When this buffer is closed.
---
--- @return "little" | "big"
function Buffer:order() end

--- Sets this buffer's [ByteOrder](lua://ByteOrder) to the specified value.
---
--- ### Errors
--- - When this buffer is closed.
---
--- @param order "little" | "big" | "default"
function Buffer:order(order) end

--- Skips the next `length` bytes in this buffer without reading them.
---
--- ### Errors
--- - When `length` is less than 0 or greater than the buffer's remaining bytes.
--- - When this buffer is not readable.
--- - When this buffer is closed.
---
--- @param length u64 -- The number of bytes to skip.
function Buffer:skip(length) end

--- Copies the contents of `src` at the specified `offset` and `length` into `self` at the current **position**.
--- If `src` is `self`, it instead moves the data within `self`.
---
--- This does not change the position of `src`.
---
--- ### Errors
--- - When `offset` and `length` are either negative or out of bounds for `src`.
--- - When `self` is not writable.
--- - When `src` is not readable.
--- - When either `self` or `src` is closed.
---
--- @param src Buffer
--- @param offset u64? -- The offset in `src` to start copying from. If not specified, it will start from the beginning of `src`.
--- @param length u64? -- The number of bytes to copy from `src`. If not specified, it will copy all remaining bytes from `src`.
function Buffer:copy(src, offset, length) end

--- Closes this buffer, releasing any resources associated with it.
--- After closing, this buffer can no longer be used.
--- This function may be called multiple times without error.
function Buffer:close() end

--- Reads the byte at this buffer's current **position** and then increments the **position** by `1`.
--- If the byte is non-zero, it will return `true`, otherwise it will return `false`.
---
--- ### Errors
--- - When there are no bytes remaining in this buffer.
--- - When this buffer is not readable.
--- - When this buffer is closed.
---
--- @return boolean -- The boolean value read.
function Buffer:read_bool() end

--- Reads the byte at this buffer's current **position**, interpreting it as an 8-bit signed integer, and then increments the **position** by `1`.
---
--- ### Errors
--- - When there are no bytes remaining in this buffer.
--- - When this buffer is not readable.
--- - When this buffer is closed.
---
--- @return i8 -- The signed 8-bit integer read.
function Buffer:read_i8() end

--- Reads the byte at this buffer's current **position**, interpreting it as an 8-bit unsigned integer, and then increments the **position** by `1`.
---
--- ### Errors
--- - When there are no bytes remaining in this buffer.
--- - When this buffer is not readable.
--- - When this buffer is closed.
---
--- @return u8 -- The unsigned 8-bit integer read.
function Buffer:read_u8() end

--- Reads the next two bytes at this buffer's current **position**, interpreting them as a 16-bit signed integer according to the current [ByteOrder](lua://ByteOrder), and then increments the **position** by `2`.
---
--- ### Errors
--- - When there are fewer than two bytes remaining in this buffer.
--- - When this buffer is not readable.
--- - When this buffer is closed.
---
--- @return i16 -- The signed 16-bit integer read.
function Buffer:read_i16() end

--- Reads the next two bytes at this buffer's current **position**, interpreting them as a 16-bit unsigned integer according to the current [ByteOrder](lua://ByteOrder), and then increments the **position** by `2`.
---
--- ### Errors
--- - When there are fewer than two bytes remaining in this buffer.
--- - When this buffer is not readable.
--- - When this buffer is closed.
---
--- @return u16 -- The unsigned 16-bit integer read.
function Buffer:read_u16() end

--- Reads the next four bytes at this buffer's current **position**, interpreting them as a 32-bit signed integer according to the current [ByteOrder](lua://ByteOrder), and then increments the **position** by `4`.
---
--- ### Errors
--- - When there are fewer than four bytes remaining in this buffer.
--- - When this buffer is not readable.
--- - When this buffer is closed.
---
--- @return i32 -- The signed 32-bit integer read.
function Buffer:read_i32() end

--- Reads the next four bytes at this buffer's current **position**, interpreting them as a 32-bit unsigned integer according to the current [ByteOrder](lua://ByteOrder), and then increments the **position** by `4`.
---
--- ### Errors
--- - When there are fewer than four bytes remaining in this buffer.
--- - When this buffer is not readable.
--- - When this buffer is closed.
---
--- @return u32 -- The unsigned 32-bit integer read.
function Buffer:read_u32() end

--- Reads the next eight bytes at this buffer's current **position**, interpreting them as a 64-bit signed integer according to the current [ByteOrder](lua://ByteOrder), and then increments the **position** by `8`.
---
--- ### Errors
--- - When there are fewer than eight bytes remaining in this buffer.
--- - When this buffer is not readable.
--- - When this buffer is closed.
---
--- @return i64 -- The signed 64-bit integer read.
function Buffer:read_i64() end

--- Reads the next eight bytes at this buffer's current **position**, interpreting them as a 64-bit unsigned integer according to the current [ByteOrder](lua://ByteOrder), and then increments the **position** by `8`.
---
--- ### Errors
--- - When there are fewer than eight bytes remaining in this buffer.
--- - When this buffer is not readable.
--- - When this buffer is closed.
---
--- @return u64 -- An integer with the same binary representation as the 64-bit unsigned integer read.
function Buffer:read_u64() end

--- Reads the next two bytes at this buffer's current **position**, interpreting them as a 16-bit floating point number according to the current [ByteOrder](lua://ByteOrder), and then increments the **position** by `2`.
---
--- ### Errors
--- - When there are fewer than two bytes remaining in this buffer.
--- - When this buffer is not readable.
--- - When this buffer is closed.
---
--- @return f16 -- The 16-bit floating point number read.
function Buffer:read_f16() end

--- Reads the next four bytes at this buffer's current **position**, interpreting them as a 32-bit floating point number according to the current [ByteOrder](lua://ByteOrder), and then increments the **position** by `4`.
---
--- ### Errors
--- - When there are fewer than four bytes remaining in this buffer.
--- - When this buffer is not readable.
--- - When this buffer is closed.
---
--- @return f32 -- The 32-bit floating point number read.
function Buffer:read_f32() end

--- Reads the next eight bytes at this buffer's current **position**, interpreting them as a 64-bit floating point number according to the current [ByteOrder](lua://ByteOrder), and then increments the **position** by `8`.
---
--- ### Errors
--- - When there are fewer than eight bytes remaining in this buffer.
--- - When this buffer is not readable.
--- - When this buffer is closed.
---
--- @return f64 -- The 64-bit floating point number read.
function Buffer:read_f64() end

--- Reads a sequence of characters from this buffer, starting at the current **position**.
--- Each character is read as a byte, and the **position** is incremented by `length`.
---
--- ### Errors
--- - When there are fewer than `length` bytes remaining in this buffer.
--- - When this buffer is not readable.
--- - When this buffer is closed.
---
--- @param length u64 -- The number of characters to read.
--- @return string -- The string read.
function Buffer:read_string(length) end

--- Reads a resource of the specified `type` from this buffer at the current **position**.
---
--- **TODO:** Currently this function will not increment the position after reading.
---
--- ### Errors
--- - When `type` is not valid.
--- - When the bytes do not match the expected format.
--- - When there are not enough bytes remaining in this buffer to read the resource.
--- - When this buffer is not readable.
--- - When this buffer is closed.
---
---@param type string|Type -- The qualified type name of the resource to read, such as `keen::RenderModel`
---@return unknown -- The resource read
function Buffer:read_resource(type) end

--- Writes a single byte at this buffer's current **position** representing the given boolean value, and then increments the **position** by `1`.
--- If the value is true, it writes 1; otherwise, it writes 0.
---
--- ### Errors
--- - When this buffer is not writable.
--- - When this buffer is closed.
---
--- @param value boolean -- The boolean value to write.
function Buffer:write_bool(value) end

--- Writes an 8-bit signed integer at this buffer's current **position**, and then increments the **position** by `1`.
---
--- ### Errors
--- - When this buffer is not writable.
--- - When this buffer is closed.
---
--- @param value i8 -- The signed 8-bit integer to write.
function Buffer:write_i8(value) end

--- Writes an 8-bit unsigned integer at this buffer's current **position**, and then increments the **position** by `1`.
---
--- ### Errors
--- - When this buffer is not writable.
--- - When this buffer is closed.
---
--- @param value u8 -- The unsigned 8-bit integer to write.
function Buffer:write_u8(value) end

--- Writes a 16-bit signed integer at this buffer's current **position** according to the current [ByteOrder](lua://ByteOrder), and then increments the **position** by `2`.
---
--- ### Errors
--- - When this buffer is not writable.
--- - When this buffer is closed.
---
--- @param value i16 -- The signed 16-bit integer to write.
function Buffer:write_i16(value) end

--- Writes a 16-bit unsigned integer at this buffer's current **position** according to the current [ByteOrder](lua://ByteOrder), and then increments the **position** by `2`.
---
--- ### Errors
--- - When this buffer is not writable.
--- - When this buffer is closed.
---
--- @param value u16 -- The unsigned 16-bit integer to write.
function Buffer:write_u16(value) end

--- Writes a 32-bit signed integer at this buffer's current **position** according to the current [ByteOrder](lua://ByteOrder), and then increments the **position** by `4`.
---
--- ### Errors
--- - When this buffer is not writable.
--- - When this buffer is closed.
---
--- @param value i32 -- The signed 32-bit integer to write.
function Buffer:write_i32(value) end

--- Writes a 32-bit unsigned integer at this buffer's current **position** according to the current [ByteOrder](lua://ByteOrder), and then increments the **position** by `4`.
---
--- ### Errors
--- - When this buffer is not writable.
--- - When this buffer is closed.
---
--- @param value u32 -- The unsigned 32-bit integer to write.
function Buffer:write_u32(value) end

--- Writes a 64-bit signed integer at this buffer's current **position** according to the current [ByteOrder](lua://ByteOrder), and then increments the **position** by `8`.
---
--- ### Errors
--- - When this buffer is not writable.
--- - When this buffer is closed.
---
--- @param value i64 -- The signed 64-bit integer to write.
function Buffer:write_i64(value) end

--- Writes a 64-bit unsigned integer at this buffer's current **position** according to the current [ByteOrder](lua://ByteOrder), and then increments the **position** by `8`.
---
--- ### Errors
--- - When this buffer is not writable.
--- - When this buffer is closed.
---
--- @param value u64 -- The unsigned 64-bit integer to write.
function Buffer:write_u64(value) end

--- Writes a 16-bit floating point number at this buffer's current **position** according to the current [ByteOrder](lua://ByteOrder), and then increments the **position** by `2`.
---
--- ### Errors
--- - When this buffer is not writable.
--- - When this buffer is closed.
---
--- @param value f16 -- The 16-bit floating point number to write.
function Buffer:write_f16(value) end

--- Writes a 32-bit floating point number at this buffer's current **position** according to the current [ByteOrder](lua://ByteOrder), and then increments the **position** by `4`.
---
--- ### Errors
--- - When this buffer is not writable.
--- - When this buffer is closed.
---
--- @param value f32 -- The 32-bit floating point number to write.
function Buffer:write_f32(value) end

--- Writes a 64-bit floating point number at this buffer's current **position** according to the current [ByteOrder](lua://ByteOrder), and then increments the **position** by `8`.
---
--- ### Errors
--- - When this buffer is not writable.
--- - When this buffer is closed.
---
--- @param value f64 -- The 64-bit floating point number to write.
function Buffer:write_f64(value) end

--- Writes the bytes of the given string at this buffer's current **position**, and then increments the **position** by the length of the string.
--- No null terminator or length prefix is written.
---
--- ### Errors
--- - When this buffer is not writable.
--- - When this buffer is closed.
---
--- @param str string -- The string to write.
function Buffer:write_string(str) end

--- Writes resource of the specified `type` at this buffer's current **position**.
---
--- ### Errors
--- - When `type` is not valid.
--- - When the value does not match the expected format for `type`.
--- - When this buffer is not writable.
--- - When this buffer is closed.
---
--- @param type string|Type -- The qualified type name of the resource to write, such as `keen::RenderModel`
--- @param value unknown -- The resource to write.
function Buffer:write_resource(type, value) end

--- Returns a string containing the bytes from this buffer between the current **position** and the **limit**.
---
--- ### Errors
--- - When this buffer is not readable.
--- - When this buffer is closed.
---
--- @return string
function Buffer:to_string() end
