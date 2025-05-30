use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Debug;
use std::path::Path;

use crate::hash::fnv;

use super::types::*;
use super::{parser, ReflectionParseError};

#[derive(Debug, Default)]
pub struct TypeCollection {
    pub version: String,
    types: Vec<TypeInfo>,
    types_by_qualified_hash: HashMap<u32, usize>,
    types_by_impact_hash: HashMap<u32, usize>,
}

impl TypeCollection {

    pub fn load_from_executable(
        &mut self,
        path: impl AsRef<Path>,
    ) -> Result<usize, ReflectionParseError> {
        parser::extract_reflection_data(path)
            .map(|types| self.extend(types))
    }

    pub fn get_type(
        &self,
        index: usize,
    ) -> Option<&TypeInfo> {
        self.types.get(index)
    }

    pub fn get_type_by_qualified_hash(
        &self,
        hash: u32
    ) -> Option<&TypeInfo> {
        self.types_by_qualified_hash.get(&hash)
            .and_then(|&index| self.get_type(index))
    }

    pub fn get_type_by_impact_hash(
        &self,
        hash: u32
    ) -> Option<&TypeInfo> {
        self.types_by_impact_hash.get(&hash)
            .and_then(|&index| self.get_type(index))
    }

    pub fn get_type_by_qualified_name(
        &self,
        name: &str
    ) -> Option<&TypeInfo> {
        self.get_type_by_qualified_hash(fnv(name.as_bytes()))
    }

    pub fn get_type_by_impact_name(
        &self,
        name: &str
    ) -> Option<&TypeInfo> {
        self.get_type_by_impact_hash(fnv(name.as_bytes()))
    }

    pub fn get_inheritance_chain<'a>(
        &'a self,
        node: &'a TypeInfo
    ) -> Vec<&'a TypeInfo> {
        let mut chain = Vec::new();
        let mut current = node;

        loop {
            chain.push(current);

            if let Some(parent) = &current.inner_type {
                if let Some(parent) = self.get_type(*parent) {
                    current = parent;
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        chain
    }

    pub fn clear(&mut self) {
        self.types.clear();
        self.types_by_qualified_hash.clear();
        self.types_by_impact_hash.clear();
    }

    pub fn extend(&mut self, types: Vec<TypeInfo>) -> usize {
        if self.types.is_empty() {
            self.types.reserve_exact(types.len());
            self.types_by_qualified_hash.reserve(types.len());
        }

        let len = types.len();

        // TODO: Handle duplicates properly

        for value in types {
            let index = self.types.len();

            if !value.flags.contains(TypeFlags::HAS_DS) {
                if self.types_by_impact_hash.contains_key(&value.impact_hash) {
                    panic!("Duplicate impact hash: {:#010X}", value.impact_hash);
                }

                self.types_by_impact_hash.insert(value.impact_hash, index);
            }

            if self.types_by_qualified_hash.contains_key(&value.qualified_hash) {
                panic!("Duplicate qualified hash: {:#010X}", value.qualified_hash);
            }

            self.types_by_qualified_hash.insert(value.qualified_hash, index);
            self.types.push(value);
        }

        len
    }

    pub fn len(&self) -> usize {
        self.types.len()
    }

    pub fn is_empty(&self) -> bool {
        self.types.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &TypeInfo> {
        self.types.iter()
    }

    /// Consumes the collection and returns the inner types.
    pub fn into_inner(self) -> Vec<TypeInfo> {
        self.types
    }
}

#[derive(Deserialize)]
struct TypeCollectionSerdeOwned {
    version: String,
    types: Vec<TypeInfo>,
}

#[derive(Serialize)]
struct TypeCollectionSerdeRef<'a> {
    version: &'a str,
    types: &'a [TypeInfo],
}

impl<'de> Deserialize<'de> for TypeCollection {
    fn deserialize<D>(deserializer: D) -> Result<TypeCollection, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let data = TypeCollectionSerdeOwned::deserialize(deserializer)?;
        let mut collection = TypeCollection {
            version: data.version,
            ..Default::default()
        };

        collection.extend(data.types);

        Ok(collection)
    }
}

impl serde::Serialize for TypeCollection {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let data = TypeCollectionSerdeRef {
            version: &self.version,
            types: &self.types
        };

        data.serialize(serializer)
    }
}
