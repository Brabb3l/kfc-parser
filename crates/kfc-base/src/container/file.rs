use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io::{BufReader, Read, Seek, SeekFrom, Write},
    path::Path,
};

use crate::{
    Hash32,
    guid::{ContentHash, ResourceId},
    io::{ReadExt, WriteExt, WriteSeekExt},
    reflection::{LookupKey, TypeRegistry},
};

use super::{KFCReadError, KFCWriteError, StaticMap, StaticMapBucket, header::*};

#[derive(Debug, Clone)]
pub struct KFCFile {
    version: String,

    containers: Vec<ContainerInfo>,
    resource_locations: Vec<ResourceLocation>,

    contents: StaticMap<ContentHash, ContentEntry>,
    resources: StaticMap<ResourceId, ResourceEntry>,

    resource_indices: Vec<u32>,
    resource_bundles: StaticMap<Hash32, ResourceBundleEntry>,
}

impl Default for KFCFile {
    fn default() -> Self {
        Self {
            resource_locations: vec![ResourceLocation::default()],

            version: String::default(),
            containers: Vec::default(),

            contents: StaticMap::default(),
            resources: StaticMap::default(),

            resource_indices: Vec::default(),
            resource_bundles: StaticMap::default(),
        }
    }
}

impl KFCFile {
    pub fn from_path<P: AsRef<Path>>(
        path: P,
        skip_entries: bool
    ) -> Result<Self, KFCReadError> {
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);
        Self::read(&mut reader, skip_entries)
    }

    pub fn from_reader<R: Read + Seek>(
        reader: &mut R,
        skip_entries: bool,
    ) -> Result<Self, KFCReadError> {
        Self::read(reader, skip_entries)
    }

    pub fn get_version_tag<P: AsRef<Path>>(
        path: P
    ) -> Result<String, KFCReadError> {
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);
        let header = KFCHeader::read(&mut reader)?;
        reader.seek(SeekFrom::Start(header.version.offset))?;
        let version = reader.read_string(header.version.count)?;
        Ok(version)
    }

    #[inline]
    pub fn resources(&self) -> &StaticMap<ResourceId, ResourceEntry> {
        &self.resources
    }

    #[inline]
    pub fn resource_bundles(&self) -> &StaticMap<Hash32, ResourceBundleEntry> {
        &self.resource_bundles
    }

    #[inline]
    pub fn contents(&self) -> &StaticMap<ContentHash, ContentEntry> {
        &self.contents
    }

    #[inline]
    pub fn game_version(&self) -> &str {
        &self.version
    }

    #[inline]
    pub fn data_offset(&self) -> u64 {
        self.resource_locations[0].offset
    }

    #[inline]
    pub fn data_size(&self) -> u64 {
        self.resource_locations[0].size
    }

    #[inline]
    pub fn containers(&self) -> &[ContainerInfo] {
        &self.containers
    }

    pub fn resource_types(&self) -> impl Iterator<Item = Hash32> {
        self.resource_bundles.values().iter().filter_map(|info| {
            let start = info.index;
            let end = start + info.count;
            let resource_indices = &self.resource_indices[start..end];

            if resource_indices.is_empty() {
                return None;
            }

            Some(self.resources.keys()[resource_indices[0] as usize].type_hash())
        })
    }

    pub fn resources_by_type(&self, type_hash: Hash32) -> impl Iterator<Item = &ResourceId> {
        self.resource_bundles
            .get(&type_hash)
            .map(|info| {
                let start = info.index;
                let end = start + info.count;
                &self.resource_indices[start..end]
            })
            .unwrap_or_default()
            .iter()
            .map(|&index| &self.resources.keys()[index as usize])
    }

    // mutators

    pub fn set_resources(
        &mut self,
        resources: StaticMap<ResourceId, ResourceEntry>,
        type_registry: &TypeRegistry,
    ) {
        self.resources = resources;
        self.rebuild_resource_bundles(type_registry);
    }

    pub fn set_contents(
        &mut self,
        contents: StaticMap<ContentHash, ContentEntry>
    ) {
        self.contents = contents;
    }

    pub fn set_containers(
        &mut self,
        containers: Vec<ContainerInfo>
    ) {
        self.containers = containers;
    }

    pub fn set_game_version(&mut self, version: String) {
        self.version = version;
    }

    pub fn set_data_location(&mut self, offset: u64, size: u64) {
        self.resource_locations[0].offset = offset;
        self.resource_locations[0].size = size;
        self.resource_locations[0].count = self.resources.len();
    }

    fn rebuild_resource_bundles(&mut self, type_registry: &TypeRegistry) {
        let mut type_hashes = self
            .resources
            .keys()
            .iter()
            .map(|guid| guid.type_hash())
            .collect::<HashSet<_>>()
            .into_iter()
            .map(|hash| {
                (
                    hash,
                    ResourceBundleEntry {
                        // TODO: Remove unwrap
                        internal_hash: type_registry
                            .get_by_hash(LookupKey::Qualified(hash))
                            .unwrap()
                            .internal_hash,
                        ..Default::default()
                    },
                )
            })
            .collect::<Vec<_>>();

        let mut indices = Vec::with_capacity(self.resources.len());

        for (hash, info) in type_hashes.iter_mut() {
            info.index = indices.len();

            let hash = *hash;
            let mut count = 0;

            for (i, (guid, _)) in self.resources.iter().enumerate() {
                if guid.type_hash() == hash {
                    indices.push(i as u32);
                    count += 1;
                }
            }

            info.count = count;
        }

        self.resource_indices = indices;
        self.resource_bundles = type_hashes.into_iter().collect::<HashMap<_, _>>().into();
    }
}

impl KFCFile {
    fn read<R: Read + Seek>(reader: &mut R, skip_entries: bool) -> Result<Self, KFCReadError> {
        let header = KFCHeader::read(reader)?;

        // version
        reader.seek(SeekFrom::Start(header.version.offset))?;
        let version = reader.read_string(header.version.count)?;

        // containers
        reader.seek(SeekFrom::Start(header.containers.offset))?;
        let containers = (0..header.containers.count)
            .map(|_| ContainerInfo::read(reader))
            .collect::<Result<Vec<_>, _>>()?;

        // resource locations
        reader.seek(SeekFrom::Start(header.resource_locations.offset))?;
        let resource_locations = (0..header.resource_locations.count)
            .map(|_| ResourceLocation::read(reader))
            .collect::<Result<Vec<_>, _>>()?;

        if !skip_entries {
            // resource indices
            reader.seek(SeekFrom::Start(header.resource_indices.offset))?;
            let resource_indices = (0..header.resource_indices.count)
                .map(|_| reader.read_u32())
                .collect::<Result<Vec<_>, _>>()?;

            // content static map

            reader.seek(SeekFrom::Start(header.content_buckets.offset))?;
            let content_buckets = (0..header.content_buckets.count)
                .map(|_| StaticMapBucket::read(reader))
                .collect::<Result<Vec<_>, _>>()?;

            reader.seek(SeekFrom::Start(header.content_keys.offset))?;
            let content_keys = (0..header.content_keys.count)
                .map(|_| ContentHash::read(reader))
                .collect::<Result<Vec<_>, _>>()?;

            reader.seek(SeekFrom::Start(header.content_values.offset))?;
            let content_values = (0..header.content_values.count)
                .map(|_| ContentEntry::read(reader))
                .collect::<Result<Vec<_>, _>>()?;

            // resource static map

            reader.seek(SeekFrom::Start(header.resource_buckets.offset))?;
            let resource_buckets = (0..header.resource_buckets.count)
                .map(|_| StaticMapBucket::read(reader))
                .collect::<Result<Vec<_>, _>>()?;

            reader.seek(SeekFrom::Start(header.resource_keys.offset))?;
            let resource_keys = (0..header.resource_keys.count)
                .map(|_| ResourceId::read(reader))
                .collect::<Result<Vec<_>, _>>()?;

            reader.seek(SeekFrom::Start(header.resource_values.offset))?;
            let resource_values = (0..header.resource_values.count)
                .map(|_| ResourceEntry::read(reader))
                .collect::<Result<Vec<_>, _>>()?;

            // resource_bundle static map

            reader.seek(SeekFrom::Start(header.resource_bundle_buckets.offset))?;
            let resource_bundle_buckets = (0..header.resource_bundle_buckets.count)
                .map(|_| StaticMapBucket::read(reader))
                .collect::<Result<Vec<_>, _>>()?;

            reader.seek(SeekFrom::Start(header.resource_bundle_keys.offset))?;
            let resource_bundle_keys = (0..header.resource_bundle_keys.count)
                .map(|_| reader.read_u32())
                .collect::<Result<Vec<_>, _>>()?;

            reader.seek(SeekFrom::Start(header.resource_bundle_values.offset))?;
            let resource_bundle_values = (0..header.resource_bundle_values.count)
                .map(|_| ResourceBundleEntry::read(reader))
                .collect::<Result<Vec<_>, _>>()?;

            Ok(Self {
                version,
                containers,

                resource_locations,
                resource_indices,

                contents: StaticMap::from_parts(content_keys, content_values, content_buckets)?,
                resources: StaticMap::from_parts(resource_keys, resource_values, resource_buckets)?,
                resource_bundles: StaticMap::from_parts(resource_bundle_keys, resource_bundle_values, resource_bundle_buckets)?,
            })
        } else {
            Ok(Self {
                version,
                containers: Vec::new(),
                resource_locations,
                resource_indices: Vec::new(),
                contents: StaticMap::default(),
                resources: StaticMap::default(),
                resource_bundles: StaticMap::default(),
            })
        }
    }

    pub(super) fn write_info<W: Write + Seek>(&self, writer: &mut W) -> Result<(), KFCWriteError> {
        // OPTIMIZE: Only write the necessary parts of the file
        self.write(writer)
    }

    pub(super) fn write<W: Write + Seek>(&self, writer: &mut W) -> Result<(), KFCWriteError> {
        KFCHeader::default().write(writer)?;

        // version
        let version_offset = writer.stream_position()?;
        writer.write_string(&self.version, self.version.len())?;
        writer.align(8)?;

        // containers
        let containers_offset = writer.stream_position()?;
        for container in &self.containers {
            container.write(writer)?;
        }
        writer.align(8)?;

        // obejct locations
        let resource_locations_offset = writer.stream_position()?;
        ResourceLocation::default().write(writer)?;

        // resource indices
        let resource_indices_offset = writer.stream_position()?;
        for resource_index in &self.resource_indices {
            writer.write_u32(*resource_index)?;
        }

        // content static map

        let content_buckets_offset = writer.stream_position()?;
        for content_bucket in self.contents.buckets() {
            content_bucket.write(writer)?;
        }

        let content_keys_offset = writer.stream_position()?;
        for content_key in self.contents.keys() {
            content_key.write(writer)?;
        }
        writer.align(8)?;

        let content_values_offset = writer.stream_position()?;
        for content_value in self.contents.values() {
            content_value.write(writer)?;
        }

        // resource static map

        let resource_buckets_offset = writer.stream_position()?;
        for resource_bucket in self.resources.buckets() {
            resource_bucket.write(writer)?;
        }

        let resource_keys_offset = writer.stream_position()?;
        for resource_key in self.resources.keys() {
            resource_key.write(writer)?;
        }
        writer.align(8)?;

        let resource_values_offset = writer.stream_position()?;
        for resource_value in self.resources.values() {
            resource_value.write(writer)?;
        }

        // resource_bundle static map

        let resource_bundle_buckets_offset = writer.stream_position()?;
        for resource_bundle_bucket in self.resource_bundles.buckets() {
            resource_bundle_bucket.write(writer)?;
        }

        let resource_bundle_keys_offset = writer.stream_position()?;
        for resource_bundle_key in self.resource_bundles.keys() {
            writer.write_u32(*resource_bundle_key)?;
        }

        let resource_bundle_values_offset = writer.stream_position()?;
        for resource_bundle_value in self.resource_bundles.values() {
            resource_bundle_value.write(writer)?;
        }

        let size = writer.stream_position()?;

        // resource locations (update)
        writer.seek(SeekFrom::Start(resource_locations_offset))?;
        for resource_location in &self.resource_locations {
            resource_location.write(writer)?;
        }

        // KFCHeader
        let header = KFCHeader {
            size,

            version: KFCLocation::new(version_offset, self.version.len()),
            containers: KFCLocation::new(containers_offset, self.containers.len()),

            resource_locations: KFCLocation::new(
                resource_locations_offset,
                self.resource_locations.len(),
            ),
            resource_indices: KFCLocation::new(resource_indices_offset, self.resource_indices.len()),

            content_buckets: KFCLocation::new(content_buckets_offset, self.contents.buckets().len()),
            content_keys: KFCLocation::new(content_keys_offset, self.contents.len()),
            content_values: KFCLocation::new(content_values_offset, self.contents.len()),

            resource_buckets: KFCLocation::new(resource_buckets_offset, self.resources.buckets().len()),
            resource_keys: KFCLocation::new(resource_keys_offset, self.resources.len()),
            resource_values: KFCLocation::new(resource_values_offset, self.resources.len()),

            resource_bundle_buckets: KFCLocation::new(resource_bundle_buckets_offset, self.resource_bundles.buckets().len()),
            resource_bundle_keys: KFCLocation::new(resource_bundle_keys_offset, self.resource_bundles.len()),
            resource_bundle_values: KFCLocation::new(resource_bundle_values_offset, self.resource_bundles.len()),

            ..Default::default()
        };

        writer.seek(SeekFrom::Start(0))?;
        header.write(writer)?;

        writer.seek(SeekFrom::Start(size))?;

        Ok(())
    }
}
