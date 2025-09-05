use std::rc::Rc;

use indexmap::IndexMap;
use mlua::{MetaMethod, String, UserData, UserDataMethods};
use ouroboros::self_referencing;

use crate::lua::{LuaValue, MethodArgs};

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
            let key = args.get::<std::string::String>(0)?;
            let key = lua.create_string(&key)?;

            Ok(this.table.get(&key).cloned().unwrap_or(LuaValue::Nil))
        });

        methods.add_meta_function(MetaMethod::Len, |_, args: MethodArgs| {
            let this = args.this::<&Self>()?;
            Ok(this.table.len() as i64)
        });

        methods.add_meta_function(MetaMethod::Pairs, |lua, args: MethodArgs| {
            let this = args.this::<&Self>()?;
            let mut iter = ReadOnlyMapIter::new(
                this.table.clone(),
                |table| table.iter(),
            );

            lua.create_function_mut(move |_, ()| {
                let next = iter.with_iter_mut(|iter| iter.next());

                if let Some((k, v)) = next {
                    Ok((LuaValue::String(k.clone()), v.clone()))
                } else {
                    Ok((LuaValue::Nil, LuaValue::Nil))
                }
            })
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

            Ok(this.array.get(key).cloned().unwrap_or(LuaValue::Nil))
        });

        methods.add_meta_function(MetaMethod::Len, |_, args: MethodArgs| {
            let this = args.this::<&Self>()?;
            Ok(this.array.len() as i64)
        });

        methods.add_meta_function(MetaMethod::Pairs, |lua, args: MethodArgs| {
            let this = args.this::<&Self>()?;
            let mut iter = ReadOnlyArrayIter::new(
                this.array.clone(),
                |array| array.iter(),
            );

            lua.create_function_mut(move |_, ()| {
                let next = iter.with_iter_mut(|iter| iter.next());

                if let Some(v) = next {
                    Ok((LuaValue::Nil, v.clone()))
                } else {
                    Ok((LuaValue::Nil, LuaValue::Nil))
                }
            })
        });
    }

}

#[self_referencing]
struct ReadOnlyArrayIter {
    array: Rc<Vec<LuaValue>>,
    #[borrows(array)]
    #[not_covariant]
    iter: std::slice::Iter<'this, LuaValue>,
}
