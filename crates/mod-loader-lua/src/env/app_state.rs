use std::{cell::{RefCell, RefMut}, collections::{hash_map::{self, Entry}, HashMap}, rc::Rc};

use bitflags::bitflags;
use mod_loader::ModEnvironment;
use once_cell::unsync::OnceCell;
use kfc::{container::{KFCCursor, KFCFile, KFCReader, KFCWriter}, guid::{ContentHash, ResourceId}, reflection::{LookupKey, TypeHandle, TypeIndex, TypeRegistry}, resource::value::Value};

use crate::{alias::{MappedValue, PathBuf}, cache::CacheDiff, env::{value::{convert_lua_to_value, convert_value_to_lua, validate_and_clone_lua_value}, Type}, log::warn, lua::{LuaError, LuaValue}, RunArgs};

pub struct AppState {
    env: ModEnvironment,
    config: AppConfig,

    type_registry: Rc<TypeRegistry>,

    ref_file: Rc<KFCFile>,
    reader: RefCell<KFCCursor<KFCReader>>,
    writer: RefCell<Option<KFCWriter<Rc<KFCFile>, Rc<TypeRegistry>>>>,

    // NOTE: the `Vec<u8>` must always be treated as immutable
    new_contents: RefCell<HashMap<ContentHash, Vec<u8>>>,
    resources: RefCell<HashMap<ResourceId, Rc<ResourceInfo>>>,
    types: RefCell<HashMap<TypeIndex, LuaValue>>,
}

pub struct AppConfig {
    skip_cache: bool,
    is_server: bool,

    feature_flags: AppFeatures,
    export_dir: PathBuf,
}

bitflags! {
    pub struct AppFeatures: u32 {
        const PATCH = 1 << 0;
        const EXPORT = 1 << 1;

        const ALL = !0;
    }
}

impl AppState {

    pub fn new(
        env: ModEnvironment,
        args: RunArgs,
        cache_diff: &CacheDiff,
    ) -> Result<Self, ()> {
        let game_dir = env.game_dir();
        let cache_dir = env.cache_dir();
        let file_name = &args.file_name;

        // prepare configuration

        let options = args.options;
        let export_dir = options.export_dir
            .map(|d| d.canonicalize())
            .transpose()
            .map_err(|e| {
                warn!(
                    error = %e,
                    "Failed to canonicalize export directory, using default instead"
                );
            })?
            .map(|d| {
                PathBuf::from_path_buf(d)
            })
            .transpose()
            .map_err(|_| {
                warn!(
                    "Export directory is not valid UTF-8, using default instead"
                );
            })?
            .unwrap_or_else(|| env.game_dir().join("export"));

        let skip_cache = options.skip_cache;
        let is_server = options.is_server
            .unwrap_or_else(|| file_name.to_lowercase().contains("server"));

        let mut feature_flags = AppFeatures::empty();

        if options.patch {
            feature_flags |= AppFeatures::PATCH;
        }

        if options.export {
            feature_flags |= AppFeatures::EXPORT;
        }

        let config = AppConfig {
            skip_cache,
            export_dir,

            is_server,

            feature_flags,
        };

        // load and attach to resources

        let (type_registry, is_dirty) = crate::load::load_type_registry(
            game_dir,
            cache_dir,
            file_name,
        )?;
        crate::load::export_lua_definitions(
            cache_dir,
            &type_registry,
            options.skip_cache || is_dirty || cache_diff.build_id_changed(),
        );
        let type_registry = Rc::new(type_registry);

        let kfc_path = game_dir
            .join(file_name)
            .with_extension("kfc");
        let bak_path = crate::load::create_backup(&kfc_path)?;

        let ref_file = crate::load::load_kfc_file(&bak_path)?;
        let reader = crate::load::create_reader(game_dir, file_name)?;
        let writer = crate::load::create_writer(game_dir, file_name, &type_registry, &ref_file)?;

        Ok(Self {
            env,
            config,

            type_registry,

            ref_file,
            reader: RefCell::new(reader),
            writer: RefCell::new(Some(writer)),

            new_contents: RefCell::new(HashMap::new()),
            resources: RefCell::new(HashMap::new()),
            types: RefCell::new(HashMap::new()),
        })
    }

    #[inline]
    pub fn env(&self) -> &ModEnvironment {
        &self.env
    }

    #[inline]
    #[allow(unused)]
    pub fn skip_cache(&self) -> bool {
        self.config.skip_cache
    }

    #[inline]
    pub fn is_server(&self) -> bool {
        self.config.is_server
    }

    #[inline]
    pub fn is_client(&self) -> bool {
        !self.config.is_server
    }

    #[inline]
    pub fn export_dir(&self) -> &PathBuf {
        &self.config.export_dir
    }

    #[inline]
    pub fn has_feature(&self, feature: AppFeatures) -> bool {
        self.config.feature_flags.contains(feature)
    }

    #[inline]
    pub fn kfc_file(&self) -> &KFCFile {
        &self.ref_file
    }

    #[inline]
    pub fn type_registry(&self) -> &Rc<TypeRegistry> {
        &self.type_registry
    }

    #[inline]
    pub fn reader(&self) -> RefMut<KFCCursor<KFCReader>> {
        self.reader.borrow_mut()
    }

    #[inline]
    pub fn take_writer(
        &self,
    ) -> KFCWriter<Rc<KFCFile>, Rc<TypeRegistry>> {
        self.writer.borrow_mut()
            .take()
            .expect("KFCWriter is no longer available")
    }

    pub fn get_cached_resources(
        &self,
    ) -> Vec<Rc<ResourceInfo>> {
        self.resources.borrow()
            .values()
            .cloned()
            .collect()
    }

    pub fn get_resource_info(
        &self,
        guid: &ResourceId,
    ) -> Option<Rc<ResourceInfo>> {
        match self.resources.borrow_mut().entry(*guid) {
            Entry::Occupied(entry) => {
                Some(entry.get().clone())
            },
            Entry::Vacant(entry) => {
                if !self.ref_file.resources().contains_key(guid) {
                    return None;
                }

                let info = ResourceInfo {
                    guid: *guid,
                    original_value: OnceCell::new(),
                    value: RefCell::default(),
                };
                let info = Rc::new(info);

                entry.insert(info.clone());

                Some(info)
            }
        }
    }

    pub fn add_resource(
        &self,
        value: &LuaValue,
        guid: &ResourceId,
        lua: &mlua::Lua,
    ) -> mlua::Result<()> {
        let mut resources = self.resources.borrow_mut();

        if resources.contains_key(guid) {
            return Err(LuaError::generic(format!(
                "resource with GUID {guid} already exists"
            )));
        }

        let r#type = TypeHandle::new(
            self.type_registry.clone(),
            self.type_registry.get_by_hash(LookupKey::Qualified(guid.type_hash()))
                .ok_or_else(|| LuaError::type_not_found(guid.type_hash()))?
                .index,
        );
        let value = validate_and_clone_lua_value(
            value,
            &r#type,
            lua,
        ).map_err(LuaError::external)?;

        let info = Rc::new(ResourceInfo {
            guid: *guid,
            original_value: OnceCell::new(),
            value: RefCell::new(Some(value)),
        });

        resources.insert(*guid, info);

        Ok(())
    }

    pub fn create_resource(
        &self,
        value: &LuaValue,
        type_index: TypeIndex,
        lua: &mlua::Lua,
    ) -> mlua::Result<ResourceId> {
        let r#type = self.type_registry.get(type_index).unwrap();
        let resources = self.resources.borrow_mut();
        let mut guid: ResourceId;

        loop {
            let uuid = uuid::Uuid::new_v4().into_bytes();

            guid = ResourceId::new(
                uuid.into(),
                r#type.qualified_hash,
                0,
            );

            if !resources.contains_key(&guid) {
                break;
            }
        }

        drop(resources);

        self.add_resource(value, &guid, lua)?;

        Ok(guid)
    }

    pub fn get_content(
        &self,
        guid: &ContentHash,
    ) -> mlua::Result<Option<Vec<u8>>> {
        if let Some(value) = self.new_contents.borrow().get(guid) {
            // OPTIMIZE: cloning here can be quite expensive
            Ok(Some(value.clone()))
        } else {
            // OPTIMIZE: consider using a reader for this
            Ok(self.reader.borrow_mut().read_content(guid)?)
        }
    }

    pub fn create_content(
        &self,
        data: &[u8],
    ) -> mlua::Result<ContentHash> {
        let guid = ContentHash::from_data(data);

        if let hash_map::Entry::Vacant(e) = self.new_contents.borrow_mut().entry(guid) {
            self.writer.borrow_mut()
                .as_mut()
                .expect("KFCWriter no longer available")
                .write_content(&guid, data)?;

            // TODO: attach a reader to the writer to avoid cloning
            e.insert(data.to_vec());
        }

        Ok(guid)
    }

    pub fn get_type(
        &self,
        lua: &mlua::Lua,
        type_index: TypeIndex,
    ) -> mlua::Result<Option<LuaValue>> {
        match self.types.borrow_mut().entry(type_index) {
            Entry::Occupied(entry) => Ok(Some(entry.get().clone())),
            Entry::Vacant(entry) => {
                if self.type_registry.get(type_index).is_none() {
                    return Ok(None);
                }

                let value = Type::new(TypeHandle::new(
                    self.type_registry.clone(),
                    type_index,
                ));
                let value = LuaValue::UserData(lua.create_userdata(value)?);

                entry.insert(value.clone());
                Ok(Some(value))
            }
        }
    }

}

pub struct ResourceInfo {
    pub guid: ResourceId,

    original_value: OnceCell<Option<MappedValue>>,
    value: RefCell<Option<LuaValue>>,
}

impl ResourceInfo {

    pub fn get_lua_value(
        &self,
        lua: &mlua::Lua,
    ) -> mlua::Result<LuaValue> {
        if let Some(value) = self.value.borrow().as_ref() {
            return Ok(value.clone());
        }

        let value = match self.get_mapped_value(lua)? {
            Some(value) => convert_value_to_lua(value, lua)?,
            None => LuaValue::Nil,
        };

        self.value.replace(Some(value.clone()));

        Ok(value)
    }

    pub fn set_lua_value(
        &self,
        lua_value: LuaValue,
        lua: &mlua::Lua,
    ) -> mlua::Result<()> {
        let app_state = lua.app_data_ref::<AppState>().unwrap();
        let type_registry = app_state.type_registry();

        let r#type = type_registry
            .get_by_hash(LookupKey::Qualified(self.guid.type_hash()))
            .ok_or_else(|| LuaError::type_not_found(self.guid.type_hash()))?;

        let lua_value = validate_and_clone_lua_value(
            &lua_value,
            &TypeHandle::new(
                type_registry.clone(),
                r#type.index,
            ),
            lua,
        ).map_err(LuaError::external)?;

        self.value.replace(Some(lua_value));

        Ok(())
    }

    pub fn apply(
        &self,
        lua: &mlua::Lua,
    ) -> mlua::Result<Option<Value>> {
        let app_state = lua.app_data_ref::<AppState>().unwrap();
        let type_registry = app_state.type_registry();

        // TODO: Check if dirty (deep)

        let lua_value = self.value.borrow();
        let lua_value = match lua_value.as_ref() {
            Some(value) => value,
            None => return Ok(None),
        };

        let r#type = type_registry
            .get_by_hash(LookupKey::Qualified(self.guid.type_hash()))
            .ok_or_else(|| LuaError::type_not_found(self.guid.type_hash()))?;

        let value = convert_lua_to_value(
            lua_value,
            &TypeHandle::new(
                type_registry.clone(),
                r#type.index,
            ),
        ).map_err(LuaError::external)?;

        Ok(Some(value))
    }

    fn get_mapped_value(
        &self,
        lua: &mlua::Lua,
    ) -> mlua::Result<Option<&MappedValue>> {
        let app_state = lua.app_data_ref::<AppState>().unwrap();

        self.original_value.get_or_try_init(|| {
            match app_state.reader().read_resource(&self.guid)? {
                Some(value) => {
                    let type_registry = app_state.type_registry();
                    let r#type = type_registry
                        .get_by_hash(LookupKey::Qualified(self.guid.type_hash()))
                        .ok_or_else(|| LuaError::type_not_found(self.guid.type_hash()))?;

                    let value = MappedValue::from_bytes(
                        type_registry,
                        r#type,
                        &Rc::from(value.into_boxed_slice()),
                    ).map_err(LuaError::external)?;

                    Ok(Some(value))
                },
                None => Ok(None),
            }
        }).map(|v| v.as_ref())
    }

}
