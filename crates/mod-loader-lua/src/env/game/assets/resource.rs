use std::rc::Rc;

use kfc::reflection::LookupKey;
use mlua::UserData;

use crate::{env::{AppState, ResourceInfo}, lua::LuaValue};

pub struct Resource {
    info: Rc<ResourceInfo>,
}

impl Resource {

    pub fn new(info: Rc<ResourceInfo>) -> Self {
        Self {
            info,
        }
    }

    pub fn info(&self) -> &Rc<ResourceInfo> {
        &self.info
    }

}

impl UserData for Resource {

    fn add_fields<F: mlua::UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("guid", |_, this| Ok(this.info.resource_id.to_string()));
        fields.add_field_method_get("type", |lua, this| {
            let app_state = lua.app_data_ref::<AppState>().unwrap();
            let hash = this.info.resource_id.type_hash();
            let index = app_state.type_registry()
                .get_by_hash(LookupKey::Qualified(hash))
                .map(|t| t.index);

            if let Some(index) = index {
                app_state.get_type(lua, index)
            } else {
                Ok(Some(LuaValue::Nil))
            }
        });
        fields.add_field_method_get("part", |_, this| Ok(this.info.resource_id.part_index()));
        fields.add_field_method_get("data", |lua, this| {
            this.info.get_lua_value(lua)
        });
        fields.add_field_method_set("data", |lua, this, value: LuaValue| {
            this.info.set_lua_value(value, lua)
        });
    }

}
