use std::{collections::{HashMap, HashSet}, fs::File, io::{BufReader, Read, Seek, SeekFrom, Write}, path::Path};

use crate::{guid::{BlobGuid, DescriptorGuid}, io::{ReadExt, WriteExt, WriteSeekExt}, reflection::{LookupKey, TypeRegistry}, Hash32};

use super::{header::*, KFCReadError, KFCWriteError, StaticMap, StaticMapBucket};

#[derive(Debug, Clone)]
pub struct KFCFile {
    version: String,

    dat_infos: Vec<DatInfo>,
    descriptor_locations: Vec<DescriptorLocation>,

    blobs: StaticMap<BlobGuid, BlobLink>,
    descriptors: StaticMap<DescriptorGuid, DescriptorLink>,

    descriptor_indices: Vec<u32>,
    groups: StaticMap<Hash32, GroupInfo>,
}

impl Default for KFCFile {

    fn default() -> Self {
        Self {
            descriptor_locations: vec![DescriptorLocation::default()],

            version: String::default(),
            dat_infos: Vec::default(),

            blobs: StaticMap::default(),
            descriptors: StaticMap::default(),

            descriptor_indices: Vec::default(),
            groups: StaticMap::default(),
        }
    }

}

impl KFCFile {

    pub fn from_path<P: AsRef<Path>>(
        path: P,
        skip_entries: bool,
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

    pub fn get_version_tag<P: AsRef<Path>>(path: P) -> Result<String, KFCReadError> {
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);
        let header = KFCHeader::read(&mut reader)?;
        reader.seek(SeekFrom::Start(header.version.offset))?;
        let version = reader.read_string(header.version.count)?;
        Ok(version)
    }

    pub fn get_descriptor_guids(&self) -> &[DescriptorGuid] {
        self.descriptors.keys()
    }

    pub fn get_descriptor_link(&self, guid: &DescriptorGuid) -> Option<&DescriptorLink> {
        self.descriptors.get(guid)
    }

    pub fn get_descriptor_iter(&self) -> impl Iterator<Item = (&DescriptorGuid, &DescriptorLink)> {
        self.descriptors.iter()
    }

    pub fn get_descriptor_map(&self) -> &StaticMap<DescriptorGuid, DescriptorLink> {
        &self.descriptors
    }

    pub fn contains_descriptor(&self, guid: &DescriptorGuid) -> bool {
        self.descriptors.contains_key(guid)
    }

    pub fn get_descriptor_types(&self) -> &[Hash32] {
        self.groups.keys()
    }

    pub fn get_descriptor_guids_by_type_hash(&self, type_hash: Hash32) -> impl Iterator<Item = &DescriptorGuid> {
        self.groups.get(&type_hash)
            .map(|info| {
                let start = info.index;
                let end = start + info.count;
                &self.descriptor_indices[start..end]
            })
            .unwrap_or_default()
            .iter()
            .map(|&index| &self.descriptors.keys()[index as usize])
    }

    pub fn get_blob_guids(&self) -> &[BlobGuid] {
        self.blobs.keys()
    }

    pub fn get_blob_link(&self, guid: &BlobGuid) -> Option<&BlobLink> {
        self.blobs.get(guid)
    }

    pub fn get_blob_iter(&self) -> impl Iterator<Item = (&BlobGuid, &BlobLink)> {
        self.blobs.iter()
    }

    pub fn get_blob_map(&self) -> &StaticMap<BlobGuid, BlobLink> {
        &self.blobs
    }

    pub fn contains_blob(&self, guid: &BlobGuid) -> bool {
        self.blobs.contains_key(guid)
    }

    pub fn game_version(&self) -> &str {
        &self.version
    }

    pub fn data_offset(&self) -> u64 {
        self.descriptor_locations[0].offset
    }

    pub fn data_size(&self) -> u64 {
        self.descriptor_locations[0].size
    }

    pub fn get_dat_infos(&self) -> &[DatInfo] {
        &self.dat_infos
    }

    // mutators

    pub fn set_descriptors(
        &mut self,
        descriptors: StaticMap<DescriptorGuid, DescriptorLink>,
        type_registry: &TypeRegistry,
    ) {
        self.descriptors = descriptors;
        self.rebuild_groups(type_registry);
    }

    pub fn set_blobs(&mut self, blobs: StaticMap<BlobGuid, BlobLink>) {
        self.blobs = blobs;
    }

    pub fn set_dat_infos(&mut self, dat_infos: Vec<DatInfo>) {
        self.dat_infos = dat_infos;
    }

    pub fn set_game_version(&mut self, version: String) {
        self.version = version;
    }

    pub fn set_data_location(&mut self, offset: u64, size: u64) {
        self.descriptor_locations[0].offset = offset;
        self.descriptor_locations[0].size = size;
        self.descriptor_locations[0].count = self.descriptors.len();
    }

    fn rebuild_groups(&mut self, type_registry: &TypeRegistry) {
        let mut type_hashes = self.descriptors.keys()
            .iter()
            .map(|guid| guid.type_hash)
            .collect::<HashSet<_>>()
            .into_iter()
            .map(|hash| (hash, GroupInfo {
                // TODO: Remove unwrap
                internal_hash: type_registry.get_by_hash(LookupKey::Qualified(hash)).unwrap().internal_hash,
                ..Default::default()
            }))
            .collect::<Vec<_>>();

        let mut indices = Vec::with_capacity(self.descriptors.len());

        for (hash, info) in type_hashes.iter_mut() {
            info.index = indices.len();

            let hash = *hash;
            let mut count = 0;

            for (i, (guid, _)) in self.descriptors.iter().enumerate() {
                if guid.type_hash == hash {
                    indices.push(i as u32);
                    count += 1;
                }
            }

            info.count = count;
        }

        self.descriptor_indices = indices;
        self.groups = type_hashes.into_iter().collect::<HashMap<_, _>>().into();
    }

}

impl KFCFile {

    fn read<R: Read + Seek>(reader: &mut R, skip_entries: bool) -> Result<Self, KFCReadError> {
        let header = KFCHeader::read(reader)?;

        // version
        reader.seek(SeekFrom::Start(header.version.offset))?;
        let version = reader.read_string(header.version.count)?;

        // dat infos
        reader.seek(SeekFrom::Start(header.dat_infos.offset))?;
        let dat_infos = (0..header.dat_infos.count)
            .map(|_| DatInfo::read(reader))
            .collect::<Result<Vec<_>, _>>()?;

        // descriptor locations
        reader.seek(SeekFrom::Start(header.descriptor_locations.offset))?;
        let descriptor_locations = (0..header.descriptor_locations.count)
            .map(|_| DescriptorLocation::read(reader))
            .collect::<Result<Vec<_>, _>>()?;

        if !skip_entries {
            // group indices
            reader.seek(SeekFrom::Start(header.descriptor_indices.offset))?;
            let descriptor_indices = (0..header.descriptor_indices.count)
                .map(|_| reader.read_u32())
                .collect::<Result<Vec<_>, _>>()?;

            // blob static map

            reader.seek(SeekFrom::Start(header.blob_buckets.offset))?;
            let blob_buckets = (0..header.blob_buckets.count)
                .map(|_| StaticMapBucket::read(reader))
                .collect::<Result<Vec<_>, _>>()?;

            reader.seek(SeekFrom::Start(header.blob_guids.offset))?;
            let blob_guids = (0..header.blob_guids.count)
                .map(|_| BlobGuid::read(reader))
                .collect::<Result<Vec<_>, _>>()?;

            reader.seek(SeekFrom::Start(header.blob_links.offset))?;
            let blob_links = (0..header.blob_links.count)
                .map(|_| BlobLink::read(reader))
                .collect::<Result<Vec<_>, _>>()?;

            // descriptor static map

            reader.seek(SeekFrom::Start(header.descriptor_buckets.offset))?;
            let descriptor_buckets = (0..header.descriptor_buckets.count)
                .map(|_| StaticMapBucket::read(reader))
                .collect::<Result<Vec<_>, _>>()?;

            reader.seek(SeekFrom::Start(header.descriptor_guids.offset))?;
            let descriptor_guids = (0..header.descriptor_guids.count)
                .map(|_| DescriptorGuid::read(reader))
                .collect::<Result<Vec<_>, _>>()?;

            reader.seek(SeekFrom::Start(header.descriptor_links.offset))?;
            let descriptor_links = (0..header.descriptor_links.count)
                .map(|_| DescriptorLink::read(reader))
                .collect::<Result<Vec<_>, _>>()?;

            // group static map

            reader.seek(SeekFrom::Start(header.group_buckets.offset))?;
            let group_buckets = (0..header.group_buckets.count)
                .map(|_| StaticMapBucket::read(reader))
                .collect::<Result<Vec<_>, _>>()?;

            reader.seek(SeekFrom::Start(header.group_hashes.offset))?;
            let group_guids = (0..header.group_hashes.count)
                .map(|_| reader.read_u32())
                .collect::<Result<Vec<_>, _>>()?;

            reader.seek(SeekFrom::Start(header.group_infos.offset))?;
            let group_links = (0..header.group_infos.count)
                .map(|_| GroupInfo::read(reader))
                .collect::<Result<Vec<_>, _>>()?;

            Ok(Self {
                version,
                dat_infos,

                descriptor_locations,
                descriptor_indices,

                blobs: StaticMap::from_parts(
                    blob_guids,
                    blob_links,
                    blob_buckets,
                )?,
                descriptors: StaticMap::from_parts(
                    descriptor_guids,
                    descriptor_links,
                    descriptor_buckets,
                )?,
                groups: StaticMap::from_parts(
                    group_guids,
                    group_links,
                    group_buckets,
                )?,
            })
        } else {
            Ok(Self {
                version,
                dat_infos: Vec::new(),
                descriptor_locations,
                descriptor_indices: Vec::new(),
                blobs: StaticMap::default(),
                descriptors: StaticMap::default(),
                groups: StaticMap::default(),
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

        // dat infos
        let dat_infos_offset = writer.stream_position()?;
        for dat_info in &self.dat_infos {
            dat_info.write(writer)?;
        }
        writer.align(8)?;

        // descriptor locations
        let descriptor_locations_offset = writer.stream_position()?;
        DescriptorLocation::default().write(writer)?;

        // group indices
        let descriptor_indices_offset = writer.stream_position()?;
        for descriptor_index in &self.descriptor_indices {
            writer.write_u32(*descriptor_index)?;
        }

        // blob static map

        let blob_buckets_offset = writer.stream_position()?;
        for blob_bucket in self.blobs.buckets() {
            blob_bucket.write(writer)?;
        }

        let blob_guids_offset = writer.stream_position()?;
        for blob_guid in self.blobs.keys() {
            blob_guid.write(writer)?;
        }
        writer.align(8)?;

        let blob_links_offset = writer.stream_position()?;
        for blob_link in self.blobs.values() {
            blob_link.write(writer)?;
        }

        // descriptor static map

        let descriptor_buckets_offset = writer.stream_position()?;
        for descriptor_bucket in self.descriptors.buckets() {
            descriptor_bucket.write(writer)?;
        }

        let descriptor_guids_offset = writer.stream_position()?;
        for descriptor_guid in self.descriptors.keys() {
            descriptor_guid.write(writer)?;
        }
        writer.align(8)?;

        let descriptor_links_offset = writer.stream_position()?;
        for descriptor_link in self.descriptors.values() {
            descriptor_link.write(writer)?;
        }

        // group static map

        let group_buckets_offset = writer.stream_position()?;
        for group_bucket in self.groups.buckets() {
            group_bucket.write(writer)?;
        }

        let group_hashes_offset = writer.stream_position()?;
        for group_hash in self.groups.keys() {
            writer.write_u32(*group_hash)?;
        }

        let group_infos_offset = writer.stream_position()?;
        for group_info in self.groups.values() {
            group_info.write(writer)?;
        }

        let size = writer.stream_position()?;

        // DescriptorLocation
        writer.seek(SeekFrom::Start(descriptor_locations_offset))?;
        for descriptor_location in &self.descriptor_locations {
            descriptor_location.write(writer)?;
        }

        // KFCHeader
        let header = KFCHeader {
            size,

            version: KFCLocation::new(version_offset, self.version.len()),
            dat_infos: KFCLocation::new(dat_infos_offset, self.dat_infos.len()),

            descriptor_locations: KFCLocation::new(descriptor_locations_offset, self.descriptor_locations.len()),
            descriptor_indices: KFCLocation::new(descriptor_indices_offset, self.descriptor_indices.len()),

            blob_buckets: KFCLocation::new(blob_buckets_offset, self.blobs.buckets().len()),
            blob_guids: KFCLocation::new(blob_guids_offset, self.blobs.len()),
            blob_links: KFCLocation::new(blob_links_offset, self.blobs.len()),

            descriptor_buckets: KFCLocation::new(descriptor_buckets_offset, self.descriptors.buckets().len()),
            descriptor_guids: KFCLocation::new(descriptor_guids_offset, self.descriptors.len()),
            descriptor_links: KFCLocation::new(descriptor_links_offset, self.descriptors.len()),

            group_buckets: KFCLocation::new(group_buckets_offset, self.groups.buckets().len()),
            group_hashes: KFCLocation::new(group_hashes_offset, self.groups.len()),
            group_infos: KFCLocation::new(group_infos_offset, self.groups.len()),

            ..Default::default()
        };

        writer.seek(SeekFrom::Start(0))?;
        header.write(writer)?;

        writer.seek(SeekFrom::Start(size))?;

        Ok(())
    }

}
