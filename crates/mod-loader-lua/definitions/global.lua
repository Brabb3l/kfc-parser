--- @meta

--- @alias bool boolean

--- 8-bit unsigned integer (0 to 255)
--- @alias u8 integer

--- 8-bit signed integer (-128 to 127)
--- @alias i8 integer

--- 16-bit unsigned integer (0 to 65535)
--- @alias u16 integer

--- 16-bit signed integer (-32768 to 32767)
--- @alias i16 integer

--- 32-bit unsigned integer (0 to 4294967295)
--- @alias u32 integer

--- 32-bit signed integer (-2147483648 to 2147483647)
--- @alias i32 integer

--- 64-bit unsigned integer (0 to 18446744073709551615)
--- Treats lua's 64-bit signed integers as 64-bit unsigned values by reinterpreting the raw bits.
--- Negative `integer` values will appear as large `u64` values (e.g. -1 becomes 18446744073709551615).
--- @alias u64 integer

--- 64-bit signed integer (-9223372036854775808 to 9223372036854775807)
--- @alias i64 integer

--- 16-bit floating point number (half-precision)
--- @alias f16 number

--- 32-bit floating point number (single-precision)
--- @alias f32 number

--- 64-bit floating point number (double-precision)
--- @alias f64 number

--- A set of flags of type `T`, represented as an array of `T` values.
--- @generic T
--- @alias Bitmask<T> T[]

--- A static array of type `T` with a fixed length of `N`.
--- @generic T, N
--- @alias StaticArray<T, N> T[]

--- An array of type `T`.
--- @generic T
--- @alias Array<T> T[]

--- A variant type that can hold a value of any sub-type `T`.
--- `type` is a string representing the qualified type name of the value stored in `value`.
--- @generic T
--- @alias Variant<T> { type: string, value: T }

--- Represents a globally unique identifier (GUID) used to uniquely identify assets in the game.
--- The GUID is a 36 character long string in the format of `XXXXXXXX-XXXX-XXXX-XXXX-XXXXXXXXXXXX` where each `X` is a hexadecimal digit.
--- Accepts `Resource` and `Content` objects as well, automatically extracting their `guid` field.
--- @alias Guid string

--- A reference to an object of type `T`, represented by its GUID.
--- Accepts `Resource` objects as well, automatically extracting their `guid` field.
--- @generic T
--- @alias ObjectReference<T> Guid

--- Returns the type of the given value as a string.
--- In comparison to `type`, this function returns a more explicit name for userdata types.
--- Otherwise, it behaves like `type`.
---
--- @param value any -- The value to get the type of.
--- @return string -- The type of the value as a string.
function typeof(value) end

--- Loads the given module and returns its value. If the module is not found, it will error with a message describing the issue.
---
--- The path should be a dotted path to the module, such as `my_module.sub_module` or `mods.my_mod.my_module`.
---
--- To load a module from other mods, you can use the `mods` prefix followed by the mod id, such as `mods.id.path`.
--- When omitting the path after `mods.id`, it will load the mod's main module, which is typically `mod.lua`. (This is equivalent to `mods.id.mod`.)
---
--- @param path string -- A dotted path to the module, such as `my_module.sub_module` or `mods.my_mod.my_module`.
--- @return unknown
function require(path) end

--- The built-in modules available in the Lua environment.
--- All of these are globally accessible, but are also available under the `builtin` table.
builtin = {
	io = io,
	integer = integer,
	game = game,
	buffer = buffer,
	hasher = hasher,
	loader = loader,
	image = image,
}
