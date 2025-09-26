use std::rc::Rc;

use indexmap::IndexMap;
use mlua::{AnyUserData, Function, MetaMethod, String, UserData, UserDataMethods};
use ouroboros::self_referencing;

use crate::lua::{FunctionArgs, LuaValue, MethodArgs};

pub struct ReadOnlyMap {
    table: Rc<IndexMap<String, LuaValue>>,
}

impl ReadOnlyMap {

    pub fn new(table: IndexMap<String, LuaValue>) -> Self {
        Self {
            table: Rc::new(table),
        }
    }

}

impl UserData for ReadOnlyMap {

    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_meta_function(MetaMethod::Index, |lua, args: MethodArgs| {
            let this = args.this::<&Self>()?;
            let key = args.get::<&[u8]>(0)?;
            let key = lua.create_string(&key)?;

            Ok(this.table.get(&key).cloned().unwrap_or(LuaValue::Nil))
        });

        methods.add_meta_function(MetaMethod::Len, |_, args: MethodArgs| {
            let this = args.this::<&Self>()?;
            Ok(this.table.len() as i64)
        });

        methods.add_meta_function(MetaMethod::Pairs, |lua, args: MethodArgs| {
            let this = args.this::<&Self>()?;
            ReadOnlyMapIter::create(&this, lua)
        });
    }

}

#[self_referencing]
struct ReadOnlyMapIter {
    table: Rc<IndexMap<String, LuaValue>>,
    #[borrows(table)]
    #[not_covariant]
    iter: indexmap::map::Iter<'this, String, LuaValue>,
}

impl ReadOnlyMapIter {

    fn create(
        value: &ReadOnlyMap,
        lua: &mlua::Lua,
    ) -> mlua::Result<(Function, AnyUserData, LuaValue)> {
        let state = Self::new(
            value.table.clone(),
            |data| data.iter()
        );
        let ud = lua.create_userdata(state)?;
        let iter_fn = lua.create_function(|lua, args: FunctionArgs| {
            let mut state = args.get::<&mut Self>(0)?;

            state.next(lua)
        })?;

        Ok((iter_fn, ud, LuaValue::Nil))
    }

    fn next(
        &mut self,
        lua: &mlua::Lua,
    ) -> mlua::Result<(LuaValue, LuaValue)> {
        let entry = self.with_iter_mut(|keys| {
            keys.next()
        });

        let (key, value) = match entry {
            Some(k) => k,
            None => return Ok((LuaValue::Nil, LuaValue::Nil)),
        };

        Ok((LuaValue::String(lua.create_string(key.as_bytes())?), value.clone()))
    }

}

impl UserData for ReadOnlyMapIter {}

pub struct ReadOnlyArray {
    array: Rc<Vec<LuaValue>>,
}

impl ReadOnlyArray {

    pub fn new(array: Vec<LuaValue>) -> Self {
        Self {
            array: Rc::new(array),
        }
    }

}

impl UserData for ReadOnlyArray {

    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_meta_function(MetaMethod::Index, |_, args: MethodArgs| {
            let this = args.this::<&Self>()?;
            let key = args.get::<usize>(0)?;

            if key == 0 {
                return Ok(LuaValue::Nil);
            }

            Ok(this.array.get(key - 1).cloned().unwrap_or(LuaValue::Nil))
        });

        methods.add_meta_function(MetaMethod::Len, |_, args: MethodArgs| {
            let this = args.this::<&Self>()?;
            Ok(this.array.len() as i64)
        });

        methods.add_meta_function(MetaMethod::Pairs, |lua, args: MethodArgs| {
            let this = args.this::<&Self>()?;
            ReadOnlyArrayIter::create(&this, lua)
        });
    }

}

struct ReadOnlyArrayIter {
    array: Rc<Vec<LuaValue>>,
    index: usize,
}

impl ReadOnlyArrayIter {

    fn create(
        value: &ReadOnlyArray,
        lua: &mlua::Lua,
    ) -> mlua::Result<(Function, AnyUserData, LuaValue)> {
        let state = Self {
            array: value.array.clone(),
            index: 0,
        };
        let ud = lua.create_userdata(state)?;
        let iter_fn = lua.create_function(|_, args: FunctionArgs| {
            let mut state = args.get::<&mut Self>(0)?;

            state.next()
        })?;

        Ok((iter_fn, ud, LuaValue::Nil))
    }

    fn next(
        &mut self,
    ) -> mlua::Result<(LuaValue, LuaValue)> {
        if self.index >= self.array.len() {
            return Ok((LuaValue::Nil, LuaValue::Nil));
        }

        let key = self.index;
        let value = self.array[self.index].clone();

        self.index += 1;

        Ok((LuaValue::Integer(key as i64 + 1), value))
    }

}

impl UserData for ReadOnlyArrayIter {}
