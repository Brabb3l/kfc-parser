use std::ops::Deref;

use kfc::reflection::{Attribute, EnumFieldMetadata, LookupKey, PrimitiveType, StructFieldMetadata, TypeFlags, TypeMetadata};
use mlua::{IntoLua, Table, UserData};
use once_cell::unsync::OnceCell;
use indexmap::IndexMap;

use crate::{alias::TypeHandle, env::{util::add_function, value::type_of, AppState}, lua::{Either, FunctionArgs, LuaValue}, util::{ReadOnlyArray, ReadOnlyMap}};

pub fn create(
    lua: &mlua::Lua
) -> mlua::Result<mlua::Table> {
    let table = lua.create_table()?;

    add_function(lua, &table, "get", lua_get)?;
    add_function(lua, &table, "get_by_qualified_hash", lua_get_by_qualified_hash)?;
    add_function(lua, &table, "get_by_impact_hash", lua_get_by_impact_hash)?;
    add_function(lua, &table, "get_by_qualified_name", lua_get_by_qualified_name)?;
    add_function(lua, &table, "get_by_impact_name", lua_get_by_impact_name)?;
    add_function(lua, &table, "get_all", lua_get_all)?;
    add_function(lua, &table, "of", lua_of)?;

    Ok(table)
}

fn lua_get(
    lua: &mlua::Lua,
    args: FunctionArgs,
) -> mlua::Result<Option<LuaValue>> {
    let hash_or_name = args.get::<Either<u32, String>>(0)?;

    match hash_or_name {
        Either::A(hash) => get_by_hash(lua, LookupKey::Qualified(hash)),
        Either::B(name) => get_by_name(lua, LookupKey::Qualified(name.as_ref())),
    }
}

fn lua_get_by_qualified_hash(
    lua: &mlua::Lua,
    args: FunctionArgs,
) -> mlua::Result<Option<LuaValue>> {
    let hash = args.get::<u32>(0)?;
    get_by_hash(lua, LookupKey::Qualified(hash))
}

fn lua_get_by_impact_hash(
    lua: &mlua::Lua,
    args: FunctionArgs,
) -> mlua::Result<Option<LuaValue>> {
    let hash = args.get::<u32>(0)?;
    get_by_hash(lua, LookupKey::Impact(hash))
}

fn lua_get_by_qualified_name(
    lua: &mlua::Lua,
    args: FunctionArgs,
) -> mlua::Result<Option<LuaValue>> {
    let name = args.get::<String>(0)?;
    get_by_name(lua, LookupKey::Qualified(&name))
}

fn lua_get_by_impact_name(
    lua: &mlua::Lua,
    args: FunctionArgs,
) -> mlua::Result<Option<LuaValue>> {
    let name = args.get::<String>(0)?;
    get_by_name(lua, LookupKey::Impact(&name))
}

fn lua_get_all(
    lua: &mlua::Lua,
    _args: FunctionArgs,
) -> mlua::Result<Table> {
    let app_state = lua.app_data_ref::<AppState>().unwrap();
    let result = lua.create_table_with_capacity(app_state.type_registry().len(), 0)?;

    for r#type in app_state.type_registry().iter() {
        if r#type.flags.contains(TypeFlags::HAS_DS) {
            continue;
        }

        let value = app_state.get_type(lua, r#type.index)?
            .expect("invalid type index");

        result.push(value)?;
    }

    Ok(result)
}

fn lua_of(
    lua: &mlua::Lua,
    args: FunctionArgs,
) -> mlua::Result<LuaValue> {
    let app_state = lua.app_data_ref::<AppState>().unwrap();

    let value = args.get::<LuaValue>(0)?;

    let index = match type_of(value)? {
        Some(index) => index,
        None => return Ok(LuaValue::Nil),
    };

    Ok(app_state.get_type(lua, index)?
        .expect("invalid type index"))
}

fn get_by_hash(
    lua: &mlua::Lua,
    key: LookupKey<u32>,
) -> mlua::Result<Option<LuaValue>> {
    let app_state = lua.app_data_ref::<AppState>().unwrap();
    let type_index = app_state.type_registry()
        .get_by_hash(key)
        .map(|t| t.index);

    let type_index = match type_index {
        Some(index) => index,
        None => return Ok(None),
    };

    app_state.get_type(lua, type_index)
}

fn get_by_name(
    lua: &mlua::Lua,
    name: LookupKey<&str>,
) -> mlua::Result<Option<LuaValue>> {
    let context = lua.app_data_ref::<AppState>().unwrap();
    let type_index = context.type_registry()
        .get_by_name(name)
        .map(|t| t.index);

    let type_index = match type_index {
        Some(index) => index,
        None => return Ok(None),
    };

    context.get_type(lua, type_index)
}

pub struct Type {
    handle: TypeHandle,

    struct_field_cache: OnceCell<LuaValue>,
    enum_field_cache: OnceCell<LuaValue>,
    attribute_cache: OnceCell<LuaValue>,
}

impl Type {

    pub fn new(
        handle: TypeHandle,
    ) -> Self {
        Self {
            handle,

            struct_field_cache: OnceCell::new(),
            enum_field_cache: OnceCell::new(),
            attribute_cache: OnceCell::new(),
        }
    }

}

impl Deref for Type {
    type Target = TypeMetadata;

    fn deref(&self) -> &Self::Target {
        self.handle.deref()
    }
}

impl UserData for Type {

    fn add_fields<F: mlua::UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("name", |_, this| Ok(this.name.clone()));
        fields.add_field_method_get("impact_name", |_, this| Ok(this.impact_name.clone()));
        fields.add_field_method_get("qualified_name", |_, this| Ok(this.qualified_name.clone()));

        fields.add_field_method_get("namespace", |_, this| Ok(this.namespace.clone()));
        fields.add_field_method_get("inner_type", |lua, this| {
            this.inner_type
                .as_ref()
                .map(|inner_type| lua.app_data_ref::<AppState>()
                    .unwrap()
                    .get_type(lua, *inner_type))
                .transpose()
        });
        fields.add_field_method_get("size", |_, this| Ok(this.size));
        fields.add_field_method_get("alignment", |_, this| Ok(this.alignment));
        fields.add_field_method_get("element_alignment", |_, this| Ok(this.element_alignment));
        fields.add_field_method_get("field_count", |_, this| Ok(this.field_count));
        fields.add_field_method_get("primitive_type", |_, this| {
            Ok(primitive_type_to_string(this.primitive_type))
        });
        // fields.add_field_method_get("flags", |_, this| Ok(this.flags));

        fields.add_field_method_get("name_hash", |_, this| Ok(this.name_hash));
        fields.add_field_method_get("impact_hash", |_, this| Ok(this.impact_hash));
        fields.add_field_method_get("qualified_hash", |_, this| Ok(this.qualified_hash));
        fields.add_field_method_get("internal_hash", |_, this| Ok(this.internal_hash));

        fields.add_field_method_get("struct_fields", |lua, this| {
            this.struct_field_cache.get_or_try_init(|| {
                struct_fields_to_lua(lua, &lua.app_data_ref().unwrap(), this)
            }).cloned()
        });
        fields.add_field_method_get("enum_fields", |lua, this| {
            this.enum_field_cache.get_or_try_init(|| {
                enum_fields_to_lua(lua, this)
            }).cloned()
        });
        // fields.add_field_method_get("default_value", |_, this| { });
        fields.add_field_method_get("attributes", |lua, this| {
            this.attribute_cache.get_or_try_init(|| {
                attributes_to_lua(lua, &lua.app_data_ref().unwrap(), &this.attributes)
            }).cloned()
        });
    }

}

fn primitive_type_to_string(primitive_type: PrimitiveType) -> &'static str {
    match primitive_type {
        PrimitiveType::None => "None",
        PrimitiveType::Bool => "Bool",
        PrimitiveType::UInt8 => "UInt8",
        PrimitiveType::SInt8 => "SInt8",
        PrimitiveType::UInt16 => "UInt16",
        PrimitiveType::SInt16 => "SInt16",
        PrimitiveType::UInt32 => "UInt32",
        PrimitiveType::SInt32 => "SInt32",
        PrimitiveType::UInt64 => "UInt64",
        PrimitiveType::SInt64 => "SInt64",
        PrimitiveType::Float32 => "Float32",
        PrimitiveType::Float64 => "Float64",
        PrimitiveType::Enum => "Enum",
        PrimitiveType::Bitmask8 => "Bitmask8",
        PrimitiveType::Bitmask16 => "Bitmask16",
        PrimitiveType::Bitmask32 => "Bitmask32",
        PrimitiveType::Bitmask64 => "Bitmask64",
        PrimitiveType::Typedef => "Typedef",
        PrimitiveType::Struct => "Struct",
        PrimitiveType::StaticArray => "StaticArray",
        PrimitiveType::DsArray => "DsArray",
        PrimitiveType::DsString => "DsString",
        PrimitiveType::DsOptional => "DsOptional",
        PrimitiveType::DsVariant => "DsVariant",
        PrimitiveType::BlobArray => "BlobArray",
        PrimitiveType::BlobString => "BlobString",
        PrimitiveType::BlobOptional => "BlobOptional",
        PrimitiveType::BlobVariant => "BlobVariant",
        PrimitiveType::ObjectReference => "ObjectReference",
        PrimitiveType::Guid => "Guid",
    }
}

fn struct_fields_to_lua(
    lua: &mlua::Lua,
    context: &AppState,
    this: &Type,
) -> mlua::Result<LuaValue> {
    let mut map = IndexMap::with_capacity(this.struct_fields.len());

    for (name, field) in &this.struct_fields {
        let key = lua.create_string(name)?;
        let value = struct_field_to_lua(lua, context, field)?;

        map.insert(key, value);
    }

    ReadOnlyMap::new(map).into_lua(lua)
}

fn struct_field_to_lua(
    lua: &mlua::Lua,
    context: &AppState,
    field: &StructFieldMetadata,
) -> mlua::Result<LuaValue> {
    let mut map = IndexMap::with_capacity(4);

    let name_key = lua.create_string("name")?;
    let type_key = lua.create_string("type")?;
    let data_offset_key = lua.create_string("data_offset")?;
    let attributes_key = lua.create_string("attributes")?;

    let name = LuaValue::String(lua.create_string(&field.name)?);
    let r#type = context.get_type(lua, field.r#type)?.unwrap_or(LuaValue::Nil);
    let data_offset = LuaValue::Integer(field.data_offset as i64);
    let attributes = attributes_to_lua(lua, context, &field.attributes)?;

    map.insert(name_key, name);
    map.insert(type_key, r#type);
    map.insert(data_offset_key, data_offset);
    map.insert(attributes_key, attributes);

    ReadOnlyMap::new(map).into_lua(lua)
}

fn attributes_to_lua(
    lua: &mlua::Lua,
    context: &AppState,
    attributes: &IndexMap<String, Attribute>,
) -> mlua::Result<LuaValue> {
    let mut map = IndexMap::with_capacity(attributes.len());

    for (name, attribute) in attributes {
        let key = lua.create_string(name)?;
        let value = attribute_to_lua(lua, context, attribute)?;

        map.insert(key, value);
    }

    ReadOnlyMap::new(map).into_lua(lua)
}

fn attribute_to_lua(
    lua: &mlua::Lua,
    context: &AppState,
    attribute: &Attribute,
) -> mlua::Result<LuaValue> {
    let mut map = IndexMap::with_capacity(4);

    let name_key = lua.create_string("name")?;
    let namespace_key = lua.create_string("namespace")?;
    let type_key = lua.create_string("type")?;
    let value_key = lua.create_string("value")?;

    let name = LuaValue::String(lua.create_string(&attribute.name)?);
    let namespace = namespace_to_lua(lua, &attribute.namespace)?;
    let r#type = attribute.r#type
        .map(|r#type| context.get_type(lua, r#type))
        .transpose()?
        .and_then(|t| t)
        .unwrap_or(LuaValue::Nil);
    let value = LuaValue::String(lua.create_string(&attribute.value)?);

    map.insert(name_key, name);
    map.insert(namespace_key, namespace);
    map.insert(type_key, r#type);
    map.insert(value_key, value);

    ReadOnlyMap::new(map).into_lua(lua)
}

fn namespace_to_lua(
    lua: &mlua::Lua,
    namespace: &Vec<String>,
) -> mlua::Result<LuaValue> {
    let mut array = Vec::with_capacity(namespace.len());

    for name in namespace {
        array.push(LuaValue::String(lua.create_string(name)?));
    }

    ReadOnlyArray::new(array).into_lua(lua)
}

fn enum_fields_to_lua(
    lua: &mlua::Lua,
    this: &Type,
) -> mlua::Result<LuaValue> {
    let mut map = IndexMap::with_capacity(this.enum_fields.len());

    for (name, field) in &this.enum_fields {
        let key = lua.create_string(name)?;
        let value = enum_field_to_lua(lua, field)?;

        map.insert(key, value);
    }

    ReadOnlyMap::new(map).into_lua(lua)
}

fn enum_field_to_lua(
    lua: &mlua::Lua,
    field: &EnumFieldMetadata,
) -> mlua::Result<LuaValue> {
    let mut map = IndexMap::with_capacity(2);

    let name_key = lua.create_string("name")?;
    let value_key = lua.create_string("value")?;

    let name = LuaValue::String(lua.create_string(&field.name)?);
    let value = LuaValue::Integer(field.value as i64);

    map.insert(name_key, name);
    map.insert(value_key, value);

    ReadOnlyMap::new(map).into_lua(lua)
}
