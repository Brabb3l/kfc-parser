//! Name-based boundary around identifiers required by the physical KFC format.
use crate::{
    container::KFCFile,
    guid::{Guid, ResourceId},
    reflection::{LookupKey, TypeIndex, TypeMetadata, TypeRegistry},
};

pub fn resource_id(
    registry: &TypeRegistry,
    qualified_name: &str,
    guid: Guid,
    part: u32,
) -> Option<ResourceId> {
    registry
        .get_by_name(LookupKey::Qualified(qualified_name))
        .map(|metadata| ResourceId::new(guid, metadata.qualified_hash, part))
}

pub fn resource_id_for_type(
    registry: &TypeRegistry,
    type_index: TypeIndex,
    guid: Guid,
    part: u32,
) -> Option<ResourceId> {
    registry
        .get(type_index)
        .and_then(|metadata| resource_id(registry, &metadata.qualified_name, guid, part))
}

pub fn type_for_resource<'a>(
    registry: &'a TypeRegistry,
    resource: &ResourceId,
) -> Option<&'a TypeMetadata> {
    registry.get_by_hash(LookupKey::Qualified(resource.type_hash()))
}

pub fn same_type(left: &ResourceId, right: &ResourceId) -> bool {
    left.type_hash() == right.type_hash()
}

pub fn resources_by_type(
    file: &KFCFile,
    registry: &TypeRegistry,
    qualified_name: &str,
) -> Vec<ResourceId> {
    let Some(metadata) = registry.get_by_name(LookupKey::Qualified(qualified_name)) else {
        return Vec::new();
    };
    file.resources_by_type(metadata.qualified_hash)
        .copied()
        .collect()
}

pub fn resource_type_indices(file: &KFCFile, registry: &TypeRegistry) -> Vec<TypeIndex> {
    file.resource_types()
        .filter_map(|identifier| registry.get_by_hash(LookupKey::Qualified(identifier)))
        .map(|metadata| metadata.index)
        .collect()
}
