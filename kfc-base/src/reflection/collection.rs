use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Debug;
use std::path::Path;
use std::sync::Arc;

use crate::hash::fnv;

use super::types::*;
use super::{parser, ReflectionParseError};

// TODO: Remove `Arc`
#[derive(Debug, Default)]
pub struct TypeCollection {
    pub version: String,
    types: Vec<Arc<TypeInfo>>,
    types_by_qualified_hash: HashMap<u32, Arc<TypeInfo>>,
    types_by_impact_hash: HashMap<u32, Arc<TypeInfo>>,
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
            .map(|node| node.as_ref())
    }

    pub fn get_type_by_qualified_hash(
        &self,
        hash: u32
    ) -> Option<&TypeInfo> {
        self.types_by_qualified_hash.get(&hash)
            .map(|node| node.as_ref())
    }

    pub fn get_type_by_impact_hash(
        &self,
        hash: u32
    ) -> Option<&TypeInfo> {
        self.types_by_impact_hash.get(&hash)
            .map(|node| node.as_ref())
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

    pub fn get_inheritance_chain<'a>(&'a self, node: &'a TypeInfo) -> Vec<&'a TypeInfo> {
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
        let len = types.len();

        // TODO: Handle duplicates properly

        for entry in types {
            let value = Arc::new(entry);

            if !value.flags.contains(TypeFlags::HAS_DS) {
                if self.types_by_impact_hash.contains_key(&value.impact_hash) {
                    panic!("Duplicate impact hash: {:#010X}", value.impact_hash);
                }

                self.types_by_impact_hash.insert(value.impact_hash, value.clone());
            }

            if self.types_by_qualified_hash.contains_key(&value.qualified_hash) {
                panic!("Duplicate qualified hash: {:#010X}", value.qualified_hash);
            }

            self.types_by_qualified_hash.insert(value.qualified_hash, value.clone());
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
            .map(|node| node.as_ref())
    }

    pub fn iter_arc(&self) -> impl Iterator<Item = &Arc<TypeInfo>> {
        self.types.iter()
    }

    /// Consumes the collection and returns the inner types.
    ///
    /// # Errors
    /// If there are still strong references to the types in the collection,
    /// it will return an error with the unchanged collection.
    ///
    /// # Panics
    /// It may panic if another thread creates a new strong
    /// reference to a type while this method is running.
    #[allow(clippy::result_large_err)]
    pub fn into_inner(self) -> Result<Vec<TypeInfo>, TypeCollection> {
        for node in &self.types {
            if Arc::strong_count(node) > 3 {
                return Err(self);
            }
        }

        drop(self.types_by_impact_hash);
        drop(self.types_by_qualified_hash);

        // panics if another thread creates a new strong reference
        let result = self.types.into_iter()
            .map(|node| Arc::try_unwrap(node).unwrap())
            .collect::<Vec<_>>();

        Ok(result)
    }
}

#[derive(Serialize, Deserialize)]
struct TypeCollectionSerde {
    version: String,
    types: Vec<TypeInfo>,
}

impl<'de> Deserialize<'de> for TypeCollection {
    fn deserialize<D>(deserializer: D) -> Result<TypeCollection, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let data = TypeCollectionSerde::deserialize(deserializer)?;
        let mut collection = TypeCollection {
            version: data.version,
            types: Vec::new(),
            types_by_qualified_hash: HashMap::new(),
            types_by_impact_hash: HashMap::new(),
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
        let data = TypeCollectionSerde {
            version: self.version.clone(),
            types: self.types.iter()
                .map(|node| node.as_ref().clone())
                .collect(),
        };

        data.serialize(serializer)
    }
}
