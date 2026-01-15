# Utilities

This section covers smaller and niche modules that provide various utility functions for different purposes.
These modules may not fit into the main categories but are still useful for mod development.

## Guid Helper

The `game.guid` module provides utility functions to work with GUID strings.

- `hash`: Computes a hash value from a GUID string.
- `from_content_hash`: Converts a `keen::ContentHash` object to a GUID string.
- `to_content_hash`: Converts a GUID string to a `keen::ContentHash` object.

## Hasher

The `hasher` module provides functions to compute hash values for data.

Currently supported hash algorithms are:
- `fnv1a32`: FNV-1a 32-bit hash
- `crc32`: CRC-32/ISO-HDLC checksum
- `crc64`: CRC-64/ECMA-182 checksum

## Integer

The `integer` module provides functions for working with fixed-sized integers.
It covers all standard sizes from 8-bit to 64-bit, both signed and unsigned.

Each type has the same set of fields:

- `MAX`: The maximum representable value for the integer type.
- `MIN`: The minimum representable value for the integer type.
- `BITS`: The number of bits used to represent the integer type.

Each type also has the following utility functions:

- `parse(string)`: Parses a string and returns the corresponding integer value.
  If the string is not a valid representation of the integer type, nil is returned.
- `truncate(value)`: Truncates the bits of a given number to fit within the bounds of the integer type.
- `clamp(value)`: Clamps a given number to fit within the bounds of the integer type.
  If the number is less than `MIN`, `MIN` is returned.
  If the number is greater than `MAX`, `MAX` is returned.
- `is_valid(value)`: Checks if a given number is within the bounds of the integer type.
- `to_string(value)`: Converts a given integer value to its string representation.

Besides these utility functions, each type has a bunch of functions for various operations on the integer type.
I will only cover a few important things here, check the `base.lua` definition file for more information about all available functions.

Besides the regular arithmetic functions like `add`, `sub`, `mul`, `div`,
there is also a `checked`, `saturating`, `wrapping`, and `overflowing` variant for each arithmetic operation.

- `checked_*(a, b)`: Performs the operation and returns nil if an overflow occurs.
- `saturating_*(a, b)`: Performs the operation and clamps the result to the bounds of the integer type if an overflow occurs.
- `wrapping_*(a, b)`: Performs the operation and wraps around the result if an overflow.
  This is the default behavior for integer operations.
- `overflowing_*(a, b)`: Performs the operation and returns a tuple containing the result and a boolean indicating whether an overflow occurred.
  If an overflow occurs, the result is wrapped around.
