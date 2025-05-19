use std::collections::HashMap;

use shared::hash::fnv_const;

use crate::{container::KFCReader, data::{localization::{LocaTagCollectionResourceData, LocaTagResource}, ContentHash}, guid::DescriptorGuid};

use super::{PrimitiveType, ReadError, TypeCollection};

const LOCA_TAG_COLLECTION_RESOURCE: u32 = fnv_const("keen::LocaTagCollectionResource");
const LOCA_TAG_COLLECTION_RESOURCE_DATA: u32 = fnv_const("keen::LocaTagCollectionResourceData");

const NAME_LOCA_TAG: u32 = fnv_const("keen::NameLocaTag"); // typedef<LocaTagReference>
const LOCA_TAG_REFERENCE: u32 = fnv_const("keen::LocaTagReference");
const LOCA_TAG_ID: u32 = fnv_const("keen::LocaTagId");

pub struct DescriptorNameMapper<'a> {
    type_collection: &'a TypeCollection,
    loca_tags: HashMap<u32, LocaTagResource>,
    guid_only: bool,
}

impl<'a> DescriptorNameMapper<'a> {

    pub fn new(type_collection: &'a TypeCollection) -> Self {
        Self {
            type_collection,
            loca_tags: HashMap::new(),
            guid_only: false,
        }
    }

    pub fn set_guid_only(&mut self, guid_only: bool) {
        self.guid_only = guid_only;
    }

    pub fn read_data(&mut self, reader: &mut KFCReader) -> Result<(), ReadError> {
        let root_type_info = reader.file.get_descriptor_guids_by_type_hash(LOCA_TAG_COLLECTION_RESOURCE);
        let guid = root_type_info.first().unwrap();
        let descriptor = reader.read_descriptor(guid)?.unwrap();
        let keenglish_data_hash = descriptor.as_object().unwrap()["keenglishDataHash"].clone();
        let keenglish_data_hash = serde_json::from_value::<ContentHash>(keenglish_data_hash).unwrap();
        let keenglish_data = reader.read_blob(&keenglish_data_hash.as_blob_guid())?.unwrap();

        let loca_type_info = self.type_collection.get_type_by_qualified_hash(LOCA_TAG_COLLECTION_RESOURCE_DATA).unwrap();
        let loca_data_json = self.type_collection.deserialize(loca_type_info, &keenglish_data)?;
        let loca_data = serde_json::from_value::<LocaTagCollectionResourceData>(loca_data_json).unwrap();

        self.loca_tags = loca_data.tags.into_iter().map(|tag| (tag.id.value, tag)).collect();

        Ok(())
    }

    pub fn get_name(
        &self,
        guid: &DescriptorGuid,
        value: &serde_json::Value
    ) -> String {
        if self.guid_only {
            return guid.to_qualified_string();
        }

        // TODO: Give singletons a proper name

        let type_info = self.type_collection.get_type_by_qualified_hash(guid.type_hash);

        let loca_field = type_info
            .and_then(|type_info| {
                type_info.struct_fields.iter()
                    .filter(|field| {
                        let type_info = self.type_collection.get_type(field.r#type).unwrap();
                        
                        type_info.qualified_hash == LOCA_TAG_REFERENCE ||
                        type_info.qualified_hash == NAME_LOCA_TAG
                    })
                    .find(|field| {
                        field.name == "name" || field.name == "debugName" || field.name == "dbgName"
                    })
                    .and_then(|field| value.get(&field.name))
                    .and_then(|loca_tag| loca_tag.as_str())
                    .and_then(|loca_tag| DescriptorGuid::from_str(loca_tag, 0, 0))
                    .and_then(|loca_tag| self.loca_tags.get(&loca_tag.hash32()))
            });

        if let Some(loca_field) = loca_field {
            return loca_field.text.to_ascii_lowercase()
                .replace(" ", "_")
                .replace(|c: char| !c.is_ascii_alphanumeric(), "");
        }

        let loca_field = type_info
            .and_then(|type_info| {
                type_info.struct_fields.iter()
                    .filter(|field| {
                        let type_info = self.type_collection.get_type(field.r#type).unwrap();

                        type_info.qualified_hash == LOCA_TAG_ID
                    })
                    .find(|field| {
                        field.name == "name" || field.name == "debugName" || field.name == "dbgName"
                    })
                    .and_then(|field| value.get(&field.name))
                    .and_then(|loca_tag| loca_tag.as_u64())
                    .and_then(|loca_tag| self.loca_tags.get(&(loca_tag as u32)))
            });

        if let Some(loca_field) = loca_field {
            return loca_field.text.to_ascii_lowercase()
                .replace(" ", "_")
                .replace(|c: char| !c.is_ascii_alphanumeric(), "");
        }

        let debug_name = type_info
            .and_then(|type_info| {
                type_info.struct_fields.iter()
                    .filter(|field| 
                        self.type_collection.resolve_typedef(
                            self.type_collection.get_type(field.r#type).unwrap()
                        ).primitive_type == PrimitiveType::BlobString
                    )
                    .find(|field| {
                        field.name == "name" || field.name == "debugName" || field.name == "dbgName"
                    })
                    .and_then(|field| value.get(&field.name))
                    .and_then(|name| name.as_str())
            });

        if let Some(name) = debug_name {
            name.to_string()
        } else {
            guid.to_qualified_string()
        }
    }

}