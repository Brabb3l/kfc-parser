use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{Debug, Display};
use std::path::Path;

use crate::hash::fnv;
use crate::reflection::{PrimitiveType, ReflectionParseError, TypeFlags, TypeMetadata};

#[derive(Debug, Default)]
pub struct TypeRegistry {
    pub version: String,
    pub(super) types: Vec<TypeMetadata>,
    pub(super) types_by_qualified_hash: HashMap<u32, TypeIndex>,
    pub(super) types_by_impact_hash: HashMap<u32, TypeIndex>,
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TypeIndex(pub(super) usize);

impl TypeIndex {

    /// Creates a new `TypeIndex` from the given index.
    ///
    /// **Note:** This does not validate the index against any type registry. Use with caution.
    #[inline]
    pub fn new(index: usize) -> Self {
        Self(index)
    }

    #[inline]
    pub fn as_usize(&self) -> usize {
        self.0
    }

}

impl Display for TypeIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LookupKey<T> {
    Qualified(T),
    Impact(T),
}

impl TypeRegistry {

    #[inline]
    pub fn load_from_executable(
        path: impl AsRef<Path>,
    ) -> Result<Self, ReflectionParseError> {
        let mut registry = Self::default();
        let types = super::extract_reflection_data(path)?;

        registry.extend(types);

        Ok(registry)
    }

    #[inline]
    pub fn get(
        &self,
        index: TypeIndex,
    ) -> Option<&TypeMetadata> {
        self.types.get(index.0)
    }

    #[inline]
    pub fn get_by_hash(
        &self,
        key: LookupKey<u32>
    ) -> Option<&TypeMetadata> {
        match key {
            LookupKey::Qualified(hash) => self.types_by_qualified_hash.get(&hash)
                .and_then(|&index| self.get(index)),
            LookupKey::Impact(hash) => self.types_by_impact_hash.get(&hash)
                .and_then(|&index| self.get(index)),
        }
    }

    #[inline]
    pub fn get_by_name<T: AsRef<str>>(
        &self,
        key: LookupKey<T>
    ) -> Option<&TypeMetadata> {
        let key = match key {
            LookupKey::Qualified(name) => LookupKey::Qualified(fnv(name.as_ref())),
            LookupKey::Impact(name) => LookupKey::Impact(fnv(name.as_ref())),
        };

        self.get_by_hash(key)
    }

    pub fn is_sub_type(
        &self,
        parent: &TypeMetadata,
        child: &TypeMetadata,
    ) -> bool {
        if child.index == parent.index {
            return true;
        }

        let mut current = child;

        while let Some(inner_type) = current.inner_type {
            if let Some(parent_type) = self.get(inner_type) {
                if parent_type.index == parent.index {
                    return true;
                }

                current = parent_type;
            } else {
                break;
            }
        }

        false
    }

    pub fn get_inheritance_chain<'a>(
        &'a self,
        node: &'a TypeMetadata
    ) -> Vec<&'a TypeMetadata> {
        let mut chain = Vec::new();
        let mut current = node;

        loop {
            chain.push(current);

            if let Some(parent) = &current.inner_type {
                if let Some(parent) = self.get(*parent) {
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

    #[inline]
    pub fn len(&self) -> usize {
        self.types.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.types.is_empty()
    }

    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &TypeMetadata> {
        self.types.iter()
    }

    #[inline]
    pub fn into_inner(self) -> Vec<TypeMetadata> {
        self.types
    }

    #[inline]
    pub fn get_inner_type<'a>(
        &'a self,
        r#type: &'a TypeMetadata
    ) -> Option<&'a TypeMetadata> {
        r#type.inner_type
            .and_then(|t| self.get(t))
            .map(|t| self.unwrap_typedef(t))
    }

    pub fn clear(&mut self) {
        self.types.clear();
        self.types_by_qualified_hash.clear();
        self.types_by_impact_hash.clear();
    }

    #[inline]
    pub fn unwrap_typedef<'a>(
        &'a self,
        r#type: &'a TypeMetadata
    ) -> &'a TypeMetadata {
        match &r#type.primitive_type {
            PrimitiveType::Typedef => {
                // TODO: ensure safe unwrap by validating it in [extend]
                self.get_inner_type(r#type)
                    .expect("missing inner type for typedef")
            },
            _ => r#type
        }
    }

    pub(super) fn extend(
        &mut self,
        types: Vec<TypeMetadata>
    ) -> usize {
        if self.types.is_empty() {
            self.types.reserve_exact(types.len());
            self.types_by_qualified_hash.reserve(types.len());
        }

        let len = types.len();

        // TODO: handle duplicates properly
        // TODO: maybe add validation at some point

        for value in types {
            if !value.flags.contains(TypeFlags::HAS_DS) {
                if let Some(previous) = self.types_by_impact_hash.get(&value.impact_hash) {
                    panic!(
                        "Duplicate impact hash: {:#010X}, previous: {}, current: {}",
                        value.impact_hash,
                        self.get(*previous).unwrap().qualified_name,
                        value.qualified_name
                    );
                }

                self.types_by_impact_hash.insert(value.impact_hash, value.index);
            }

            if let Some(previous) = self.types_by_qualified_hash.get(&value.qualified_hash) {
                panic!(
                    "Duplicate qualified hash: {:#010X}, previous: {}, current: {}",
                    value.qualified_hash,
                    self.get(*previous).unwrap().qualified_name,
                    value.qualified_name
                );
            }

            self.types_by_qualified_hash.insert(value.qualified_hash, value.index);
            self.types.push(value);
        }

        len
    }

}
