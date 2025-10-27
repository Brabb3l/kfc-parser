use std::borrow::Cow;

use half::f16;
use mlua::BorrowedBytes;

use crate::{lua::{LuaError, LuaValue}, util::short_type_name};

pub trait CastLua<'a> {
    type Output;

    fn try_cast_lua(
        value: &'a LuaValue,
    ) -> mlua::Result<Option<Self::Output>>;

    fn name() -> Cow<'static, str>;
}

impl<'a> CastLua<'a> for LuaValue {
    type Output = &'a Self;

    fn try_cast_lua(
        value: &'a LuaValue,
    ) -> mlua::Result<Option<Self::Output>> {
        Ok(Some(value))
    }

    fn name() -> Cow<'static, str> {
        "any".into()
    }
}

impl<'a> CastLua<'a> for mlua::AnyUserData {
    type Output = &'a Self;

    fn try_cast_lua(
        value: &'a LuaValue,
    ) -> mlua::Result<Option<Self::Output>> {
        match value {
            LuaValue::UserData(ud) => Ok(Some(ud)),
            _ => Ok(None),
        }
    }

    fn name() -> Cow<'static, str> {
        "userdata".into()
    }
}

impl<'a> CastLua<'a> for mlua::Table {
    type Output = &'a Self;

    fn try_cast_lua(
        value: &'a LuaValue,
    ) -> mlua::Result<Option<Self::Output>> {
        match value {
            LuaValue::Table(t) => Ok(Some(t)),
            _ => Ok(None),
        }
    }

    fn name() -> Cow<'static, str> {
        "table".into()
    }
}

impl CastLua<'_> for bool {
    type Output = Self;

    fn try_cast_lua(
        value: &LuaValue,
    ) -> mlua::Result<Option<Self::Output>> {
        match value {
            LuaValue::Boolean(b) => Ok(Some(*b)),
            _ => Ok(None),
        }
    }

    fn name() -> Cow<'static, str> {
        "boolean".into()
    }
}

impl CastLua<'_> for String {
    type Output = Self;

    fn try_cast_lua(
        value: &LuaValue,
    ) -> mlua::Result<Option<Self::Output>> {
        match value {
            LuaValue::String(s) => Ok(Some(s.to_str()?.to_string())),
            LuaValue::Number(n) => Ok(Some(n.to_string())),
            LuaValue::Integer(i) => Ok(Some(i.to_string())),
            _ => Ok(None),
        }
    }

    fn name() -> Cow<'static, str> {
        "string".into()
    }
}

impl<'a> CastLua<'a> for &[u8] {
    type Output = BorrowedBytes<'a>;

    fn try_cast_lua(
        value: &'a LuaValue,
    ) -> mlua::Result<Option<Self::Output>> {
        match value {
            LuaValue::String(s) => Ok(Some(s.as_bytes())),
            _ => Ok(None),
        }
    }

    fn name() -> Cow<'static, str> {
        "string".into()
    }
}

macro_rules! into {
    ($n:expr, u8) => { $n as u64 };
    ($n:expr, u16) => { $n as u64 };
    ($n:expr, u32) => { $n as u64 };
    ($n:expr, u64) => { $n as u64 };
    ($n:expr, usize) => { $n as u64 };
    ($n:expr, i8) => { $n as i64 };
    ($n:expr, i16) => { $n as i64 };
    ($n:expr, i32) => { $n as i64 };
    ($n:expr, i64) => { $n as i64 };
    ($n:expr, isize) => { $n as i64 };
    ($n:expr, f16) => { $n.to_f64_const() };
    ($n:expr, f32) => { $n as f64 };
    ($n:expr, f64) => { $n as f64 };
}

macro_rules! primitive_type {
    (u8) => { u64 };
    (u16) => { u64 };
    (u32) => { u64 };
    (u64) => { u64 };
    (usize) => { u64 };
    (i8) => { i64 };
    (i16) => { i64 };
    (i32) => { i64 };
    (i64) => { i64 };
    (isize) => { i64 };
    (f16) => { f64 };
    (f32) => { f64 };
    (f64) => { f64 };
}

macro_rules! from {
    ($n:expr, f16) => { f16::from_f64($n) };
    ($n:expr, $ty:ident) => { $n as $ty };
}

macro_rules! check_fractional {
    (u8, $i:expr) => { $i.fract() == 0.0 };
    (u16, $i:expr) => { $i.fract() == 0.0 };
    (u32, $i:expr) => { $i.fract() == 0.0 };
    (u64, $i:expr) => { $i.fract() == 0.0 };
    (usize, $i:expr) => { $i.fract() == 0.0 };
    (i8, $i:expr) => { $i.fract() == 0.0 };
    (i16, $i:expr) => { $i.fract() == 0.0 };
    (i32, $i:expr) => { $i.fract() == 0.0 };
    (i64, $i:expr) => { $i.fract() == 0.0 };
    (isize, $i:expr) => { $i.fract() == 0.0 };
    (f16, $i:expr) => { true };
    (f32, $i:expr) => { true };
    (f64, $i:expr) => { true };
}

macro_rules! cast {
    (u64, $i:expr, $value:expr) => { Ok(*$i as u64) };
    (i64, $i:expr, $value:expr) => { Ok(*$i as i64) };
    (f64, $i:expr, $value:expr) => { Ok(*$i as f64) };
    ($ty:ident, $i:ident, $value:ident, $mode:ident) => {{
        let i = *$i as primitive_type!($ty);

        if is!($ty, $i, $mode) {
            Ok(Some(from!(i, $ty)))
        } else {
            Ok(None)
        }
    }};
}

macro_rules! is {
    ($ty:ident, $i:expr, INTEGER) => {{
        const MIN: primitive_type!($ty) = into!(<$ty>::MIN, $ty);
        const MAX: primitive_type!($ty) = into!(<$ty>::MAX, $ty);

        let i = *$i as primitive_type!($ty);

        i >= MIN && i <= MAX
    }};
    ($ty:ident, $i:expr, NUMBER) => {{
        const MIN: primitive_type!($ty) = into!(<$ty>::MIN, $ty);
        const MAX: primitive_type!($ty) = into!(<$ty>::MAX, $ty);

        let i = *$i as primitive_type!($ty);

        check_fractional!($ty, *$i) && i >= MIN && i <= MAX
    }};
}

macro_rules! impl_primitive {
    ($ty:ident, $name:expr) => {
        impl CastLua<'_> for $ty {
            type Output = $ty;

            #[allow(clippy::manual_range_contains)]
            fn try_cast_lua(
                value: &LuaValue,
            ) -> mlua::Result<Option<Self::Output>> {
                match value {
                    LuaValue::Integer(i) => cast!($ty, i, value, INTEGER),
                    LuaValue::Number(n) => cast!($ty, n, value, NUMBER),
                    LuaValue::String(s) => Ok(s.to_str()?.parse::<$ty>().ok()),
                    _ => Ok(None),
                }
            }

            fn name() -> Cow<'static, str> {
                $name.into()
            }
        }
    };
}

impl_primitive!(u8, "8-bit unsigned integer");
impl_primitive!(u16, "16-bit unsigned integer");
impl_primitive!(u32, "32-bit unsigned integer");
impl_primitive!(u64, "64-bit unsigned integer");
impl_primitive!(usize, "64-bit unsigned integer");
impl_primitive!(i8, "8-bit signed integer");
impl_primitive!(i16, "16-bit signed integer");
impl_primitive!(i32, "32-bit signed integer");
impl_primitive!(i64, "64-bit signed integer");
impl_primitive!(isize, "64-bit signed integer");
impl_primitive!(f16, "16-bit floating point");
impl_primitive!(f32, "32-bit floating point");
impl_primitive!(f64, "64-bit floating point");

impl<U> CastLua<'_> for &U
where
    U: mlua::UserData + 'static,
{
    type Output = mlua::UserDataRef<U>;

    fn try_cast_lua(
        value: &LuaValue,
    ) -> mlua::Result<Option<Self::Output>> {
        match &value {
            LuaValue::UserData(ud) => {
                if !ud.is::<U>() {
                    return Ok(None);
                }

                match ud.borrow::<U>() {
                    Ok(data) => Ok(Some(data)),
                    Err(mlua::Error::UserDataTypeMismatch) => Ok(None),
                    Err(e) => Err(e),
                }
            }
            _ => Ok(None),
        }
    }

    fn name() -> Cow<'static, str> {
        short_type_name::<U>().into()
    }
}

impl<U> CastLua<'_> for &mut U
where
    U: mlua::UserData + 'static,
{
    type Output = mlua::UserDataRefMut<U>;

    fn try_cast_lua(
        value: &LuaValue,
    ) -> mlua::Result<Option<Self::Output>> {
        match &value {
            LuaValue::UserData(ud) => {
                if !ud.is::<U>() {
                    return Ok(None);
                }

                match ud.borrow_mut::<U>() {
                    Ok(data) => Ok(Some(data)),
                    Err(mlua::Error::UserDataTypeMismatch) => Ok(None),
                    Err(e) => Err(e),
                }
            }
            _ => Ok(None),
        }
    }

    fn name() -> Cow<'static, str> {
        short_type_name::<U>().into()
    }
}

impl<'a, T> CastLua<'a> for Option<T>
where
    T: CastLua<'a>,
{
    type Output = Option<T::Output>;

    fn try_cast_lua(
        value: &'a LuaValue,
    ) -> mlua::Result<Option<Self::Output>> {
        if value.is_nil() {
            Ok(Some(None))
        } else {
            T::try_cast_lua(value)
                .map(|v| v.map(Some))
        }
    }

    fn name() -> Cow<'static, str> {
        format!("{}?", T::name()).into()
    }
}

pub enum Either<T1, T2> {
    A(T1),
    B(T2),
}

impl<'a, T1, T2> CastLua<'a> for Either<T1, T2>
where
    T1: CastLua<'a>,
    T2: CastLua<'a>,
{
    type Output = Either<T1::Output, T2::Output>;

    fn try_cast_lua(
        value: &'a LuaValue,
    ) -> mlua::Result<Option<Self::Output>> {
        if let Some(value) = T1::try_cast_lua(value)? {
            Ok(Some(Either::A(value)))
        } else if let Some(value) = T2::try_cast_lua(value)? {
            Ok(Some(Either::B(value)))
        } else {
            Ok(None)
        }
    }

    fn name() -> Cow<'static, str> {
        format!("{} or {}", T1::name(), T2::name()).into()
    }
}

pub enum Either3<T1, T2, T3> {
    A(T1),
    B(T2),
    C(T3),
}

impl<'a, T1, T2, T3> CastLua<'a> for Either3<T1, T2, T3>
where
    T1: CastLua<'a>,
    T2: CastLua<'a>,
    T3: CastLua<'a>,
{
    type Output = Either3<T1::Output, T2::Output, T3::Output>;

    fn try_cast_lua(
        value: &'a LuaValue,
    ) -> mlua::Result<Option<Self::Output>> {
        if let Some(value) = T1::try_cast_lua(value)? {
            Ok(Some(Either3::A(value)))
        } else if let Some(value) = T2::try_cast_lua(value)? {
            Ok(Some(Either3::B(value)))
        } else if let Some(value) = T3::try_cast_lua(value)? {
            Ok(Some(Either3::C(value)))
        } else {
            Ok(None)
        }
    }

    fn name() -> Cow<'static, str> {
        format!("{}, {} or {}", T1::name(), T2::name(), T3::name()).into()
    }
}

pub trait CastLuaExt {

    fn cast<'a, T>(
        &'a self,
    ) -> mlua::Result<T::Output>
    where
        T: CastLua<'a>;

    fn cast_with_context<'a, T>(
        &'a self,
        context: impl AsRef<str>,
    ) -> mlua::Result<T::Output>
    where
        T: CastLua<'a>;

    fn try_cast<'a, T>(
        &'a self,
    ) -> mlua::Result<Option<T::Output>>
    where
        T: CastLua<'a>;

}

impl CastLuaExt for LuaValue {

    fn cast<'a, T>(
        &'a self,
    ) -> mlua::Result<T::Output>
    where
        T: CastLua<'a>,
    {
        self.try_cast::<T>()?
            .ok_or_else(|| LuaError::type_mismatch(T::name().as_ref(), self))
    }

    fn cast_with_context<'a, T>(
        &'a self,
        context: impl AsRef<str>,
    ) -> mlua::Result<T::Output>
    where
        T: CastLua<'a>,
    {
        self.try_cast::<T>()?
            .ok_or_else(|| LuaError::type_mismatch_with_context(
                context,
                T::name().as_ref(),
                self,
            ))
    }

    fn try_cast<'a, T>(
        &'a self,
    ) -> mlua::Result<Option<T::Output>>
    where
        T: CastLua<'a>,
    {
        T::try_cast_lua(self)
    }

}
