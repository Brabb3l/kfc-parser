use std::{borrow::Borrow, ops::Deref};

use crate::reflection::{StructFieldMetadata, TypeIndex, TypeMetadata, TypeRegistry};

/// A utility type which provides a handle to a type in a type registry.
#[derive(Debug, Clone)]
pub struct TypeHandle<T> {
    type_registry: T,
    index: TypeIndex,
}

impl<T> TypeHandle<T>
where
    T: Borrow<TypeRegistry> + Clone,
{
    #[inline]
    pub fn new(type_registry: T, index: TypeIndex) -> Self {
        Self {
            type_registry,
            index,
        }
    }

    #[inline]
    pub fn try_new(type_registry: T, index: TypeIndex) -> Option<Self> {
        if type_registry.borrow().get(index).is_some() {
            Some(Self {
                type_registry,
                index,
            })
        } else {
            None
        }
    }

    #[inline]
    pub fn type_registry(&self) -> &T {
        &self.type_registry
    }

    pub fn get_field_type(
        &self,
        field_name: &str
    ) -> Option<Self> {
        let field = self.get_field_metadata(field_name)?;
        let type_registry = self.type_registry.clone();
        let field_type = Self::try_new(type_registry, field.r#type)
            .expect("field type must exist in the type registry");

        Some(field_type)
    }

    pub fn get_field_metadata(
        &self,
        field_name: &str
    ) -> Option<&StructFieldMetadata> {
        let type_registry = self.type_registry.borrow();
        let mut r#type = self.deref();
        let mut field;

        loop {
            field = r#type.struct_fields.get(field_name);

            if field.is_some() {
                break;
            }

            match type_registry.get_inner_type(r#type) {
                Some(parent_type) => r#type = parent_type,
                None => return None,
            }
        }

        field
    }

    pub fn is_sub_type_of(
        &self,
        parent: &TypeMetadata,
    ) -> bool {
        self.type_registry.borrow().is_sub_type(self, parent)
    }

    #[inline]
    pub fn index(&self) -> TypeIndex {
        self.index
    }

    #[inline]
    pub fn inner_type(&self) -> Option<Self> {
        self.inner_type
            .map(|index| Self::new(self.type_registry.clone(), index))
    }

    #[inline]
    pub fn unwrap_typedef(self) -> Self {
        Self::new(
            self.type_registry.clone(),
            self.type_registry.borrow()
                .unwrap_typedef(&self)
                .index
        )
    }

    pub fn iter_fields(&self) -> impl Iterator<Item = &StructFieldMetadata> {
        let mut depth = 0;
        let mut current_type: &TypeMetadata = self;

        while let Some(parent_type) = self.type_registry.borrow().get_inner_type(current_type) {
            depth += 1;
            current_type = parent_type;
        }

        FieldIter {
            type_registry: &self.type_registry,
            base_type: self,
            current_type,
            depth,
            index: 0,
        }
    }

}

impl<T> Deref for TypeHandle<T>
where
    T: Borrow<TypeRegistry> + Clone,
{
    type Target = TypeMetadata;

    #[inline]
    fn deref(&self) -> &Self::Target {
        // This assumes that the index is always valid which is guaranteed by the TypeRegistry
        self.type_registry.borrow()
            .get(self.index)
            .unwrap()
    }
}

struct FieldIter<'a, T> {
    type_registry: &'a T,
    base_type: &'a TypeMetadata,
    current_type: &'a TypeMetadata,
    depth: usize,
    index: usize,
}

impl<'a, T> Iterator for FieldIter<'a, T>
where
    T: Borrow<TypeRegistry>,
{
    type Item = &'a StructFieldMetadata;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        while self.index >= self.current_type.struct_fields.len() {
            if self.depth > 0 {
                self.depth -= 1;
                self.index = 0;

                let mut current_type = self.base_type;

                for _ in 0..self.depth {
                    // This is checked during construction of the iterator
                    current_type = self.type_registry.borrow()
                        .get_inner_type(current_type)
                        .unwrap();
                }

                self.current_type = current_type;
            } else {
                return None;
            }
        }

        let field = &self.current_type.struct_fields[self.index];

        self.index += 1;

        Some(field)
    }
}
