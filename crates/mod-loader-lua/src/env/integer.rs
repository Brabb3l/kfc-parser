use crate::{lua::{LuaValue, FunctionArgs, LuaError}};

pub fn create(
    lua: &mlua::Lua
) -> mlua::Result<mlua::Table> {
    let table = lua.create_table()?;

    table.raw_set("u8", create_u8(lua)?)?;
    table.raw_set("u16", create_u16(lua)?)?;
    table.raw_set("u32", create_u32(lua)?)?;
    table.raw_set("u64", create_u64(lua)?)?;

    table.raw_set("i8", create_i8(lua)?)?;
    table.raw_set("i16", create_i16(lua)?)?;
    table.raw_set("i32", create_i32(lua)?)?;
    table.raw_set("i64", create_i64(lua)?)?;

    Ok(table)
}

#[inline]
fn add_function<F, A, R>(
    lua: &mlua::Lua,
    table: &mlua::Table,
    name: &str,
    func: F
) -> mlua::Result<()>
where
    F: Fn(&mlua::Lua, A) -> mlua::Result<R> + mlua::MaybeSend + 'static,
    A: mlua::FromLuaMulti,
    R: mlua::IntoLuaMulti,
{
    let function = lua.create_function(func)?;
    table.raw_set(name, function)?;

    Ok(())
}

macro_rules! impl_integer_signed {
    ($name:ident, $type:ty) => {
        #[inline]
        fn $name(
            lua: &mlua::Lua
        ) -> mlua::Result<mlua::Table> {
            let table = lua.create_table()?;

            table.raw_set("MIN", LuaValue::Integer(<$type>::MIN as i64))?;
            table.raw_set("MAX", LuaValue::Integer(<$type>::MAX as i64))?;
            table.raw_set("BITS", LuaValue::Integer(<$type>::BITS as i64))?;

            impl_integer_fns!(lua, &table, $type);
            impl_integer_fn_wrapper!(lua, &table, UNARY, neg, $type, std::ops::Neg::neg);

            Ok(table)
        }
    };
}

macro_rules! impl_integer_unsigned {
    ($name:ident, $type:ty) => {
        #[inline]
        fn $name(
            lua: &mlua::Lua
        ) -> mlua::Result<mlua::Table> {
            let table = lua.create_table()?;

            table.raw_set("MIN", LuaValue::Integer(<$type>::MIN as i64))?;
            table.raw_set("MAX", LuaValue::Integer(<$type>::MAX as i64))?;
            table.raw_set("BITS", LuaValue::Integer(<$type>::BITS as i64))?;

            impl_integer_fns!(lua, &table, $type);

            Ok(table)
        }
    };
}

macro_rules! impl_integer_fns {
    ($lua:expr, $table:expr, $type:ty) => {
        add_function($lua, $table, "parse", |_, value: FunctionArgs| {
            match value.get::<String>(0)?.parse::<$type>() {
                Ok(v) => Ok(LuaValue::Integer(v as i64)),
                Err(_) => Ok(LuaValue::Nil),
            }
        })?;
        add_function($lua, $table, "truncate", |_, value: LuaValue| {
            match value {
                LuaValue::Integer(i) => Ok(LuaValue::Integer(i as $type as i64)),
                LuaValue::Number(n) => Ok(LuaValue::Integer(n as $type as i64)),
                _ => Err(LuaError::bad_argument_type(1, "number or integer", &value).into()),
            }
        })?;
        add_function($lua, $table, "clamp", |_, value: LuaValue| {
            match value {
                LuaValue::Integer(i) => {
                    let clamped = i.clamp(<$type>::MIN as i64, <$type>::MAX as i64) as $type;
                    Ok(LuaValue::Integer(clamped as i64))
                }
                LuaValue::Number(n) => {
                    let clamped = n.clamp(<$type>::MIN as f64, <$type>::MAX as f64) as $type;
                    Ok(LuaValue::Integer(clamped as i64))
                }
                _ => Err(LuaError::bad_argument_type(1, "number or integer", &value).into()),
            }
        })?;
        add_function($lua, $table, "is_valid", |_, value: LuaValue| {
            match value {
                LuaValue::Integer(i) => {
                    if i >= <$type>::MIN as i64 && i <= <$type>::MAX as i64 {
                        Ok(LuaValue::Boolean(true))
                    } else {
                        Ok(LuaValue::Boolean(false))
                    }
                }
                LuaValue::Number(n) => {
                    if n.fract() == 0.0 && n >= <$type>::MIN as f64 && n <= <$type>::MAX as f64 {
                        Ok(LuaValue::Boolean(true))
                    } else {
                        Ok(LuaValue::Boolean(false))
                    }
                }
                _ => Err(LuaError::bad_argument_type(1, "number or integer", &value).into()),
            }
        })?;
        add_function($lua, $table, "to_string", |lua, value: LuaValue| {
            match value {
                LuaValue::Integer(i) => Ok(LuaValue::String(lua.create_string((i as $type).to_string())?)),
                LuaValue::Number(n) => Ok(LuaValue::String(lua.create_string((n as $type).to_string())?)),
                _ => Err(LuaError::bad_argument_type(1, "number or integer", &value).into()),
            }
        })?;

        impl_integer_fn_wrapper!($lua, $table, UNARY, count_ones, $type);
        impl_integer_fn_wrapper!($lua, $table, UNARY, count_zeros, $type);
        impl_integer_fn_wrapper!($lua, $table, UNARY, leading_ones, $type);
        impl_integer_fn_wrapper!($lua, $table, UNARY, leading_zeros, $type);
        impl_integer_fn_wrapper!($lua, $table, UNARY, trailing_ones, $type);
        impl_integer_fn_wrapper!($lua, $table, UNARY, trailing_zeros, $type);
        impl_integer_fn_wrapper!($lua, $table, BINARY, rotate_left, $type, u32);
        impl_integer_fn_wrapper!($lua, $table, BINARY, rotate_right, $type, u32);
        impl_integer_fn_wrapper!($lua, $table, UNARY, swap_bytes, $type);
        impl_integer_fn_wrapper!($lua, $table, UNARY, reverse_bits, $type);
        impl_integer_fn_wrapper!($lua, $table, UNARY, from_be, $type);
        impl_integer_fn_wrapper!($lua, $table, UNARY, from_le, $type);
        impl_integer_fn_wrapper!($lua, $table, UNARY, to_be, $type);
        impl_integer_fn_wrapper!($lua, $table, UNARY, to_le, $type);

        impl_integer_fn_wrapper!($lua, $table, BINARY, checked_add, $type, $type);
        impl_integer_fn_wrapper!($lua, $table, BINARY, checked_sub, $type, $type);
        impl_integer_fn_wrapper!($lua, $table, BINARY, checked_mul, $type, $type);
        impl_integer_fn_wrapper!($lua, $table, BINARY, checked_div, $type, $type);
        impl_integer_fn_wrapper!($lua, $table, BINARY, checked_rem, $type, $type);
        impl_integer_fn_wrapper!($lua, $table, UNARY, checked_neg, $type);
        impl_integer_fn_wrapper!($lua, $table, BINARY, checked_shl, $type, u32);
        impl_integer_fn_wrapper!($lua, $table, BINARY, checked_shr, $type, u32);
        impl_integer_fn_wrapper!($lua, $table, BINARY, checked_pow, $type, u32);

        impl_integer_fn_wrapper!($lua, $table, BINARY, saturating_add, $type, $type);
        impl_integer_fn_wrapper!($lua, $table, BINARY, saturating_sub, $type, $type);
        impl_integer_fn_wrapper!($lua, $table, BINARY, saturating_mul, $type, $type);
        impl_integer_fn_wrapper!($lua, $table, BINARY, saturating_div, $type, $type);
        impl_integer_fn_wrapper!($lua, $table, BINARY, saturating_pow, $type, u32);

        impl_integer_fn_wrapper!($lua, $table, BINARY, wrapping_add, $type, $type);
        impl_integer_fn_wrapper!($lua, $table, BINARY, wrapping_sub, $type, $type);
        impl_integer_fn_wrapper!($lua, $table, BINARY, wrapping_mul, $type, $type);
        impl_integer_fn_wrapper!($lua, $table, BINARY, wrapping_div, $type, $type);
        impl_integer_fn_wrapper!($lua, $table, BINARY, wrapping_rem, $type, $type);
        impl_integer_fn_wrapper!($lua, $table, UNARY, wrapping_neg, $type);
        impl_integer_fn_wrapper!($lua, $table, BINARY, wrapping_shl, $type, u32);
        impl_integer_fn_wrapper!($lua, $table, BINARY, wrapping_shr, $type, u32);
        impl_integer_fn_wrapper!($lua, $table, BINARY, wrapping_pow, $type, u32);

        impl_integer_fn_wrapper!($lua, $table, BINARY, overflowing_add, $type, $type);
        impl_integer_fn_wrapper!($lua, $table, BINARY, overflowing_sub, $type, $type);
        impl_integer_fn_wrapper!($lua, $table, BINARY, overflowing_mul, $type, $type);
        impl_integer_fn_wrapper!($lua, $table, BINARY, overflowing_div, $type, $type);
        impl_integer_fn_wrapper!($lua, $table, BINARY, overflowing_rem, $type, $type);
        impl_integer_fn_wrapper!($lua, $table, UNARY, overflowing_neg, $type);
        impl_integer_fn_wrapper!($lua, $table, BINARY, overflowing_shl, $type, u32);
        impl_integer_fn_wrapper!($lua, $table, BINARY, overflowing_shr, $type, u32);
        impl_integer_fn_wrapper!($lua, $table, BINARY, overflowing_pow, $type, u32);

        impl_integer_fn_wrapper!($lua, $table, BINARY, add, $type, $type, <$type>::wrapping_add);
        impl_integer_fn_wrapper!($lua, $table, BINARY, sub, $type, $type, <$type>::wrapping_sub);
        impl_integer_fn_wrapper!($lua, $table, BINARY, mul, $type, $type, <$type>::wrapping_mul);
        impl_integer_fn_wrapper!($lua, $table, BINARY, div, $type, $type, <$type>::wrapping_div);
        impl_integer_fn_wrapper!($lua, $table, BINARY, rem, $type, $type, <$type>::wrapping_rem);
        impl_integer_fn_wrapper!($lua, $table, BINARY, shl, $type, u32, <$type>::wrapping_shl);
        impl_integer_fn_wrapper!($lua, $table, BINARY, shr, $type, u32, <$type>::wrapping_shr);
        impl_integer_fn_wrapper!($lua, $table, BINARY, pow, $type, u32, <$type>::wrapping_pow);

        impl_integer_fn_wrapper!($lua, $table, BINARY, bit_and, $type, $type, std::ops::BitAnd::bitand);
        impl_integer_fn_wrapper!($lua, $table, BINARY, bit_or, $type, $type, std::ops::BitOr::bitor);
        impl_integer_fn_wrapper!($lua, $table, BINARY, bit_xor, $type, $type, std::ops::BitXor::bitxor);
        impl_integer_fn_wrapper!($lua, $table, UNARY, bit_not, $type, std::ops::Not::not);
    };
}

macro_rules! impl_integer_fn_wrapper {
    ($lua:expr, $table:expr, UNARY, $name:ident, $type:ty) => {
        impl_integer_fn_wrapper!($lua, $table, UNARY, $name, $type, <$type>::$name);
    };
    ($lua:expr, $table:expr, UNARY, $name:ident, $type:ty, $op:expr) => {
        add_function($lua, $table, stringify!($name), |_, value: FunctionArgs| {
            let value = value.get::<$type>(0)?;

            Ok($op(value).convert())
        })?;
    };
    ($lua:expr, $table:expr, BINARY, $name:ident, $lhs:ty, $rhs:ty) => {
        impl_integer_fn_wrapper!($lua, $table, BINARY, $name, $lhs, $rhs, <$lhs>::$name);
    };
    ($lua:expr, $table:expr, BINARY, $name:ident, $lhs:ty, $rhs:ty, $op:expr) => {
        add_function($lua, $table, stringify!($name), |_, value: FunctionArgs| {
            let lhs = value.get::<$lhs>(0)?;
            let rhs = value.get::<$rhs>(1)?;

            Ok($op(lhs, rhs).convert())
        })?;
    };
}

trait ConvertResult {
    type Output;

    fn convert(self) -> Self::Output;
}

macro_rules! impl_primitive {
    ($($type:ty),*) => {
        $(
            impl ConvertResult for $type {
                type Output = i64;

                fn convert(self) -> Self::Output {
                    self as i64
                }
            }
        )*
    };
}

impl<T: ConvertResult> ConvertResult for (T, bool) {
    type Output = (T::Output, bool);

    fn convert(self) -> Self::Output {
        (self.0.convert(), self.1)
    }
}

impl<T: ConvertResult> ConvertResult for Option<T> {
    type Output = Option<T::Output>;

    fn convert(self) -> Self::Output {
        self.map(|v| v.convert())
    }
}

impl_primitive!(u8, u16, u32, u64, i8, i16, i32, i64);

impl_integer_unsigned!(create_u8, u8);
impl_integer_unsigned!(create_u16, u16);
impl_integer_unsigned!(create_u32, u32);
impl_integer_unsigned!(create_u64, u64);

impl_integer_signed!(create_i8, i8);
impl_integer_signed!(create_i16, i16);
impl_integer_signed!(create_i32, i32);
impl_integer_signed!(create_i64, i64);
