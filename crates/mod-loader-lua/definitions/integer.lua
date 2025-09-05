--- @meta

--- TODO: add documentation

--- @class integer
integer = {}

--- @class integer.u8
--- @field MIN u8
--- @field MAX u8
--- @field BITS u32
integer.u8 = {}

--- @param value string
--- @param radix? u32
--- @return u8|nil
function integer.u8.parse(value, radix) end

--- @param value integer|number
--- @return u8
function integer.u8.truncate(value) end

--- @param value integer|number
--- @return u8
function integer.u8.clamp(value) end

--- @param value integer|number
--- @return bool
function integer.u8.is_valid(value) end

--- @param value u8
--- @return string
function integer.u8.to_string(value) end

--- @param value u8
--- @return u32
function integer.u8.count_ones(value) end

--- @param value u8
--- @return u32
function integer.u8.count_zeros(value) end

--- @param value u8
--- @return u32
function integer.u8.leading_zeros(value) end

--- @param value u8
--- @return u32
function integer.u8.trailing_zeros(value) end

--- @param value u8
--- @return u32
function integer.u8.leading_ones(value) end

--- @param value u8
--- @return u32
function integer.u8.trailing_ones(value) end

--- @param value u8
--- @param count u32
--- @return u8
function integer.u8.rotate_left(value, count) end

--- @param value u8
--- @param count u32
--- @return u8
function integer.u8.rotate_right(value, count) end

--- @param value u8
--- @return u8
function integer.u8.swap_bytes(value) end

--- @param value u8
--- @return u8
function integer.u8.reverse_bits(value) end

--- @param value u8
--- @return u8
function integer.u8.from_be(value) end

--- @param value u8
--- @return u8
function integer.u8.from_le(value) end

--- @param value u8
--- @return u8
function integer.u8.to_be(value) end

--- @param value u8
--- @return u8
function integer.u8.to_le(value) end

--- @param lhs u8
--- @param rhs u8
--- @return u8|nil
function integer.u8.checked_add(lhs, rhs) end

--- @param lhs u8
--- @param rhs u8
--- @return u8|nil
function integer.u8.checked_sub(lhs, rhs) end

--- @param lhs u8
--- @param rhs u8
--- @return u8|nil
function integer.u8.checked_mul(lhs, rhs) end

--- @param lhs u8
--- @param rhs u8
--- @return u8|nil
function integer.u8.checked_div(lhs, rhs) end

--- @param lhs u8
--- @param rhs u8
--- @return u8|nil
function integer.u8.checked_rem(lhs, rhs) end

--- @param value u8
--- @return u8|nil
function integer.u8.checked_neg(value) end

--- @param lhs u8
--- @param rhs u32
--- @return u8|nil
function integer.u8.checked_shl(lhs, rhs) end

--- @param lhs u8
--- @param rhs u32
--- @return u8|nil
function integer.u8.checked_shr(lhs, rhs) end

--- @param lhs u8
--- @param exp u32
--- @return u8|nil
function integer.u8.checked_pow(lhs, exp) end

--- @param lhs u8
--- @param rhs u8
--- @return u8
function integer.u8.saturating_add(lhs, rhs) end

--- @param lhs u8
--- @param rhs u8
--- @return u8
function integer.u8.saturating_sub(lhs, rhs) end

--- @param lhs u8
--- @param rhs u8
--- @return u8
function integer.u8.saturating_mul(lhs, rhs) end

--- @param lhs u8
--- @param rhs u8
--- @return u8
function integer.u8.saturating_div(lhs, rhs) end

--- @param lhs u8
--- @param exp u32
--- @return u8
function integer.u8.saturating_pow(lhs, exp) end

--- @param lhs u8
--- @param rhs u8
--- @return u8
function integer.u8.wrapping_add(lhs, rhs) end

--- @param lhs u8
--- @param rhs u8
--- @return u8
function integer.u8.wrapping_sub(lhs, rhs) end

--- @param lhs u8
--- @param rhs u8
--- @return u8
function integer.u8.wrapping_mul(lhs, rhs) end

--- @param lhs u8
--- @param rhs u8
--- @return u8
function integer.u8.wrapping_div(lhs, rhs) end

--- @param lhs u8
--- @param rhs u8
--- @return u8
function integer.u8.wrapping_rem(lhs, rhs) end

--- @param value u8
--- @return u8
function integer.u8.wrapping_neg(value) end

--- @param lhs u8
--- @param rhs u32
--- @return u8
function integer.u8.wrapping_shl(lhs, rhs) end

--- @param lhs u8
--- @param rhs u32
--- @return u8
function integer.u8.wrapping_shr(lhs, rhs) end

--- @param lhs u8
--- @param exp u32
--- @return u8
function integer.u8.wrapping_pow(lhs, exp) end

--- @param lhs u8
--- @param rhs u8
--- @return u8, bool
function integer.u8.overflowing_add(lhs, rhs) end

--- @param lhs u8
--- @param rhs u8
--- @return u8, bool
function integer.u8.overflowing_sub(lhs, rhs) end

--- @param lhs u8
--- @param rhs u8
--- @return u8, bool
function integer.u8.overflowing_mul(lhs, rhs) end

--- @param lhs u8
--- @param rhs u8
--- @return u8, bool
function integer.u8.overflowing_div(lhs, rhs) end

--- @param lhs u8
--- @param rhs u8
--- @return u8, bool
function integer.u8.overflowing_rem(lhs, rhs) end

--- @param value u8
--- @return u8, bool
function integer.u8.overflowing_neg(value) end

--- @param lhs u8
--- @param rhs u32
--- @return u8, bool
function integer.u8.overflowing_shl(lhs, rhs) end

--- @param lhs u8
--- @param rhs u32
--- @return u8, bool
function integer.u8.overflowing_shr(lhs, rhs) end

--- @param lhs u8
--- @param exp u32
--- @return u8, bool
function integer.u8.overflowing_pow(lhs, exp) end

--- @param lhs u8
--- @param rhs u8
--- @return u8
function integer.u8.add(lhs, rhs) end

--- @param lhs u8
--- @param rhs u8
--- @return u8
function integer.u8.sub(lhs, rhs) end

--- @param lhs u8
--- @param rhs u8
--- @return u8
function integer.u8.mul(lhs, rhs) end

--- @param lhs u8
--- @param rhs u8
--- @return u8
function integer.u8.div(lhs, rhs) end

--- @param lhs u8
--- @param rhs u8
--- @return u8
function integer.u8.rem(lhs, rhs) end

--- @param value u8
--- @return u8
function integer.u8.neg(value) end

--- @param lhs u8
--- @param rhs u32
--- @return u8
function integer.u8.shl(lhs, rhs) end

--- @param lhs u8
--- @param rhs u32
--- @return u8
function integer.u8.shr(lhs, rhs) end

--- @param lhs u8
--- @param exp u32
--- @return u8
function integer.u8.pow(lhs, exp) end

--- @param lhs u8
--- @param rhs u8
--- @return u8
function integer.u8.bit_and(lhs, rhs) end

--- @param lhs u8
--- @param rhs u8
--- @return u8
function integer.u8.bit_or(lhs, rhs) end

--- @param lhs u8
--- @param rhs u8
--- @return u8
function integer.u8.bit_xor(lhs, rhs) end

--- @param value u8
--- @return u8
function integer.u8.bit_not(value) end

--- @class integer.u16
--- @field MIN u16
--- @field MAX u16
--- @field BITS u32
integer.u16 = {}

--- @param value string
--- @param radix? u32
--- @return u16|nil
function integer.u16.parse(value, radix) end

--- @param value integer|number
--- @return u16
function integer.u16.truncate(value) end

--- @param value integer|number
--- @return u16
function integer.u16.clamp(value) end

--- @param value integer|number
--- @return bool
function integer.u16.is_valid(value) end

--- @param value u16
--- @return string
function integer.u16.to_string(value) end

--- @param value u16
--- @return u32
function integer.u16.count_ones(value) end

--- @param value u16
--- @return u32
function integer.u16.count_zeros(value) end

--- @param value u16
--- @return u32
function integer.u16.leading_zeros(value) end

--- @param value u16
--- @return u32
function integer.u16.trailing_zeros(value) end

--- @param value u16
--- @return u32
function integer.u16.leading_ones(value) end

--- @param value u16
--- @return u32
function integer.u16.trailing_ones(value) end

--- @param value u16
--- @param count u32
--- @return u16
function integer.u16.rotate_left(value, count) end

--- @param value u16
--- @param count u32
--- @return u16
function integer.u16.rotate_right(value, count) end

--- @param value u16
--- @return u16
function integer.u16.swap_bytes(value) end

--- @param value u16
--- @return u16
function integer.u16.reverse_bits(value) end

--- @param value u16
--- @return u16
function integer.u16.from_be(value) end

--- @param value u16
--- @return u16
function integer.u16.from_le(value) end

--- @param value u16
--- @return u16
function integer.u16.to_be(value) end

--- @param value u16
--- @return u16
function integer.u16.to_le(value) end

--- @param lhs u16
--- @param rhs u16
--- @return u16|nil
function integer.u16.checked_add(lhs, rhs) end

--- @param lhs u16
--- @param rhs u16
--- @return u16|nil
function integer.u16.checked_sub(lhs, rhs) end

--- @param lhs u16
--- @param rhs u16
--- @return u16|nil
function integer.u16.checked_mul(lhs, rhs) end

--- @param lhs u16
--- @param rhs u16
--- @return u16|nil
function integer.u16.checked_div(lhs, rhs) end

--- @param lhs u16
--- @param rhs u16
--- @return u16|nil
function integer.u16.checked_rem(lhs, rhs) end

--- @param value u16
--- @return u16|nil
function integer.u16.checked_neg(value) end

--- @param lhs u16
--- @param rhs u32
--- @return u16|nil
function integer.u16.checked_shl(lhs, rhs) end

--- @param lhs u16
--- @param rhs u32
--- @return u16|nil
function integer.u16.checked_shr(lhs, rhs) end

--- @param lhs u16
--- @param exp u32
--- @return u16|nil
function integer.u16.checked_pow(lhs, exp) end

--- @param lhs u16
--- @param rhs u16
--- @return u16
function integer.u16.saturating_add(lhs, rhs) end

--- @param lhs u16
--- @param rhs u16
--- @return u16
function integer.u16.saturating_sub(lhs, rhs) end

--- @param lhs u16
--- @param rhs u16
--- @return u16
function integer.u16.saturating_mul(lhs, rhs) end

--- @param lhs u16
--- @param rhs u16
--- @return u16
function integer.u16.saturating_div(lhs, rhs) end

--- @param lhs u16
--- @param exp u32
--- @return u16
function integer.u16.saturating_pow(lhs, exp) end

--- @param lhs u16
--- @param rhs u16
--- @return u16
function integer.u16.wrapping_add(lhs, rhs) end

--- @param lhs u16
--- @param rhs u16
--- @return u16
function integer.u16.wrapping_sub(lhs, rhs) end

--- @param lhs u16
--- @param rhs u16
--- @return u16
function integer.u16.wrapping_mul(lhs, rhs) end

--- @param lhs u16
--- @param rhs u16
--- @return u16
function integer.u16.wrapping_div(lhs, rhs) end

--- @param lhs u16
--- @param rhs u16
--- @return u16
function integer.u16.wrapping_rem(lhs, rhs) end

--- @param value u16
--- @return u16
function integer.u16.wrapping_neg(value) end

--- @param lhs u16
--- @param rhs u32
--- @return u16
function integer.u16.wrapping_shl(lhs, rhs) end

--- @param lhs u16
--- @param rhs u32
--- @return u16
function integer.u16.wrapping_shr(lhs, rhs) end

--- @param lhs u16
--- @param exp u32
--- @return u16
function integer.u16.wrapping_pow(lhs, exp) end

--- @param lhs u16
--- @param rhs u16
--- @return u16, bool
function integer.u16.overflowing_add(lhs, rhs) end

--- @param lhs u16
--- @param rhs u16
--- @return u16, bool
function integer.u16.overflowing_sub(lhs, rhs) end

--- @param lhs u16
--- @param rhs u16
--- @return u16, bool
function integer.u16.overflowing_mul(lhs, rhs) end

--- @param lhs u16
--- @param rhs u16
--- @return u16, bool
function integer.u16.overflowing_div(lhs, rhs) end

--- @param lhs u16
--- @param rhs u16
--- @return u16, bool
function integer.u16.overflowing_rem(lhs, rhs) end

--- @param value u16
--- @return u16, bool
function integer.u16.overflowing_neg(value) end

--- @param lhs u16
--- @param rhs u32
--- @return u16, bool
function integer.u16.overflowing_shl(lhs, rhs) end

--- @param lhs u16
--- @param rhs u32
--- @return u16, bool
function integer.u16.overflowing_shr(lhs, rhs) end

--- @param lhs u16
--- @param exp u32
--- @return u16, bool
function integer.u16.overflowing_pow(lhs, exp) end

--- @param lhs u16
--- @param rhs u16
--- @return u16
function integer.u16.add(lhs, rhs) end

--- @param lhs u16
--- @param rhs u16
--- @return u16
function integer.u16.sub(lhs, rhs) end

--- @param lhs u16
--- @param rhs u16
--- @return u16
function integer.u16.mul(lhs, rhs) end

--- @param lhs u16
--- @param rhs u16
--- @return u16
function integer.u16.div(lhs, rhs) end

--- @param lhs u16
--- @param rhs u16
--- @return u16
function integer.u16.rem(lhs, rhs) end

--- @param value u16
--- @return u16
function integer.u16.neg(value) end

--- @param lhs u16
--- @param rhs u32
--- @return u16
function integer.u16.shl(lhs, rhs) end

--- @param lhs u16
--- @param rhs u32
--- @return u16
function integer.u16.shr(lhs, rhs) end

--- @param lhs u16
--- @param exp u32
--- @return u16
function integer.u16.pow(lhs, exp) end

--- @param lhs u16
--- @param rhs u16
--- @return u16
function integer.u16.bit_and(lhs, rhs) end

--- @param lhs u16
--- @param rhs u16
--- @return u16
function integer.u16.bit_or(lhs, rhs) end

--- @param lhs u16
--- @param rhs u16
--- @return u16
function integer.u16.bit_xor(lhs, rhs) end

--- @param value u16
--- @return u16
function integer.u16.bit_not(value) end

--- @class integer.u32
--- @field MIN u32
--- @field MAX u32
--- @field BITS u32
integer.u32 = {}

--- @param value string
--- @param radix? u32
--- @return u32|nil
function integer.u32.parse(value, radix) end

--- @param value integer|number
--- @return u32
function integer.u32.truncate(value) end

--- @param value integer|number
--- @return u32
function integer.u32.clamp(value) end

--- @param value integer|number
--- @return bool
function integer.u32.is_valid(value) end

--- @param value u32
--- @return string
function integer.u32.to_string(value) end

--- @param value u32
--- @return u32
function integer.u32.count_ones(value) end

--- @param value u32
--- @return u32
function integer.u32.count_zeros(value) end

--- @param value u32
--- @return u32
function integer.u32.leading_zeros(value) end

--- @param value u32
--- @return u32
function integer.u32.trailing_zeros(value) end

--- @param value u32
--- @return u32
function integer.u32.leading_ones(value) end

--- @param value u32
--- @return u32
function integer.u32.trailing_ones(value) end

--- @param value u32
--- @param count u32
--- @return u32
function integer.u32.rotate_left(value, count) end

--- @param value u32
--- @param count u32
--- @return u32
function integer.u32.rotate_right(value, count) end

--- @param value u32
--- @return u32
function integer.u32.swap_bytes(value) end

--- @param value u32
--- @return u32
function integer.u32.reverse_bits(value) end

--- @param value u32
--- @return u32
function integer.u32.from_be(value) end

--- @param value u32
--- @return u32
function integer.u32.from_le(value) end

--- @param value u32
--- @return u32
function integer.u32.to_be(value) end

--- @param value u32
--- @return u32
function integer.u32.to_le(value) end

--- @param lhs u32
--- @param rhs u32
--- @return u32|nil
function integer.u32.checked_add(lhs, rhs) end

--- @param lhs u32
--- @param rhs u32
--- @return u32|nil
function integer.u32.checked_sub(lhs, rhs) end

--- @param lhs u32
--- @param rhs u32
--- @return u32|nil
function integer.u32.checked_mul(lhs, rhs) end

--- @param lhs u32
--- @param rhs u32
--- @return u32|nil
function integer.u32.checked_div(lhs, rhs) end

--- @param lhs u32
--- @param rhs u32
--- @return u32|nil
function integer.u32.checked_rem(lhs, rhs) end

--- @param value u32
--- @return u32|nil
function integer.u32.checked_neg(value) end

--- @param lhs u32
--- @param rhs u32
--- @return u32|nil
function integer.u32.checked_shl(lhs, rhs) end

--- @param lhs u32
--- @param rhs u32
--- @return u32|nil
function integer.u32.checked_shr(lhs, rhs) end

--- @param lhs u32
--- @param exp u32
--- @return u32|nil
function integer.u32.checked_pow(lhs, exp) end

--- @param lhs u32
--- @param rhs u32
--- @return u32
function integer.u32.saturating_add(lhs, rhs) end

--- @param lhs u32
--- @param rhs u32
--- @return u32
function integer.u32.saturating_sub(lhs, rhs) end

--- @param lhs u32
--- @param rhs u32
--- @return u32
function integer.u32.saturating_mul(lhs, rhs) end

--- @param lhs u32
--- @param rhs u32
--- @return u32
function integer.u32.saturating_div(lhs, rhs) end

--- @param lhs u32
--- @param exp u32
--- @return u32
function integer.u32.saturating_pow(lhs, exp) end

--- @param lhs u32
--- @param rhs u32
--- @return u32
function integer.u32.wrapping_add(lhs, rhs) end

--- @param lhs u32
--- @param rhs u32
--- @return u32
function integer.u32.wrapping_sub(lhs, rhs) end

--- @param lhs u32
--- @param rhs u32
--- @return u32
function integer.u32.wrapping_mul(lhs, rhs) end

--- @param lhs u32
--- @param rhs u32
--- @return u32
function integer.u32.wrapping_div(lhs, rhs) end

--- @param lhs u32
--- @param rhs u32
--- @return u32
function integer.u32.wrapping_rem(lhs, rhs) end

--- @param value u32
--- @return u32
function integer.u32.wrapping_neg(value) end

--- @param lhs u32
--- @param rhs u32
--- @return u32
function integer.u32.wrapping_shl(lhs, rhs) end

--- @param lhs u32
--- @param rhs u32
--- @return u32
function integer.u32.wrapping_shr(lhs, rhs) end

--- @param lhs u32
--- @param exp u32
--- @return u32
function integer.u32.wrapping_pow(lhs, exp) end

--- @param lhs u32
--- @param rhs u32
--- @return u32, bool
function integer.u32.overflowing_add(lhs, rhs) end

--- @param lhs u32
--- @param rhs u32
--- @return u32, bool
function integer.u32.overflowing_sub(lhs, rhs) end

--- @param lhs u32
--- @param rhs u32
--- @return u32, bool
function integer.u32.overflowing_mul(lhs, rhs) end

--- @param lhs u32
--- @param rhs u32
--- @return u32, bool
function integer.u32.overflowing_div(lhs, rhs) end

--- @param lhs u32
--- @param rhs u32
--- @return u32, bool
function integer.u32.overflowing_rem(lhs, rhs) end

--- @param value u32
--- @return u32, bool
function integer.u32.overflowing_neg(value) end

--- @param lhs u32
--- @param rhs u32
--- @return u32, bool
function integer.u32.overflowing_shl(lhs, rhs) end

--- @param lhs u32
--- @param rhs u32
--- @return u32, bool
function integer.u32.overflowing_shr(lhs, rhs) end

--- @param lhs u32
--- @param exp u32
--- @return u32, bool
function integer.u32.overflowing_pow(lhs, exp) end

--- @param lhs u32
--- @param rhs u32
--- @return u32
function integer.u32.add(lhs, rhs) end

--- @param lhs u32
--- @param rhs u32
--- @return u32
function integer.u32.sub(lhs, rhs) end

--- @param lhs u32
--- @param rhs u32
--- @return u32
function integer.u32.mul(lhs, rhs) end

--- @param lhs u32
--- @param rhs u32
--- @return u32
function integer.u32.div(lhs, rhs) end

--- @param lhs u32
--- @param rhs u32
--- @return u32
function integer.u32.rem(lhs, rhs) end

--- @param value u32
--- @return u32
function integer.u32.neg(value) end

--- @param lhs u32
--- @param rhs u32
--- @return u32
function integer.u32.shl(lhs, rhs) end

--- @param lhs u32
--- @param rhs u32
--- @return u32
function integer.u32.shr(lhs, rhs) end

--- @param lhs u32
--- @param exp u32
--- @return u32
function integer.u32.pow(lhs, exp) end

--- @param lhs u32
--- @param rhs u32
--- @return u32
function integer.u32.bit_and(lhs, rhs) end

--- @param lhs u32
--- @param rhs u32
--- @return u32
function integer.u32.bit_or(lhs, rhs) end

--- @param lhs u32
--- @param rhs u32
--- @return u32
function integer.u32.bit_xor(lhs, rhs) end

--- @param value u32
--- @return u32
function integer.u32.bit_not(value) end

--- @class integer.u64
--- @field MIN u64
--- @field MAX u64
--- @field BITS u32
integer.u64 = {}

--- @param value string
--- @param radix? u32
--- @return u64|nil
function integer.u64.parse(value, radix) end

--- @param value integer|number
--- @return u64
function integer.u64.truncate(value) end

--- @param value integer|number
--- @return u64
function integer.u64.clamp(value) end

--- @param value integer|number
--- @return bool
function integer.u64.is_valid(value) end

--- @param value u64
--- @return string
function integer.u64.to_string(value) end

--- @param value u64
--- @return u32
function integer.u64.count_ones(value) end

--- @param value u64
--- @return u32
function integer.u64.count_zeros(value) end

--- @param value u64
--- @return u32
function integer.u64.leading_zeros(value) end

--- @param value u64
--- @return u32
function integer.u64.trailing_zeros(value) end

--- @param value u64
--- @return u32
function integer.u64.leading_ones(value) end

--- @param value u64
--- @return u32
function integer.u64.trailing_ones(value) end

--- @param value u64
--- @param count u32
--- @return u64
function integer.u64.rotate_left(value, count) end

--- @param value u64
--- @param count u32
--- @return u64
function integer.u64.rotate_right(value, count) end

--- @param value u64
--- @return u64
function integer.u64.swap_bytes(value) end

--- @param value u64
--- @return u64
function integer.u64.reverse_bits(value) end

--- @param value u64
--- @return u64
function integer.u64.from_be(value) end

--- @param value u64
--- @return u64
function integer.u64.from_le(value) end

--- @param value u64
--- @return u64
function integer.u64.to_be(value) end

--- @param value u64
--- @return u64
function integer.u64.to_le(value) end

--- @param lhs u64
--- @param rhs u64
--- @return u64|nil
function integer.u64.checked_add(lhs, rhs) end

--- @param lhs u64
--- @param rhs u64
--- @return u64|nil
function integer.u64.checked_sub(lhs, rhs) end

--- @param lhs u64
--- @param rhs u64
--- @return u64|nil
function integer.u64.checked_mul(lhs, rhs) end

--- @param lhs u64
--- @param rhs u64
--- @return u64|nil
function integer.u64.checked_div(lhs, rhs) end

--- @param lhs u64
--- @param rhs u64
--- @return u64|nil
function integer.u64.checked_rem(lhs, rhs) end

--- @param value u64
--- @return u64|nil
function integer.u64.checked_neg(value) end

--- @param lhs u64
--- @param rhs u32
--- @return u64|nil
function integer.u64.checked_shl(lhs, rhs) end

--- @param lhs u64
--- @param rhs u32
--- @return u64|nil
function integer.u64.checked_shr(lhs, rhs) end

--- @param lhs u64
--- @param exp u32
--- @return u64|nil
function integer.u64.checked_pow(lhs, exp) end

--- @param lhs u64
--- @param rhs u64
--- @return u64
function integer.u64.saturating_add(lhs, rhs) end

--- @param lhs u64
--- @param rhs u64
--- @return u64
function integer.u64.saturating_sub(lhs, rhs) end

--- @param lhs u64
--- @param rhs u64
--- @return u64
function integer.u64.saturating_mul(lhs, rhs) end

--- @param lhs u64
--- @param rhs u64
--- @return u64
function integer.u64.saturating_div(lhs, rhs) end

--- @param lhs u64
--- @param exp u32
--- @return u64
function integer.u64.saturating_pow(lhs, exp) end

--- @param lhs u64
--- @param rhs u64
--- @return u64
function integer.u64.wrapping_add(lhs, rhs) end

--- @param lhs u64
--- @param rhs u64
--- @return u64
function integer.u64.wrapping_sub(lhs, rhs) end

--- @param lhs u64
--- @param rhs u64
--- @return u64
function integer.u64.wrapping_mul(lhs, rhs) end

--- @param lhs u64
--- @param rhs u64
--- @return u64
function integer.u64.wrapping_div(lhs, rhs) end

--- @param lhs u64
--- @param rhs u64
--- @return u64
function integer.u64.wrapping_rem(lhs, rhs) end

--- @param value u64
--- @return u64
function integer.u64.wrapping_neg(value) end

--- @param lhs u64
--- @param rhs u32
--- @return u64
function integer.u64.wrapping_shl(lhs, rhs) end

--- @param lhs u64
--- @param rhs u32
--- @return u64
function integer.u64.wrapping_shr(lhs, rhs) end

--- @param lhs u64
--- @param exp u32
--- @return u64
function integer.u64.wrapping_pow(lhs, exp) end

--- @param lhs u64
--- @param rhs u64
--- @return u64, bool
function integer.u64.overflowing_add(lhs, rhs) end

--- @param lhs u64
--- @param rhs u64
--- @return u64, bool
function integer.u64.overflowing_sub(lhs, rhs) end

--- @param lhs u64
--- @param rhs u64
--- @return u64, bool
function integer.u64.overflowing_mul(lhs, rhs) end

--- @param lhs u64
--- @param rhs u64
--- @return u64, bool
function integer.u64.overflowing_div(lhs, rhs) end

--- @param lhs u64
--- @param rhs u64
--- @return u64, bool
function integer.u64.overflowing_rem(lhs, rhs) end

--- @param value u64
--- @return u64, bool
function integer.u64.overflowing_neg(value) end

--- @param lhs u64
--- @param rhs u32
--- @return u64, bool
function integer.u64.overflowing_shl(lhs, rhs) end

--- @param lhs u64
--- @param rhs u32
--- @return u64, bool
function integer.u64.overflowing_shr(lhs, rhs) end

--- @param lhs u64
--- @param exp u32
--- @return u64, bool
function integer.u64.overflowing_pow(lhs, exp) end

--- @param lhs u64
--- @param rhs u64
--- @return u64
function integer.u64.add(lhs, rhs) end

--- @param lhs u64
--- @param rhs u64
--- @return u64
function integer.u64.sub(lhs, rhs) end

--- @param lhs u64
--- @param rhs u64
--- @return u64
function integer.u64.mul(lhs, rhs) end

--- @param lhs u64
--- @param rhs u64
--- @return u64
function integer.u64.div(lhs, rhs) end

--- @param lhs u64
--- @param rhs u64
--- @return u64
function integer.u64.rem(lhs, rhs) end

--- @param value u64
--- @return u64
function integer.u64.neg(value) end

--- @param lhs u64
--- @param rhs u32
--- @return u64
function integer.u64.shl(lhs, rhs) end

--- @param lhs u64
--- @param rhs u32
--- @return u64
function integer.u64.shr(lhs, rhs) end

--- @param lhs u64
--- @param exp u32
--- @return u64
function integer.u64.pow(lhs, exp) end

--- @param lhs u64
--- @param rhs u64
--- @return u64
function integer.u64.bit_and(lhs, rhs) end

--- @param lhs u64
--- @param rhs u64
--- @return u64
function integer.u64.bit_or(lhs, rhs) end

--- @param lhs u64
--- @param rhs u64
--- @return u64
function integer.u64.bit_xor(lhs, rhs) end

--- @param value u64
--- @return u64
function integer.u64.bit_not(value) end

--- @class integer.i8
--- @field MIN i8
--- @field MAX i8
--- @field BITS u32
integer.i8 = {}

--- @param value string
--- @param radix? u32
--- @return i8|nil
function integer.i8.parse(value, radix) end

--- @param value integer|number
--- @return i8
function integer.i8.truncate(value) end

--- @param value integer|number
--- @return i8
function integer.i8.clamp(value) end

--- @param value integer|number
--- @return bool
function integer.i8.is_valid(value) end

--- @param value i8
--- @return string
function integer.i8.to_string(value) end

--- @param value i8
--- @return u32
function integer.i8.count_ones(value) end

--- @param value i8
--- @return u32
function integer.i8.count_zeros(value) end

--- @param value i8
--- @return u32
function integer.i8.leading_zeros(value) end

--- @param value i8
--- @return u32
function integer.i8.trailing_zeros(value) end

--- @param value i8
--- @return u32
function integer.i8.leading_ones(value) end

--- @param value i8
--- @return u32
function integer.i8.trailing_ones(value) end

--- @param value i8
--- @param count u32
--- @return i8
function integer.i8.rotate_left(value, count) end

--- @param value i8
--- @param count u32
--- @return i8
function integer.i8.rotate_right(value, count) end

--- @param value i8
--- @return i8
function integer.i8.swap_bytes(value) end

--- @param value i8
--- @return i8
function integer.i8.reverse_bits(value) end

--- @param value i8
--- @return i8
function integer.i8.from_be(value) end

--- @param value i8
--- @return i8
function integer.i8.from_le(value) end

--- @param value i8
--- @return i8
function integer.i8.to_be(value) end

--- @param value i8
--- @return i8
function integer.i8.to_le(value) end

--- @param lhs i8
--- @param rhs i8
--- @return i8|nil
function integer.i8.checked_add(lhs, rhs) end

--- @param lhs i8
--- @param rhs i8
--- @return i8|nil
function integer.i8.checked_sub(lhs, rhs) end

--- @param lhs i8
--- @param rhs i8
--- @return i8|nil
function integer.i8.checked_mul(lhs, rhs) end

--- @param lhs i8
--- @param rhs i8
--- @return i8|nil
function integer.i8.checked_div(lhs, rhs) end

--- @param lhs i8
--- @param rhs i8
--- @return i8|nil
function integer.i8.checked_rem(lhs, rhs) end

--- @param value i8
--- @return i8|nil
function integer.i8.checked_neg(value) end

--- @param lhs i8
--- @param rhs u32
--- @return i8|nil
function integer.i8.checked_shl(lhs, rhs) end

--- @param lhs i8
--- @param rhs u32
--- @return i8|nil
function integer.i8.checked_shr(lhs, rhs) end

--- @param lhs i8
--- @param exp u32
--- @return i8|nil
function integer.i8.checked_pow(lhs, exp) end

--- @param lhs i8
--- @param rhs i8
--- @return i8
function integer.i8.saturating_add(lhs, rhs) end

--- @param lhs i8
--- @param rhs i8
--- @return i8
function integer.i8.saturating_sub(lhs, rhs) end

--- @param lhs i8
--- @param rhs i8
--- @return i8
function integer.i8.saturating_mul(lhs, rhs) end

--- @param lhs i8
--- @param rhs i8
--- @return i8
function integer.i8.saturating_div(lhs, rhs) end

--- @param lhs i8
--- @param exp u32
--- @return i8
function integer.i8.saturating_pow(lhs, exp) end

--- @param lhs i8
--- @param rhs i8
--- @return i8
function integer.i8.wrapping_add(lhs, rhs) end

--- @param lhs i8
--- @param rhs i8
--- @return i8
function integer.i8.wrapping_sub(lhs, rhs) end

--- @param lhs i8
--- @param rhs i8
--- @return i8
function integer.i8.wrapping_mul(lhs, rhs) end

--- @param lhs i8
--- @param rhs i8
--- @return i8
function integer.i8.wrapping_div(lhs, rhs) end

--- @param lhs i8
--- @param rhs i8
--- @return i8
function integer.i8.wrapping_rem(lhs, rhs) end

--- @param value i8
--- @return i8
function integer.i8.wrapping_neg(value) end

--- @param lhs i8
--- @param rhs u32
--- @return i8
function integer.i8.wrapping_shl(lhs, rhs) end

--- @param lhs i8
--- @param rhs u32
--- @return i8
function integer.i8.wrapping_shr(lhs, rhs) end

--- @param lhs i8
--- @param exp u32
--- @return i8
function integer.i8.wrapping_pow(lhs, exp) end

--- @param lhs i8
--- @param rhs i8
--- @return i8, bool
function integer.i8.overflowing_add(lhs, rhs) end

--- @param lhs i8
--- @param rhs i8
--- @return i8, bool
function integer.i8.overflowing_sub(lhs, rhs) end

--- @param lhs i8
--- @param rhs i8
--- @return i8, bool
function integer.i8.overflowing_mul(lhs, rhs) end

--- @param lhs i8
--- @param rhs i8
--- @return i8, bool
function integer.i8.overflowing_div(lhs, rhs) end

--- @param lhs i8
--- @param rhs i8
--- @return i8, bool
function integer.i8.overflowing_rem(lhs, rhs) end

--- @param value i8
--- @return i8, bool
function integer.i8.overflowing_neg(value) end

--- @param lhs i8
--- @param rhs u32
--- @return i8, bool
function integer.i8.overflowing_shl(lhs, rhs) end

--- @param lhs i8
--- @param rhs u32
--- @return i8, bool
function integer.i8.overflowing_shr(lhs, rhs) end

--- @param lhs i8
--- @param exp u32
--- @return i8, bool
function integer.i8.overflowing_pow(lhs, exp) end

--- @param lhs i8
--- @param rhs i8
--- @return i8
function integer.i8.add(lhs, rhs) end

--- @param lhs i8
--- @param rhs i8
--- @return i8
function integer.i8.sub(lhs, rhs) end

--- @param lhs i8
--- @param rhs i8
--- @return i8
function integer.i8.mul(lhs, rhs) end

--- @param lhs i8
--- @param rhs i8
--- @return i8
function integer.i8.div(lhs, rhs) end

--- @param lhs i8
--- @param rhs i8
--- @return i8
function integer.i8.rem(lhs, rhs) end

--- @param value i8
--- @return i8
function integer.i8.neg(value) end

--- @param lhs i8
--- @param rhs u32
--- @return i8
function integer.i8.shl(lhs, rhs) end

--- @param lhs i8
--- @param rhs u32
--- @return i8
function integer.i8.shr(lhs, rhs) end

--- @param lhs i8
--- @param exp u32
--- @return i8
function integer.i8.pow(lhs, exp) end

--- @param lhs i8
--- @param rhs i8
--- @return i8
function integer.i8.bit_and(lhs, rhs) end

--- @param lhs i8
--- @param rhs i8
--- @return i8
function integer.i8.bit_or(lhs, rhs) end

--- @param lhs i8
--- @param rhs i8
--- @return i8
function integer.i8.bit_xor(lhs, rhs) end

--- @param value i8
--- @return i8
function integer.i8.bit_not(value) end

--- @class integer.i16
--- @field MIN i16
--- @field MAX i16
--- @field BITS u32
integer.i16 = {}

--- @param value string
--- @param radix? u32
--- @return i16|nil
function integer.i16.parse(value, radix) end

--- @param value integer|number
--- @return i16
function integer.i16.truncate(value) end

--- @param value integer|number
--- @return i16
function integer.i16.clamp(value) end

--- @param value integer|number
--- @return bool
function integer.i16.is_valid(value) end

--- @param value i16
--- @return string
function integer.i16.to_string(value) end

--- @param value i16
--- @return u32
function integer.i16.count_ones(value) end

--- @param value i16
--- @return u32
function integer.i16.count_zeros(value) end

--- @param value i16
--- @return u32
function integer.i16.leading_zeros(value) end

--- @param value i16
--- @return u32
function integer.i16.trailing_zeros(value) end

--- @param value i16
--- @return u32
function integer.i16.leading_ones(value) end

--- @param value i16
--- @return u32
function integer.i16.trailing_ones(value) end

--- @param value i16
--- @param count u32
--- @return i16
function integer.i16.rotate_left(value, count) end

--- @param value i16
--- @param count u32
--- @return i16
function integer.i16.rotate_right(value, count) end

--- @param value i16
--- @return i16
function integer.i16.swap_bytes(value) end

--- @param value i16
--- @return i16
function integer.i16.reverse_bits(value) end

--- @param value i16
--- @return i16
function integer.i16.from_be(value) end

--- @param value i16
--- @return i16
function integer.i16.from_le(value) end

--- @param value i16
--- @return i16
function integer.i16.to_be(value) end

--- @param value i16
--- @return i16
function integer.i16.to_le(value) end

--- @param lhs i16
--- @param rhs i16
--- @return i16|nil
function integer.i16.checked_add(lhs, rhs) end

--- @param lhs i16
--- @param rhs i16
--- @return i16|nil
function integer.i16.checked_sub(lhs, rhs) end

--- @param lhs i16
--- @param rhs i16
--- @return i16|nil
function integer.i16.checked_mul(lhs, rhs) end

--- @param lhs i16
--- @param rhs i16
--- @return i16|nil
function integer.i16.checked_div(lhs, rhs) end

--- @param lhs i16
--- @param rhs i16
--- @return i16|nil
function integer.i16.checked_rem(lhs, rhs) end

--- @param value i16
--- @return i16|nil
function integer.i16.checked_neg(value) end

--- @param lhs i16
--- @param rhs u32
--- @return i16|nil
function integer.i16.checked_shl(lhs, rhs) end

--- @param lhs i16
--- @param rhs u32
--- @return i16|nil
function integer.i16.checked_shr(lhs, rhs) end

--- @param lhs i16
--- @param exp u32
--- @return i16|nil
function integer.i16.checked_pow(lhs, exp) end

--- @param lhs i16
--- @param rhs i16
--- @return i16
function integer.i16.saturating_add(lhs, rhs) end

--- @param lhs i16
--- @param rhs i16
--- @return i16
function integer.i16.saturating_sub(lhs, rhs) end

--- @param lhs i16
--- @param rhs i16
--- @return i16
function integer.i16.saturating_mul(lhs, rhs) end

--- @param lhs i16
--- @param rhs i16
--- @return i16
function integer.i16.saturating_div(lhs, rhs) end

--- @param lhs i16
--- @param exp u32
--- @return i16
function integer.i16.saturating_pow(lhs, exp) end

--- @param lhs i16
--- @param rhs i16
--- @return i16
function integer.i16.wrapping_add(lhs, rhs) end

--- @param lhs i16
--- @param rhs i16
--- @return i16
function integer.i16.wrapping_sub(lhs, rhs) end

--- @param lhs i16
--- @param rhs i16
--- @return i16
function integer.i16.wrapping_mul(lhs, rhs) end

--- @param lhs i16
--- @param rhs i16
--- @return i16
function integer.i16.wrapping_div(lhs, rhs) end

--- @param lhs i16
--- @param rhs i16
--- @return i16
function integer.i16.wrapping_rem(lhs, rhs) end

--- @param value i16
--- @return i16
function integer.i16.wrapping_neg(value) end

--- @param lhs i16
--- @param rhs u32
--- @return i16
function integer.i16.wrapping_shl(lhs, rhs) end

--- @param lhs i16
--- @param rhs u32
--- @return i16
function integer.i16.wrapping_shr(lhs, rhs) end

--- @param lhs i16
--- @param exp u32
--- @return i16
function integer.i16.wrapping_pow(lhs, exp) end

--- @param lhs i16
--- @param rhs i16
--- @return i16, bool
function integer.i16.overflowing_add(lhs, rhs) end

--- @param lhs i16
--- @param rhs i16
--- @return i16, bool
function integer.i16.overflowing_sub(lhs, rhs) end

--- @param lhs i16
--- @param rhs i16
--- @return i16, bool
function integer.i16.overflowing_mul(lhs, rhs) end

--- @param lhs i16
--- @param rhs i16
--- @return i16, bool
function integer.i16.overflowing_div(lhs, rhs) end

--- @param lhs i16
--- @param rhs i16
--- @return i16, bool
function integer.i16.overflowing_rem(lhs, rhs) end

--- @param value i16
--- @return i16, bool
function integer.i16.overflowing_neg(value) end

--- @param lhs i16
--- @param rhs u32
--- @return i16, bool
function integer.i16.overflowing_shl(lhs, rhs) end

--- @param lhs i16
--- @param rhs u32
--- @return i16, bool
function integer.i16.overflowing_shr(lhs, rhs) end

--- @param lhs i16
--- @param exp u32
--- @return i16, bool
function integer.i16.overflowing_pow(lhs, exp) end

--- @param lhs i16
--- @param rhs i16
--- @return i16
function integer.i16.add(lhs, rhs) end

--- @param lhs i16
--- @param rhs i16
--- @return i16
function integer.i16.sub(lhs, rhs) end

--- @param lhs i16
--- @param rhs i16
--- @return i16
function integer.i16.mul(lhs, rhs) end

--- @param lhs i16
--- @param rhs i16
--- @return i16
function integer.i16.div(lhs, rhs) end

--- @param lhs i16
--- @param rhs i16
--- @return i16
function integer.i16.rem(lhs, rhs) end

--- @param value i16
--- @return i16
function integer.i16.neg(value) end

--- @param lhs i16
--- @param rhs u32
--- @return i16
function integer.i16.shl(lhs, rhs) end

--- @param lhs i16
--- @param rhs u32
--- @return i16
function integer.i16.shr(lhs, rhs) end

--- @param lhs i16
--- @param exp u32
--- @return i16
function integer.i16.pow(lhs, exp) end

--- @param lhs i16
--- @param rhs i16
--- @return i16
function integer.i16.bit_and(lhs, rhs) end

--- @param lhs i16
--- @param rhs i16
--- @return i16
function integer.i16.bit_or(lhs, rhs) end

--- @param lhs i16
--- @param rhs i16
--- @return i16
function integer.i16.bit_xor(lhs, rhs) end

--- @param value i16
--- @return i16
function integer.i16.bit_not(value) end

--- @class integer.i32
--- @field MIN i32
--- @field MAX i32
--- @field BITS u32
integer.i32 = {}

--- @param value string
--- @param radix? u32
--- @return i32|nil
function integer.i32.parse(value, radix) end

--- @param value integer|number
--- @return i32
function integer.i32.truncate(value) end

--- @param value integer|number
--- @return i32
function integer.i32.clamp(value) end

--- @param value integer|number
--- @return bool
function integer.i32.is_valid(value) end

--- @param value i32
--- @return string
function integer.i32.to_string(value) end

--- @param value i32
--- @return u32
function integer.i32.count_ones(value) end

--- @param value i32
--- @return u32
function integer.i32.count_zeros(value) end

--- @param value i32
--- @return u32
function integer.i32.leading_zeros(value) end

--- @param value i32
--- @return u32
function integer.i32.trailing_zeros(value) end

--- @param value i32
--- @return u32
function integer.i32.leading_ones(value) end

--- @param value i32
--- @return u32
function integer.i32.trailing_ones(value) end

--- @param value i32
--- @param count u32
--- @return i32
function integer.i32.rotate_left(value, count) end

--- @param value i32
--- @param count u32
--- @return i32
function integer.i32.rotate_right(value, count) end

--- @param value i32
--- @return i32
function integer.i32.swap_bytes(value) end

--- @param value i32
--- @return i32
function integer.i32.reverse_bits(value) end

--- @param value i32
--- @return i32
function integer.i32.from_be(value) end

--- @param value i32
--- @return i32
function integer.i32.from_le(value) end

--- @param value i32
--- @return i32
function integer.i32.to_be(value) end

--- @param value i32
--- @return i32
function integer.i32.to_le(value) end

--- @param lhs i32
--- @param rhs i32
--- @return i32|nil
function integer.i32.checked_add(lhs, rhs) end

--- @param lhs i32
--- @param rhs i32
--- @return i32|nil
function integer.i32.checked_sub(lhs, rhs) end

--- @param lhs i32
--- @param rhs i32
--- @return i32|nil
function integer.i32.checked_mul(lhs, rhs) end

--- @param lhs i32
--- @param rhs i32
--- @return i32|nil
function integer.i32.checked_div(lhs, rhs) end

--- @param lhs i32
--- @param rhs i32
--- @return i32|nil
function integer.i32.checked_rem(lhs, rhs) end

--- @param value i32
--- @return i32|nil
function integer.i32.checked_neg(value) end

--- @param lhs i32
--- @param rhs u32
--- @return i32|nil
function integer.i32.checked_shl(lhs, rhs) end

--- @param lhs i32
--- @param rhs u32
--- @return i32|nil
function integer.i32.checked_shr(lhs, rhs) end

--- @param lhs i32
--- @param exp u32
--- @return i32|nil
function integer.i32.checked_pow(lhs, exp) end

--- @param lhs i32
--- @param rhs i32
--- @return i32
function integer.i32.saturating_add(lhs, rhs) end

--- @param lhs i32
--- @param rhs i32
--- @return i32
function integer.i32.saturating_sub(lhs, rhs) end

--- @param lhs i32
--- @param rhs i32
--- @return i32
function integer.i32.saturating_mul(lhs, rhs) end

--- @param lhs i32
--- @param rhs i32
--- @return i32
function integer.i32.saturating_div(lhs, rhs) end

--- @param lhs i32
--- @param exp u32
--- @return i32
function integer.i32.saturating_pow(lhs, exp) end

--- @param lhs i32
--- @param rhs i32
--- @return i32
function integer.i32.wrapping_add(lhs, rhs) end

--- @param lhs i32
--- @param rhs i32
--- @return i32
function integer.i32.wrapping_sub(lhs, rhs) end

--- @param lhs i32
--- @param rhs i32
--- @return i32
function integer.i32.wrapping_mul(lhs, rhs) end

--- @param lhs i32
--- @param rhs i32
--- @return i32
function integer.i32.wrapping_div(lhs, rhs) end

--- @param lhs i32
--- @param rhs i32
--- @return i32
function integer.i32.wrapping_rem(lhs, rhs) end

--- @param value i32
--- @return i32
function integer.i32.wrapping_neg(value) end

--- @param lhs i32
--- @param rhs u32
--- @return i32
function integer.i32.wrapping_shl(lhs, rhs) end

--- @param lhs i32
--- @param rhs u32
--- @return i32
function integer.i32.wrapping_shr(lhs, rhs) end

--- @param lhs i32
--- @param exp u32
--- @return i32
function integer.i32.wrapping_pow(lhs, exp) end

--- @param lhs i32
--- @param rhs i32
--- @return i32, bool
function integer.i32.overflowing_add(lhs, rhs) end

--- @param lhs i32
--- @param rhs i32
--- @return i32, bool
function integer.i32.overflowing_sub(lhs, rhs) end

--- @param lhs i32
--- @param rhs i32
--- @return i32, bool
function integer.i32.overflowing_mul(lhs, rhs) end

--- @param lhs i32
--- @param rhs i32
--- @return i32, bool
function integer.i32.overflowing_div(lhs, rhs) end

--- @param lhs i32
--- @param rhs i32
--- @return i32, bool
function integer.i32.overflowing_rem(lhs, rhs) end

--- @param value i32
--- @return i32, bool
function integer.i32.overflowing_neg(value) end

--- @param lhs i32
--- @param rhs u32
--- @return i32, bool
function integer.i32.overflowing_shl(lhs, rhs) end

--- @param lhs i32
--- @param rhs u32
--- @return i32, bool
function integer.i32.overflowing_shr(lhs, rhs) end

--- @param lhs i32
--- @param exp u32
--- @return i32, bool
function integer.i32.overflowing_pow(lhs, exp) end

--- @param lhs i32
--- @param rhs i32
--- @return i32
function integer.i32.add(lhs, rhs) end

--- @param lhs i32
--- @param rhs i32
--- @return i32
function integer.i32.sub(lhs, rhs) end

--- @param lhs i32
--- @param rhs i32
--- @return i32
function integer.i32.mul(lhs, rhs) end

--- @param lhs i32
--- @param rhs i32
--- @return i32
function integer.i32.div(lhs, rhs) end

--- @param lhs i32
--- @param rhs i32
--- @return i32
function integer.i32.rem(lhs, rhs) end

--- @param value i32
--- @return i32
function integer.i32.neg(value) end

--- @param lhs i32
--- @param rhs u32
--- @return i32
function integer.i32.shl(lhs, rhs) end

--- @param lhs i32
--- @param rhs u32
--- @return i32
function integer.i32.shr(lhs, rhs) end

--- @param lhs i32
--- @param exp u32
--- @return i32
function integer.i32.pow(lhs, exp) end

--- @param lhs i32
--- @param rhs i32
--- @return i32
function integer.i32.bit_and(lhs, rhs) end

--- @param lhs i32
--- @param rhs i32
--- @return i32
function integer.i32.bit_or(lhs, rhs) end

--- @param lhs i32
--- @param rhs i32
--- @return i32
function integer.i32.bit_xor(lhs, rhs) end

--- @param value i32
--- @return i32
function integer.i32.bit_not(value) end

--- @class integer.i64
--- @field MIN i64
--- @field MAX i64
--- @field BITS u32
integer.i64 = {}

--- @param value string
--- @param radix? u32
--- @return i64|nil
function integer.i64.parse(value, radix) end

--- @param value integer|number
--- @return i64
function integer.i64.truncate(value) end

--- @param value integer|number
--- @return i64
function integer.i64.clamp(value) end

--- @param value integer|number
--- @return bool
function integer.i64.is_valid(value) end

--- @param value i64
--- @return string
function integer.i64.to_string(value) end

--- @param value i64
--- @return u32
function integer.i64.count_ones(value) end

--- @param value i64
--- @return u32
function integer.i64.count_zeros(value) end

--- @param value i64
--- @return u32
function integer.i64.leading_zeros(value) end

--- @param value i64
--- @return u32
function integer.i64.trailing_zeros(value) end

--- @param value i64
--- @return u32
function integer.i64.leading_ones(value) end

--- @param value i64
--- @return u32
function integer.i64.trailing_ones(value) end

--- @param value i64
--- @param count u32
--- @return i64
function integer.i64.rotate_left(value, count) end

--- @param value i64
--- @param count u32
--- @return i64
function integer.i64.rotate_right(value, count) end

--- @param value i64
--- @return i64
function integer.i64.swap_bytes(value) end

--- @param value i64
--- @return i64
function integer.i64.reverse_bits(value) end

--- @param value i64
--- @return i64
function integer.i64.from_be(value) end

--- @param value i64
--- @return i64
function integer.i64.from_le(value) end

--- @param value i64
--- @return i64
function integer.i64.to_be(value) end

--- @param value i64
--- @return i64
function integer.i64.to_le(value) end

--- @param lhs i64
--- @param rhs i64
--- @return i64|nil
function integer.i64.checked_add(lhs, rhs) end

--- @param lhs i64
--- @param rhs i64
--- @return i64|nil
function integer.i64.checked_sub(lhs, rhs) end

--- @param lhs i64
--- @param rhs i64
--- @return i64|nil
function integer.i64.checked_mul(lhs, rhs) end

--- @param lhs i64
--- @param rhs i64
--- @return i64|nil
function integer.i64.checked_div(lhs, rhs) end

--- @param lhs i64
--- @param rhs i64
--- @return i64|nil
function integer.i64.checked_rem(lhs, rhs) end

--- @param value i64
--- @return i64|nil
function integer.i64.checked_neg(value) end

--- @param lhs i64
--- @param rhs u32
--- @return i64|nil
function integer.i64.checked_shl(lhs, rhs) end

--- @param lhs i64
--- @param rhs u32
--- @return i64|nil
function integer.i64.checked_shr(lhs, rhs) end

--- @param lhs i64
--- @param exp u32
--- @return i64|nil
function integer.i64.checked_pow(lhs, exp) end

--- @param lhs i64
--- @param rhs i64
--- @return i64
function integer.i64.saturating_add(lhs, rhs) end

--- @param lhs i64
--- @param rhs i64
--- @return i64
function integer.i64.saturating_sub(lhs, rhs) end

--- @param lhs i64
--- @param rhs i64
--- @return i64
function integer.i64.saturating_mul(lhs, rhs) end

--- @param lhs i64
--- @param rhs i64
--- @return i64
function integer.i64.saturating_div(lhs, rhs) end

--- @param lhs i64
--- @param exp u32
--- @return i64
function integer.i64.saturating_pow(lhs, exp) end

--- @param lhs i64
--- @param rhs i64
--- @return i64
function integer.i64.wrapping_add(lhs, rhs) end

--- @param lhs i64
--- @param rhs i64
--- @return i64
function integer.i64.wrapping_sub(lhs, rhs) end

--- @param lhs i64
--- @param rhs i64
--- @return i64
function integer.i64.wrapping_mul(lhs, rhs) end

--- @param lhs i64
--- @param rhs i64
--- @return i64
function integer.i64.wrapping_div(lhs, rhs) end

--- @param lhs i64
--- @param rhs i64
--- @return i64
function integer.i64.wrapping_rem(lhs, rhs) end

--- @param value i64
--- @return i64
function integer.i64.wrapping_neg(value) end

--- @param lhs i64
--- @param rhs u32
--- @return i64
function integer.i64.wrapping_shl(lhs, rhs) end

--- @param lhs i64
--- @param rhs u32
--- @return i64
function integer.i64.wrapping_shr(lhs, rhs) end

--- @param lhs i64
--- @param exp u32
--- @return i64
function integer.i64.wrapping_pow(lhs, exp) end

--- @param lhs i64
--- @param rhs i64
--- @return i64, bool
function integer.i64.overflowing_add(lhs, rhs) end

--- @param lhs i64
--- @param rhs i64
--- @return i64, bool
function integer.i64.overflowing_sub(lhs, rhs) end

--- @param lhs i64
--- @param rhs i64
--- @return i64, bool
function integer.i64.overflowing_mul(lhs, rhs) end

--- @param lhs i64
--- @param rhs i64
--- @return i64, bool
function integer.i64.overflowing_div(lhs, rhs) end

--- @param lhs i64
--- @param rhs i64
--- @return i64, bool
function integer.i64.overflowing_rem(lhs, rhs) end

--- @param value i64
--- @return i64, bool
function integer.i64.overflowing_neg(value) end

--- @param lhs i64
--- @param rhs u32
--- @return i64, bool
function integer.i64.overflowing_shl(lhs, rhs) end

--- @param lhs i64
--- @param rhs u32
--- @return i64, bool
function integer.i64.overflowing_shr(lhs, rhs) end

--- @param lhs i64
--- @param exp u32
--- @return i64, bool
function integer.i64.overflowing_pow(lhs, exp) end

--- @param lhs i64
--- @param rhs i64
--- @return i64
function integer.i64.add(lhs, rhs) end

--- @param lhs i64
--- @param rhs i64
--- @return i64
function integer.i64.sub(lhs, rhs) end

--- @param lhs i64
--- @param rhs i64
--- @return i64
function integer.i64.mul(lhs, rhs) end

--- @param lhs i64
--- @param rhs i64
--- @return i64
function integer.i64.div(lhs, rhs) end

--- @param lhs i64
--- @param rhs i64
--- @return i64
function integer.i64.rem(lhs, rhs) end

--- @param value i64
--- @return i64
function integer.i64.neg(value) end

--- @param lhs i64
--- @param rhs u32
--- @return i64
function integer.i64.shl(lhs, rhs) end

--- @param lhs i64
--- @param rhs u32
--- @return i64
function integer.i64.shr(lhs, rhs) end

--- @param lhs i64
--- @param exp u32
--- @return i64
function integer.i64.pow(lhs, exp) end

--- @param lhs i64
--- @param rhs i64
--- @return i64
function integer.i64.bit_and(lhs, rhs) end

--- @param lhs i64
--- @param rhs i64
--- @return i64
function integer.i64.bit_or(lhs, rhs) end

--- @param lhs i64
--- @param rhs i64
--- @return i64
function integer.i64.bit_xor(lhs, rhs) end

--- @param value i64
--- @return i64
function integer.i64.bit_not(value) end
