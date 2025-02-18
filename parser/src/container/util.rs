use std::hash::{DefaultHasher, Hasher};
use shared::hash::{crc64, fnv};
use crate::container::{BlobLink, DescriptorLink, KFCFile, PreloadLink};
use crate::guid::{BlobGuid, DescriptorGuid};

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
        todo!("Implement hash function for DescriptorGuid");
        let hash = 0;

        let index = hash % self.descriptor_buckets.len() as u64;
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