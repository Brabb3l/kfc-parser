use crate::lua::LuaValue;

pub struct LuaError;

macro_rules! error {
    ($($tt:tt)*) => {
        mlua::Error::external(format!($($tt)*))
    };
}

#[allow(unused)]
impl LuaError {

    pub fn generic(
        message: impl Into<String>
    ) -> mlua::Error {
        error!("{}", message.into())
    }

    pub fn invalid_operation(
        operation: impl Into<String>,
        value_type: impl Into<String>
    ) -> mlua::Error {
        error!(
            "attempt to {} a {} value",
            operation.into(),
            value_type.into()
        )
    }

    pub fn bad_argument(
        index: usize,
        cause: impl Into<Box<dyn std::error::Error + Send + Sync>>,
    ) -> mlua::Error {
        error!(
            "bad argument #{} ({})",
            index + 1,
            cause.into()
        )
    }

    pub fn bad_argument_type(
        index: usize,
        expected: impl Into<String>,
        got: &LuaValue,
    ) -> mlua::Error {
        error!(
            "bad argument #{} ({} expected, got {})",
            index + 1,
            expected.into(),
            type_name(got),
        )
    }

    pub fn bad_method_call(
        expected: impl Into<String>,
        got: &LuaValue,
    ) -> mlua::Error {
        error!(
            "attempt to call method on invalid receiver ({} expected, got {})",
            expected.into(),
            type_name(got),
        )
    }

    pub fn type_mismatch(
        expected: impl Into<String>,
        got: &LuaValue,
    ) -> mlua::Error {
        error!(
            "{} expected, got {}",
            expected.into(),
            type_name(got),
        )
    }

    pub fn type_mismatch_with_context(
        context: impl AsRef<str>,
        expected: impl Into<String>,
        got: &LuaValue,
    ) -> mlua::Error {
        error!(
            "{} ({} expected, got {})",
            context.as_ref(),
            expected.into(),
            type_name(got),
        )
    }

    pub fn position_out_of_bounds(
        position: usize,
        start: usize,
        end: usize,
    ) -> mlua::Error {
        error!(
            "position out of bounds: expected index in range [{}, {}], got {}",
            start,
            end,
            position
        )
    }


    pub fn position_out_of_bounds_named(
        name: impl AsRef<str>,
        position: usize,
        start: usize,
        end: usize,
    ) -> mlua::Error {
        error!(
            "position out of bounds: expected {} in range [{}, {}], got {}",
            name.as_ref(),
            start,
            end,
            position
        )
    }

    pub fn module_not_found(
        path: impl Into<String>,
    ) -> mlua::Error {
        error!(
            "module not found: {}",
            path.into()
        )
    }

    pub fn module_load(
        path: impl Into<String>,
        cause: impl Into<Box<dyn std::error::Error + Send + Sync>>,
    ) -> mlua::Error {
        error!(
            "failed to load module: {} ({})",
            path.into(),
            cause.into()
        )
    }

    pub fn mod_not_found(
        id: impl Into<String>,
    ) -> mlua::Error {
        error!(
            "mod not found: {}",
            id.into()
        )
    }

    pub fn circular_dependency(
        path: impl Into<String>,
    ) -> mlua::Error {
        error!(
            "circular dependency detected: {}",
            path.into()
        )
    }

    pub fn type_not_found(
        hash: u32,
    ) -> mlua::Error {
        error!(
            "type not found: {}",
            hash
        )
    }

    pub fn content_not_found(
        guid: impl Into<String>,
    ) -> mlua::Error {
        error!(
            "content not found: {}",
            guid.into()
        )
    }

    pub fn external(
        error: impl Into<Box<dyn std::error::Error + Send + Sync>>,
    ) -> mlua::Error {
        error!("{}", error.into())
    }

}

fn type_name(value: &LuaValue) -> String {
    match value {
        LuaValue::UserData(_) => format!("{value:#?}"),
        _ => value.type_name().to_string(),
    }
}
