use kfc::guid::ContentHash;
use mlua::UserData;

use crate::{env::{AppState, Buffer}, lua::{LuaError, LuaValue, MethodArgs}};

pub struct Content {
    guid: ContentHash,
}

impl Content {

    pub fn new(guid: ContentHash) -> Self {
        Self {
            guid,
        }
    }

    pub fn get_data(&self, lua: &mlua::Lua) -> mlua::Result<LuaValue> {
        let app_state = lua.app_data_ref::<AppState>().unwrap();
        let buf = app_state.get_content(&self.guid)?;

        match buf {
            Some(buf) => {
                let mut buffer = Buffer::wrap(buf);

                lua.gc_step_kbytes(buffer.capacity()?.div_ceil(1024) as i32)?;
                buffer.set_writable(false)?;
                lua.create_userdata(buffer).map(LuaValue::UserData)
            }
            None => Err(LuaError::content_not_found(self.guid.to_string())),
        }
    }

}

impl UserData for Content {

    fn add_fields<F: mlua::UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("guid", |_, this| Ok(this.guid.to_string()));
        fields.add_field_method_get("size", |_, this| Ok(this.guid.size()));
    }

    fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
        methods.add_function("read_data", |lua, args: MethodArgs| {
            let this = args.this::<&Self>()?;
            this.get_data(lua)
        });
    }

}
