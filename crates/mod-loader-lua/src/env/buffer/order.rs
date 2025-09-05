use mlua::IntoLua;

use crate::lua::{CastLua, LuaValue};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ByteOrder {
    BigEndian,
    LittleEndian,
}

impl CastLua<'_> for ByteOrder {
    type Output = Self;

    fn try_cast_lua(
        value: &LuaValue
    ) -> mlua::Result<Option<Self::Output>> {
        match &value {
            LuaValue::String(s) => if s == "big" {
                Ok(Some(Self::BigEndian))
            } else if s == "little" {
                Ok(Some(Self::LittleEndian))
            } else {
                Ok(None)
            },
            _ => Ok(None),
        }
    }

    fn name() -> std::borrow::Cow<'static, str> {
        "'big' or 'little'".into()
    }
}

impl IntoLua for ByteOrder {
    fn into_lua(self, lua: &mlua::Lua) -> mlua::Result<LuaValue> {
        let s = match self {
            Self::BigEndian => "big",
            Self::LittleEndian => "little",
        };
        LuaValue::String(lua.create_string(s)?).into_lua(lua)
    }
}
