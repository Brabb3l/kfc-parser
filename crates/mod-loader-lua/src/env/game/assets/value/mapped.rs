use std::{cell::RefCell, collections::HashMap, ops::Deref, rc::Rc};

use indexmap::IndexMap;
use kfc::{resource::value::{Value, Variant}, reflection::{PrimitiveType, TypeMetadata}};
use mlua::{AnyUserData, Function, MetaMethod, UserData};
use ouroboros::self_referencing;

use crate::{alias::{MappedArray, MappedStruct, MappedVariant, TypeHandle}, env::{value::{converter::{value_to_lua, Converter, LuaConversionErrorKind}, name_of, validate_and_clone_lua_value, validator::try_clone_lua_value}}, lua::{FunctionArgs, LuaError, LuaValue, MethodArgs}};

pub struct MappedStructValue {
    data: MappedStruct,
    cache: Rc<RefCell<HashMap<String, LuaValue>>>,
}

impl MappedStructValue {

    #[inline]
    pub fn new(data: MappedStruct) -> Self {
        Self {
            data,
            cache: Default::default(),
        }
    }

    #[inline]
    pub fn r#type(&self) -> &TypeHandle {
        self.data.r#type()
    }

    pub fn convert(
        &self,
        converter: &mut Converter,
    ) -> Result<Value, LuaConversionErrorKind> {
        let cache = self.cache.borrow();

        let mut result = IndexMap::with_capacity(self.data.len());

        for field in self.data.r#type().iter_fields() {
            let key = field.name.as_str();

            converter.path.push(key);

            let value = match cache.get(key) {
                Some(value) => {
                    let field_type = self.data.r#type()
                        .get_field_type(key)
                        .expect("expected a valid field in struct");

                    converter.process(
                        value,
                        &field_type,
                    )?
                }
                None => {
                    let value = self.data.get(key)?
                        .expect("expected a valid field in struct");

                    Value::from(value)?
                },
            };

            result.insert(
                key.to_string(),
                value,
            );

            converter.path.pop();
        }

        Ok(result.into())
    }

    pub fn try_clone_lua(
        &self,
        lua: &mlua::Lua,
    ) -> mlua::Result<Self> {
        let mut cache = HashMap::with_capacity(self.cache.borrow().len());

        for (key, value) in self.cache.borrow().iter() {
            let cloned_value = try_clone_lua_value(value, lua)?;
            cache.insert(key.clone(), cloned_value);
        }

        Ok(Self {
            data: self.data.clone(),
            cache: Rc::new(RefCell::new(cache)),
        })
    }

    pub fn is(
        &self,
        r#type: &TypeMetadata,
    ) -> bool {
        self.r#type().deref() == r#type
    }

    pub fn get(
        &self,
        key: String,
        lua: &mlua::Lua,
    ) -> mlua::Result<LuaValue> {
        Self::get_field(
            &self.data,
            &mut self.cache.borrow_mut(),
            lua,
            &key
        )
    }

    pub fn set(
        &self,
        key: &str,
        value: &LuaValue,
        lua: &mlua::Lua,
    ) -> mlua::Result<()> {
        let field_type = match self.data.r#type().get_field_type(key) {
            Some(field) => field,
            None => {
                return Err(LuaError::generic(format!(
                    "attempt to set invalid property '{key}' on {}",
                    name_of(self.r#type())
                )));
            }
        };

        let value = validate_and_clone_lua_value(value, &field_type, lua)
            .map_err(LuaError::external)?;

        if let Some(entry) = self.cache.borrow_mut().get_mut(key) {
            *entry = value;
        } else {
            self.cache.borrow_mut().insert(
                key.to_string(),
                value
            );
        }

        Ok(())
    }

    fn get_field(
        data: &MappedStruct,
        cache: &mut HashMap<String, LuaValue>,
        lua: &mlua::Lua,
        key: &str
    ) -> mlua::Result<LuaValue> {
        if let Some(value) = cache.get(key) {
            return Ok(value.clone());
        }

        // NOTE: won't cause an error if not mapped and the field is not present
        let field = data.get(key)
            .map_err(LuaError::external)?;

        if let Some(value) = field {
            let lua_value = value_to_lua(&value, lua)?;

            // only cache complex types
            match &lua_value {
                LuaValue::Nil |
                LuaValue::Boolean(_) |
                LuaValue::LightUserData(_) |
                LuaValue::Integer(_) |
                LuaValue::Number(_) => {},

                LuaValue::String(_) | // OPTIMIZE: check if this is worth it
                LuaValue::Table(_) |
                LuaValue::Function(_) |
                LuaValue::Thread(_) |
                LuaValue::UserData(_) |
                LuaValue::Error(_) |
                LuaValue::Other(_) => {
                    cache.insert(
                        key.to_string(),
                        lua_value.clone()
                    );
                },
            }

            Ok(lua_value)
        } else {
            Ok(LuaValue::Nil)
        }
    }

}

impl UserData for MappedStructValue {

    fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
        methods.add_meta_function(MetaMethod::Len, |_, args: MethodArgs| {
            let this = args.this::<&Self>()?;
            Ok(this.data.len())
        });

        methods.add_meta_function(MetaMethod::Index, |lua, args: MethodArgs| {
            let this = args.this::<&Self>()?;
            let key = args.get::<String>(0)?;

            this.get(key, lua)
        });

        methods.add_meta_function(MetaMethod::NewIndex, |lua, args: MethodArgs| {
            let this = args.this::<&Self>()?;
            let key = args.get::<String>(0)?;
            let value = args.get::<LuaValue>(1)?;

            this.set(&key, value, lua)
        });

        methods.add_meta_function(MetaMethod::Pairs, |lua, args: MethodArgs| {
            let this = args.this::<&Self>()?;
            MappedStructValueIter::create(&this, lua)
        });

        methods.add_meta_function(MetaMethod::ToString, |lua, args: MethodArgs| {
            let this = args.this::<&Self>()?;
            let cache = this.cache.borrow();
            let mut result = String::new();

            result.push('{');

            for (i, field) in this.r#type().iter_fields().enumerate() {
                if i > 0 {
                    result.push(',');
                }

                let key = field.name.as_str();
                let value = match cache.get(key) {
                    Some(value) => value,
                    None => {
                        let field_value = this.data.get(key)
                            .map_err(LuaError::external)?;

                        &match field_value {
                            Some(value) => value_to_lua(&value, lua)?,
                            None => panic!("expected a valid field in struct"),
                        }
                    }
                };

                result.push('"');
                result.push_str(key);
                result.push_str("\":");
                result.push_str(&value.to_string()?);
            }

            result.push('}');

            Ok(LuaValue::String(lua.create_string(result)?))
        });
    }

}

#[self_referencing]
struct MappedStructValueIter {
    data: MappedStruct,
    cache: Rc<RefCell<HashMap<String, LuaValue>>>,
    #[borrows(data)]
    #[not_covariant]
    keys: RefCell<Box<dyn Iterator<Item = &'this str> + 'this>>,
}

impl MappedStructValueIter {

    fn create(
        value: &MappedStructValue,
        lua: &mlua::Lua,
    ) -> mlua::Result<(Function, AnyUserData, LuaValue)> {
        let state = Self::new(
            value.data.clone(),
            value.cache.clone(),
            |data| RefCell::new(Box::new(data.iter_keys()))
        );
        let ud = lua.create_userdata(state)?;
        let iter_fn = lua.create_function(|lua, args: FunctionArgs| {
            let state = args.get::<&Self>(0)?;

            state.next(lua)
        })?;

        Ok((iter_fn, ud, LuaValue::Nil))
    }

    fn next(
        &self,
        lua: &mlua::Lua,
    ) -> mlua::Result<(LuaValue, LuaValue)> {
        let key = self.with_keys(|keys| {
            keys.borrow_mut().next()
        });

        let key = match key {
            Some(k) => k,
            None => return Ok((LuaValue::Nil, LuaValue::Nil)),
        };

        let value = MappedStructValue::get_field(
            self.borrow_data(),
            &mut self.borrow_cache().borrow_mut(),
            lua,
            key
        )?;

        Ok((LuaValue::String(lua.create_string(key)?), value))
    }

}

impl UserData for MappedStructValueIter {}

pub struct MappedArrayValue {
    data: MappedArray,
    values: Rc<RefCell<Vec<MappedArrayValueEntry>>>,
}

#[derive(Debug, Clone)]
enum MappedArrayValueEntry {
    Value(LuaValue),
    Uninitialized(usize),
}

impl MappedArrayValue {

    pub fn new(data: MappedArray) -> Self {
        let mut cache = Vec::with_capacity(data.len());

        for i in 0..data.len() {
            cache.push(MappedArrayValueEntry::Uninitialized(i));
        }

        Self {
            data,
            values: Rc::new(RefCell::new(cache)),
        }
    }

    #[inline]
    pub fn r#type(&self) -> &TypeHandle {
        self.data.r#type()
    }

    #[inline]
    pub fn element_type(&self) -> &TypeHandle {
        self.data.element_type()
    }

    pub fn convert(
        &self,
        converter: &mut Converter,
    ) -> Result<Value, LuaConversionErrorKind> {
        let values = self.values.borrow();
        let mut result = Vec::with_capacity(values.len());

        for entry in values.iter() {
            match entry {
                MappedArrayValueEntry::Uninitialized(i) => {
                    let original_value = self.data.get(*i)?
                        .expect("expected a value in the array");

                    result.push(Value::from(original_value)?);
                },
                MappedArrayValueEntry::Value(lua_value) => {
                    let value = converter.process(
                        lua_value,
                        self.data.element_type(),
                    ).expect("failed to convert modified value");

                    result.push(value);
                },
            }
        }

        Ok(result.into())
    }

    pub fn try_clone_lua(
        &self,
        lua: &mlua::Lua,
    ) -> mlua::Result<Self> {
        let values = self.values.borrow();
        let mut cloned_cache = Vec::with_capacity(values.len());

        for entry in values.iter() {
            match entry {
                MappedArrayValueEntry::Uninitialized(i) => {
                    cloned_cache.push(MappedArrayValueEntry::Uninitialized(*i));
                },
                MappedArrayValueEntry::Value(lua_value) => {
                    let cloned_value = try_clone_lua_value(lua_value, lua)?;
                    cloned_cache.push(MappedArrayValueEntry::Value(cloned_value));
                },
            }
        }

        Ok(Self {
            data: self.data.clone(),
            values: Rc::new(RefCell::new(cloned_cache)),
        })
    }

    pub fn is(
        &self,
        r#type: &TypeMetadata,
    ) -> bool {
        self.r#type().deref() == r#type
    }

    pub fn len(&self) -> usize {
        self.values.borrow().len()
    }

    pub fn get(
        &self,
        index: usize,
        lua: &mlua::Lua,
    ) -> mlua::Result<LuaValue> {
        if index == 0 {
            return Ok(LuaValue::Nil);
        }

        Self::get_field(
            index,
            &self.data,
            &mut self.values.borrow_mut(),
            lua
        )
    }

    pub fn set(
        &self,
        index: usize,
        value: &LuaValue,
        lua: &mlua::Lua,
    ) -> mlua::Result<()> {
        let index = self.check_range(index, 1, self.len())?;

        let value = validate_and_clone_lua_value(value, self.element_type(), lua)
            .map_err(LuaError::external)?;

        self.values.borrow_mut()[index] = MappedArrayValueEntry::Value(value);

        Ok(())
    }

    pub fn insert(
        &mut self,
        index: usize,
        value: &LuaValue,
        lua: &mlua::Lua,
    ) -> mlua::Result<()> {
        let index = self.check_range(index, 1, self.len() + 1)?;
        self.check_static()?;

        let value = validate_and_clone_lua_value(value, self.element_type(), lua)
            .map_err(LuaError::external)?;

        self.values.borrow_mut().insert(
            index,
            MappedArrayValueEntry::Value(value)
        );

        Ok(())
    }

    pub fn remove(
        &mut self,
        index: usize,
        lua: &mlua::Lua,
    ) -> mlua::Result<Option<LuaValue>> {
        if index == 0 {
            return Ok(None);
        }

        let index = self.check_range(index, 1, self.len() + 1)?;
        self.check_static()?;

        if index >= self.values.borrow().len() {
            return Ok(None);
        }

        let value = Self::get_field(
            index,
            &self.data,
            &mut self.values.borrow_mut(),
            lua
        )?;

        self.values.borrow_mut().remove(index);

        Ok(Some(value))
    }

    pub fn clear(&mut self) -> mlua::Result<()> {
        self.check_static()?;

        self.values.borrow_mut().clear();

        Ok(())
    }

    fn get_field(
        key: usize,
        data: &MappedArray,
        values: &mut [MappedArrayValueEntry],
        lua: &mlua::Lua,
    ) -> mlua::Result<LuaValue> {
        let entry = match values.get_mut(key) {
            Some(value) => value,
            None => return Ok(LuaValue::Nil),
        };

        match entry {
            MappedArrayValueEntry::Uninitialized(index) => {
                let field = data.get(*index)
                    .map_err(LuaError::external)?;

                let value = match field {
                    Some(value) => value,
                    None => return Ok(LuaValue::Nil),
                };

                let lua_value = value_to_lua(&value, lua)?;

                *entry = MappedArrayValueEntry::Value(lua_value.clone());

                Ok(lua_value)
            },
            MappedArrayValueEntry::Value(lua_value) => {
                Ok(lua_value.clone())
            },
        }
    }

    fn check_range(
        &self,
        index: usize,
        start: usize,
        end: usize,
    ) -> mlua::Result<usize> {
        if index == 0 || index > end {
            Err(LuaError::position_out_of_bounds(index, start, end))
        } else {
            Ok(index - 1)
        }
    }

    fn check_static(
        &self,
    ) -> mlua::Result<()> {
        if self.r#type().primitive_type == PrimitiveType::StaticArray {
            Err(LuaError::generic("attempt to modify static array"))
        } else {
            Ok(())
        }
    }

}

impl UserData for MappedArrayValue {

    fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
        methods.add_meta_function(MetaMethod::Len, |_, args: MethodArgs| {
            let this = args.this::<&Self>()?;
            Ok(this.values.borrow().len() as i32)
        });

        methods.add_meta_function(MetaMethod::Index, |lua, args: MethodArgs| {
            let this = args.this::<&Self>()?;
            let key = args.get::<usize>(0)?;

            this.get(key, lua)
        });

        methods.add_meta_function(MetaMethod::NewIndex, |lua, args: MethodArgs| {
            let this = args.this::<&Self>()?;
            let key = args.get::<usize>(0)?;
            let value = args.get::<LuaValue>(1)?;

            this.set(key, value, lua)
        });

        methods.add_meta_function(MetaMethod::Pairs, |lua, args: MethodArgs| {
            let this = args.this::<&Self>()?;
            MappedArrayValueIter::create(&this, lua)
        });

        methods.add_meta_function(MetaMethod::ToString, |lua, args: MethodArgs| {
            let this = args.this::<&Self>()?;
            let values = this.values.borrow();
            let mut result = String::new();

            result.push('[');

            for (i, entry) in values.iter().enumerate() {
                if i > 0 {
                    result.push(',');
                }

                let value = match entry {
                    MappedArrayValueEntry::Uninitialized(index) => {
                        let field_value = this.data.get(*index)
                            .map_err(LuaError::external)?;

                        match field_value {
                            Some(value) => &value_to_lua(&value, lua)?,
                            None => panic!("expected a valid field in struct"),
                        }
                    }
                    MappedArrayValueEntry::Value(lua_value) => lua_value,
                };

                result.push_str(&value.to_string()?);
            }

            result.push('}');

            Ok(LuaValue::String(lua.create_string(result)?))
        });
    }

}

struct MappedArrayValueIter {
    data: MappedArray,
    cache: Rc<RefCell<Vec<MappedArrayValueEntry>>>,
    index: usize,
}

impl MappedArrayValueIter {

    fn create(
        value: &MappedArrayValue,
        lua: &mlua::Lua,
    ) -> mlua::Result<(Function, AnyUserData, LuaValue)> {
        let state = Self {
            data: value.data.clone(),
            cache: value.values.clone(),
            index: 0,
        };
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
        if self.index >= self.data.len() {
            return Ok((LuaValue::Nil, LuaValue::Nil));
        }

        let key = self.index;
        let value = MappedArrayValue::get_field(
            key,
            &self.data,
            &mut self.cache.borrow_mut(),
            lua,
        )?;

        self.index += 1;

        Ok((LuaValue::Integer(key as i64 + 1), value))
    }

}

impl UserData for MappedArrayValueIter {}

pub struct MappedVariantValue {
    data: MappedVariant,
    value: RefCell<Option<LuaValue>>,
}

impl MappedVariantValue {

    pub fn new(data: MappedVariant) -> Self {
        Self {
            data,
            value: RefCell::new(None),
        }
    }

    #[inline]
    pub fn r#type(&self) -> &TypeHandle {
        self.data.base_type()
    }

    #[inline]
    pub fn variant_type(&self) -> &TypeHandle {
        self.data.variant_type()
    }

    pub fn convert(
        &self,
        converter: &mut Converter,
    ) -> Result<Value, LuaConversionErrorKind> {
        match self.value.borrow().as_ref() {
            Some(value) => {
                let variant_type = self.variant_type();
                let value = converter.process(
                    value,
                    variant_type,
                )?
                    .into_struct()
                    .expect("variant value must be convertible to struct");

                Ok(Variant {
                    type_index: variant_type.index,
                    value,
                }.into())
            }
            None => {
                Ok(Value::from(self.data.clone().into())?)
            }
        }
    }

    pub fn try_clone_lua(
        &self,
        lua: &mlua::Lua,
    ) -> mlua::Result<Self> {
        Ok(Self {
            data: self.data.clone(),
            value: match self.value.borrow().as_ref() {
                Some(value) => RefCell::new(Some(try_clone_lua_value(value, lua)?)),
                None => RefCell::new(None),
            }
        })
    }

    pub fn is(
        &self,
        r#type: &TypeMetadata,
    ) -> bool {
        self.r#type().deref() == r#type
    }

}

impl UserData for MappedVariantValue {

    fn add_fields<F: mlua::UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("type", |_, this| Ok(this.variant_type().qualified_name.clone()));
        fields.add_field_method_get("value", |lua, this| {
            let mut value_slot = this.value.borrow_mut();

            match value_slot.as_ref() {
                Some(value) => Ok(value.clone()),
                None => {
                    let value = value_to_lua(&this.data.value().clone().into(), lua)?;

                    *value_slot = Some(value.clone());

                    Ok(value)
                }
            }
        });

        fields.add_field_method_set("value", |lua, this, value: LuaValue| {
            let value = validate_and_clone_lua_value(&value, this.variant_type(), lua)
                .map_err(LuaError::external)?;

            this.value.replace(Some(value));

            Ok(())
        });
    }

}
