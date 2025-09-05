use crate::lua::LuaError;

pub struct RequirePath<'a> {
    mod_id: &'a str,
    path: &'a str,
    is_local: bool,
}

impl RequirePath<'_> {

    #[inline]
    pub fn mod_id(&self) -> &str {
        self.mod_id
    }

    #[inline]
    pub fn path(&self) -> &str {
        self.path
    }

    #[inline]
    pub fn is_local(&self) -> bool {
        self.is_local
    }

    pub fn parse<'a>(
        id: &'a str,
        fallback_mod_id: &'a str
    ) -> mlua::Result<RequirePath<'a>> {
        let mod_id: &str;
        let path: &str;
        let is_local: bool;

        if let Some(mod_id_with_path) = id.strip_prefix("mods.") {
            let mut split = mod_id_with_path.splitn(2, '.');

            mod_id = match split.next() {
                Some(id) => id,
                None => return Err(LuaError::module_not_found(id)),
            };
            path = split.next().unwrap_or("mod");
            is_local = false;
        } else {
            mod_id = fallback_mod_id;
            path = id;
            is_local = true;
        }

        Ok(RequirePath {
            mod_id,
            path,
            is_local,
        })
    }

    pub fn parse_qualified(id: &str) -> mlua::Result<RequirePath<'_>> {
        if let Some(mod_id_with_path) = id.strip_prefix("mods.") {
            let mut split = mod_id_with_path.splitn(2, '.');

            let mod_id = match split.next() {
                Some(id) => id,
                None => return Err(LuaError::module_not_found(id)),
            };
            let path = split.next().unwrap_or("mod");

            Ok(RequirePath {
                mod_id,
                path,
                is_local: false,
            })
        } else {
            Err(LuaError::module_not_found(id))
        }
    }

    #[inline]
    pub fn qualified_id(&self) -> String {
        format!("mods.{}.{}", self.mod_id, self.path)
    }

}

