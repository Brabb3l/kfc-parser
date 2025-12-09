use std::{cell::RefCell, collections::HashMap, ops::Deref, rc::Rc};

use indexmap::IndexMap;
use kfc::{resource::value::{Value, Variant}, reflection::{PrimitiveType, TypeMetadata}};
use mlua::{AnyUserData, Function, MetaMethod, UserData};
use ouroboros::self_referencing;

use crate::{alias::TypeHandle, env::value::{converter::{Converter, LuaConversionErrorKind}, name_of, validate_and_clone_lua_value, validator::try_clone_lua_value}, lua::{FunctionArgs, LuaError, LuaValue, MethodArgs}};

pub struct StructValue {
    r#type: TypeHandle,
    fields: Rc<RefCell<HashMap<String, LuaValue>>>,
}

impl StructValue {

    /// Creates a new `StructValue` with the given type and a pre-populated map.
    ///
    /// # Contract
    ///
    /// The provided `data` must adhere to the type's structure,
    /// otherwise it will lead to unexpected behavior.
    #[inline]
    pub fn new_with_data(
        r#type: TypeHandle,
        map: HashMap<String, LuaValue>,
    ) -> mlua::Result<Self> {
        Ok(Self {
            r#type,
            fields: Rc::new(RefCell::new(map)),
        })
    }

    #[inline]
    pub fn r#type(&self) -> &TypeHandle {
        &self.r#type
    }

    pub fn convert(
        &self,
        converter: &mut Converter,
    ) -> Result<Value, LuaConversionErrorKind> {
        let fields = self.fields.borrow();
        let mut result = IndexMap::with_capacity(fields.len());

        for (key, value) in fields.iter() {
            converter.path.push(key);

            let field_type = self.r#type.get_field_type(key)
                .expect("field type must be present");
            let converted_value = converter.process(
                value,
                &field_type,
            )?;

            result.insert(key.clone(), converted_value);

            converter.path.pop();
        }

        Ok(result.into())
    }

    pub fn try_clone_lua(
        &self,
        lua: &mlua::Lua,
    ) -> mlua::Result<Self> {
        let table = self.fields.borrow();
        let mut fields = HashMap::with_capacity(table.len());

        for (key, value) in table.iter() {
            fields.insert(
                key.clone(),
                try_clone_lua_value(value, lua)?
            );
        }

        Ok(Self {
            r#type: self.r#type.clone(),
            fields: Rc::new(RefCell::new(fields)),
        })
    }

    pub fn is(
        &self,
        r#type: &TypeMetadata,
    ) -> bool {
        self.r#type.deref() == r#type
    }

    pub fn get(
        &self,
        key: String,
    ) -> mlua::Result<LuaValue> {
        Ok(self.fields.borrow().get(&key).cloned().unwrap_or(LuaValue::Nil))
    }

    pub fn set(
        &self,
        key: String,
        value: &LuaValue,
        lua: &mlua::Lua,
    ) -> mlua::Result<()> {
        if !self.fields.borrow().contains_key(&key) {
            return Err(LuaError::generic(format!(
                "attempt to set invalid property '{key}' on {}",
                name_of(&self.r#type)
            )));
        }

        let value = validate_and_clone_lua_value(value, &self.r#type, lua)
            .map_err(LuaError::external)?;

        self.fields.borrow_mut().insert(key, value);

        Ok(())
    }

    pub fn is_dirty(&self) -> mlua::Result<bool> {
        Ok(true)
    }

}

impl UserData for StructValue {

    fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
        methods.add_meta_function(MetaMethod::Len, |_, args: MethodArgs| {
            let this = args.this::<&Self>()?;
            Ok(this.fields.borrow().len())
        });

        methods.add_meta_function(MetaMethod::Index, |_, args: MethodArgs| {
            let this = args.this::<&Self>()?;
            let key = args.get::<String>(0)?;

            this.get(key)
        });

        methods.add_meta_function(MetaMethod::NewIndex, |lua, args: MethodArgs| {
            let this = args.this::<&Self>()?;
            let key = args.get::<String>(0)?;
            let value = args.get::<LuaValue>(1)?;

            this.set(key, value, lua)
        });

        methods.add_meta_function(MetaMethod::Pairs, |lua, args: MethodArgs| {
            let this = args.this::<&Self>()?;
            StructValueIter::create(&this, lua)
        });

        methods.add_meta_function(MetaMethod::ToString, |lua, args: MethodArgs| {
            let this = args.this::<&Self>()?;
            let values = this.fields.borrow();
            let mut result = String::new();

            result.push('{');

            for (i, (key, value)) in values.iter().enumerate() {
                if i > 0 {
                    result.push(',');
                }

                result.push('"');
                result.push_str(key);
                result.push_str("\":");
                // Add quotes to value if it is a string
                if value.is_string() {
                    result.push('"');
                    (result).push_str(&value.to_string()?);
                    result.push('"');
                }
                else {
                    (result).push_str(&value.to_string()?);
                }
            }

            result.push('}');

            Ok(LuaValue::String(lua.create_string(result)?))
        });
    }

}

#[self_referencing]
struct StructValueIter {
    r#type: TypeHandle,
    fields: Rc<RefCell<HashMap<String, LuaValue>>>,
    #[borrows(r#type)]
    #[not_covariant]
    keys: RefCell<Box<dyn Iterator<Item = &'this str> + 'this>>,
}

impl StructValueIter {

    fn create(
        value: &StructValue,
        lua: &mlua::Lua,
    ) -> mlua::Result<(Function, AnyUserData, LuaValue)> {
        let state = Self::new(
            value.r#type.clone(),
            value.fields.clone(),
            |r#type| {
                let iter = r#type.iter_fields()
                    .map(|field| field.name.as_str());

                RefCell::new(Box::new(iter))
            }
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

        let value = self.with_fields(|fields| {
            fields.borrow()
                .get(key)
                .cloned()
                .unwrap_or(LuaValue::Nil)
        });

        Ok((LuaValue::String(lua.create_string(key)?), value))
    }

}

impl UserData for StructValueIter {}

pub struct ArrayValue {
    r#type: TypeHandle,
    inner_type: TypeHandle,
    values: Rc<RefCell<Vec<LuaValue>>>,
}

impl ArrayValue {

    /// Creates a new `ArrayValue` with the given type and a pre-populated array.
    ///
    /// # Contract
    ///
    /// The provided `data` must adhere to the type's structure,
    /// otherwise it will lead to unexpected behavior.
    #[inline]
    pub fn new_with_data(
        r#type: TypeHandle,
        data: Vec<LuaValue>,
    ) -> mlua::Result<Self> {
        Ok(Self {
            inner_type: r#type.inner_type()
                .expect("array type must have an inner type"),
            r#type,
            values: Rc::new(RefCell::new(data)),
        })
    }

    #[inline]
    pub fn r#type(&self) -> &TypeHandle {
        &self.r#type
    }

    pub fn convert(
        &self,
        converter: &mut Converter,
    ) -> Result<Value, LuaConversionErrorKind> {
        let values = self.values.borrow();
        let mut result = Vec::with_capacity(values.len());

        let field_type = self.r#type.inner_type()
            .expect("array type must have an inner type");

        for (i, value) in values.iter().enumerate() {
            converter.path.push_index(i);

            let converted_value = converter.process(
                value,
                &field_type,
            )?;
            result.push(converted_value);

            converter.path.pop();
        }

        Ok(result.into())
    }

    pub fn try_clone_lua(
        &self,
        lua: &mlua::Lua,
    ) -> mlua::Result<Self> {
        let values = self.values.borrow();
        let mut result = Vec::with_capacity(values.len());

        for value in values.iter() {
            result.push(try_clone_lua_value(value, lua)?);
        }

        Ok(Self {
            r#type: self.r#type.clone(),
            inner_type: self.inner_type.clone(),
            values: Rc::new(RefCell::new(result)),
        })
    }

    pub fn is(
        &self,
        r#type: &TypeMetadata,
    ) -> bool {
        self.r#type.deref() == r#type
    }

    pub fn len(&self) -> usize {
        self.values.borrow().len()
    }

    pub fn get(
        &self,
        index: usize,
    ) -> mlua::Result<LuaValue> {
        if index == 0 {
            return Ok(LuaValue::Nil);
        }

        let value = self.values.borrow().get(index - 1)
            .cloned()
            .unwrap_or(LuaValue::Nil);

        Ok(value)
    }

    pub fn set(
        &self,
        index: usize,
        value: &LuaValue,
        lua: &mlua::Lua,
    ) -> mlua::Result<()> {
        let index = self.check_range(index, 1, self.len())?;

        let value = validate_and_clone_lua_value(value, &self.r#type, lua)
            .map_err(LuaError::external)?;

        self.values.borrow_mut()[index] = value;

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

        let value = validate_and_clone_lua_value(value, &self.r#type, lua)
            .map_err(LuaError::external)?;

        self.values.borrow_mut().insert(index, value);

        Ok(())
    }

    pub fn remove(
        &mut self,
        index: usize,
    ) -> mlua::Result<Option<LuaValue>> {
        if index == 0 {
            return Ok(None);
        }

        let index = self.check_range(index, 1, self.len() + 1)?;
        self.check_static()?;

        if index >= self.values.borrow().len() {
            return Ok(None);
        }

        Ok(Some(self.values.borrow_mut().remove(index)))
    }

    pub fn clear(&mut self) -> mlua::Result<()> {
        self.check_static()?;

        self.values.borrow_mut().clear();

        Ok(())
    }

    pub fn is_dirty(&self) -> mlua::Result<bool> {
        Ok(true)
    }

    fn check_range(
        &self,
        index: usize,
        start: usize,
        end: usize,
    ) -> mlua::Result<usize> {
        if index == 0 || index > end {
            Err(LuaError::position_out_of_bounds(
                index,
                start,
                end,
            ))
        } else {
            Ok(index - 1)
        }
    }

    fn check_static(
        &self,
    ) -> mlua::Result<()> {
        if self.r#type.primitive_type == PrimitiveType::StaticArray {
            Err(LuaError::generic("attempt to modify static array"))
        } else {
            Ok(())
        }
    }

}

impl UserData for ArrayValue {

    fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
        methods.add_meta_function(MetaMethod::Len, |_, args: MethodArgs| {
            let this = args.this::<&Self>()?;
            Ok(this.values.borrow().len())
        });

        methods.add_meta_function(MetaMethod::Index, |_, args: MethodArgs| {
            let this = args.this::<&Self>()?;
            let key = args.get::<usize>(0)?;

            this.get(key)
        });

        methods.add_meta_function(MetaMethod::NewIndex, |lua, args: MethodArgs| {
            let this = args.this::<&Self>()?;
            let index = args.get::<usize>(0)?;
            let value = args.get::<LuaValue>(1)?;

            this.set(index, value, lua)
        });

        methods.add_meta_function(MetaMethod::Pairs, |lua, args: MethodArgs| {
            let this = args.this::<&Self>()?;
            ArrayValueIter::create(&this, lua)
        });

        methods.add_meta_function(MetaMethod::ToString, |lua, args: MethodArgs| {
            let this = args.this::<&Self>()?;
            let values = this.values.borrow();
            let mut result = String::new();

            result.push('[');

            for (i, value) in values.iter().enumerate() {
                if i > 0 {
                    result.push(',');
                }

                // Add quotes to value if it is a string
                if value.is_string() {
                    result.push('"');
                    (result).push_str(&value.to_string()?);
                    result.push('"');
                }
                else {
                    (result).push_str(&value.to_string()?);
                }
            }

            result.push(']');

            Ok(LuaValue::String(lua.create_string(result)?))
        });
    }

}

struct ArrayValueIter {
    values: Rc<RefCell<Vec<LuaValue>>>,
    index: usize,
}

impl ArrayValueIter {

    fn create(
        value: &ArrayValue,
        lua: &mlua::Lua,
    ) -> mlua::Result<(Function, AnyUserData, LuaValue)> {
        let state = Self {
            values: value.values.clone(),
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
        if self.index >= self.values.borrow().len() {
            return Ok((LuaValue::Nil, LuaValue::Nil));
        }

        let key = self.index;
        let value = self.values.borrow()[key].clone();

        self.index += 1;

        Ok((LuaValue::Integer(key as i64 + 1), value))
    }

}

impl UserData for ArrayValueIter {}

pub struct VariantValue {
    r#type: TypeHandle,
    variant_type: TypeHandle,
    value: LuaValue,
}

impl VariantValue {

    /// Creates a new `VariantValue` with the given types and a value.
    ///
    /// # Contract
    ///
    /// The provided `data` must adhere to the type's structure,
    /// otherwise it will lead to unexpected behavior.
    #[inline]
    pub fn new_with_data(
        r#type: TypeHandle,
        variant_type: TypeHandle,
        value: LuaValue,
    ) -> mlua::Result<Self> {
        Ok(Self {
            r#type,
            variant_type,
            value,
        })
    }

    #[inline]
    pub fn r#type(&self) -> &TypeHandle {
        &self.r#type
    }

    #[inline]
    pub fn variant_type(&self) -> &TypeHandle {
        &self.variant_type
    }

    pub fn convert(
        &self,
        converter: &mut Converter,
    ) -> Result<Value, LuaConversionErrorKind> {
        let value = converter.process(
            &self.value,
            &self.variant_type,
        )?
            .into_struct()
            .expect("variant value must be convertible to struct");

        Ok(Variant {
            type_index: self.variant_type.index,
            value,
        }.into())
    }

    pub fn try_clone_lua(
        &self,
        lua: &mlua::Lua,
    ) -> mlua::Result<Self> {
        Ok(Self {
            r#type: self.r#type.clone(),
            variant_type: self.variant_type.clone(),
            value: try_clone_lua_value(&self.value, lua)?,
        })
    }

    pub fn is(
        &self,
        r#type: &TypeMetadata,
    ) -> bool {
        self.r#type.deref() == r#type
    }

    pub fn is_dirty(&self) -> mlua::Result<bool> {
        Ok(true)
    }

}

impl UserData for VariantValue {

    fn add_fields<F: mlua::UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("type", |_, this| Ok(this.variant_type().qualified_name.clone()));
        fields.add_field_method_get("value", |_, this| Ok(this.value.clone()));

        fields.add_field_method_set("value", |lua, this, value: LuaValue| {
            this.value = validate_and_clone_lua_value(&value, &this.variant_type, lua)
                .map_err(LuaError::external)?;

            Ok(())
        });
    }

}
