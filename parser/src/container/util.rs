use shared::hash::fnv_with_seed;

use crate::guid::{BlobGuid, DescriptorGuid};

use super::{BlobLink, DescriptorLink, KFCFile, PreloadLink};

impl KFCFile {
    pub fn get_blob_link(&self, guid: &BlobGuid) -> Option<&BlobLink> {
        let hash = u32::from_le_bytes([
            guid.data[4],
            guid.data[5],
            guid.data[6],
            guid.data[7],
        ]);

        let index = hash % self.blob_buckets.len() as u32;
        let bucket = &self.blob_buckets[index as usize];

        for i in 0..bucket.count {
            let index = bucket.index as usize + i as usize;

            if self.blob_guids[index] == *guid {
                return Some(&self.blob_links[bucket.index as usize + i as usize]);
            }
        }

        None
    }

    pub fn get_descriptor_link(&self, guid: &DescriptorGuid) -> Option<&DescriptorLink> {
        let seed = u32::from_le_bytes(guid.data[0..4].try_into().unwrap());
        let mut rest = [0u8; 8];
        rest[0..4].copy_from_slice(guid.type_hash.to_le_bytes().as_ref());
        rest[4..8].copy_from_slice(guid.part_number.to_le_bytes().as_ref());

        let hash = fnv_with_seed(rest, seed);

        let index = hash as usize % self.descriptor_buckets.len();
        let bucket = &self.descriptor_buckets[index as usize];

        for i in 0..bucket.count {
            let index = bucket.index as usize + i as usize;

            if self.descriptor_guids[index] == *guid {
                return Some(&self.descriptor_links[index]);
            }
        }

        None
    }

    pub fn get_preload_link(&self, hash: u32) -> Option<&PreloadLink> {
        let index = hash % self.preload_buckets.len() as u32;
        let bucket = &self.preload_buckets[index as usize];

        for i in 0..bucket.count {
            let index = bucket.index as usize + i as usize;

            if self.preload_guids[index].hash == hash {
                return Some(&self.preload_links[bucket.index as usize + i as usize]);
            }
        }

        None
    }
}