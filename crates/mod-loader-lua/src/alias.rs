use std::rc::Rc;

use kfc::reflection::TypeRegistry;

pub type Path = camino::Utf8Path;
pub type PathBuf = camino::Utf8PathBuf;

pub type TypeHandle = kfc::reflection::TypeHandle<Rc<TypeRegistry>>;
pub type MappedValue = kfc::resource::mapped::MappedValue<Rc<[u8]>, Rc<TypeRegistry>>;
pub type MappedStruct = kfc::resource::mapped::MappedStruct<Rc<[u8]>, Rc<TypeRegistry>>;
pub type MappedArray = kfc::resource::mapped::MappedArray<Rc<[u8]>, Rc<TypeRegistry>>;
pub type MappedVariant = kfc::resource::mapped::MappedVariant<Rc<[u8]>, Rc<TypeRegistry>>;
pub type MappedBitmask = kfc::resource::mapped::MappedBitmask<Rc<TypeRegistry>>;
