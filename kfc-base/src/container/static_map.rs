// TODO: Implement better Debug trait representation for StaticMap and StaticMapBuilder

use std::{cmp::{Eq, Ord}, collections::HashMap, hash::Hash, io::{Read, Write}};

use crate::io::{ReadExt, WriteExt};

use super::StaticMapError;

pub trait StaticHash {
    fn static_hash(&self) -> u32;
}

#[derive(Debug, Clone, Default)]
pub struct StaticMap<K, V> {
    keys: Vec<K>,
    values: Vec<V>,
    buckets: Vec<StaticMapBucket>,
}

impl<K: std::cmp::PartialEq + StaticHash, V> StaticMap<K, V> {

    pub fn from_parts(
        keys: Vec<K>,
        values: Vec<V>,
        buckets: Vec<StaticMapBucket>,
    ) -> Result<Self, StaticMapError> {
        let bucket_ref_count = buckets.iter().map(|b| b.count).sum();

        if keys.len() != values.len() {
            return Err(StaticMapError::LengthMismatch(keys.len(), values.len()));
        }

        if keys.len() != bucket_ref_count {
            return Err(StaticMapError::BucketCountMismatch(keys.len(), bucket_ref_count));
        }

        if !buckets.is_empty() && buckets.len().count_ones() != 1 {
            return Err(StaticMapError::InvalidBucketSize(buckets.len()));
        }

        Ok(Self {
            keys,
            values,
            buckets,
        })
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        let hash = key.static_hash();
        let bucket_index = hash as usize % self.buckets.len();
        let bucket = &self.buckets[bucket_index];

        for i in bucket.index..bucket.index + bucket.count {
            if self.keys[i] == *key {
                return Some(&self.values[i]);
            }
        }

        None
    }

    pub fn contains_key(&self, key: &K) -> bool {
        self.get(key).is_some()
    }

    pub fn len(&self) -> usize {
        self.keys.len()
    }

    pub fn is_empty(&self) -> bool {
        self.keys.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&K, &V)> {
        self.keys.iter().zip(self.values.iter())
    }

    pub fn keys(&self) -> &[K] {
        &self.keys
    }

    pub fn values(&self) -> &[V] {
        &self.values
    }

    pub fn buckets(&self) -> &[StaticMapBucket] {
        &self.buckets
    }

}

impl<K: Eq + Hash + Ord + StaticHash, V> StaticMap<K, V> {

    pub fn into_builder(self) -> StaticMapBuilder<K, V> {
        StaticMapBuilder {
            entries: self.keys.into_iter().zip(self.values).collect()
        }
    }

}

impl<K: Eq + Hash + Ord + StaticHash + Clone, V: Clone> StaticMap<K, V> {

    pub fn as_builder(&self) -> StaticMapBuilder<K, V> {
        StaticMapBuilder {
            entries: self.keys.iter().cloned().zip(self.values.iter().cloned()).collect(),
        }
    }

}

#[derive(Debug, Clone, Default)]
pub struct StaticMapBucket {
    index: usize,
    count: usize,
}

impl StaticMapBucket {

    pub fn read<R: Read>(reader: &mut R) -> std::io::Result<Self> {
        let index = reader.read_u32()? as usize;
        let count = reader.read_u32()? as usize;

        Ok(Self { index, count })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        writer.write_u32(self.index as u32)?;
        writer.write_u32(self.count as u32)?;

        Ok(())
    }

}

#[derive(Debug, Clone, Default)]
pub struct StaticMapBuilder<K, V> {
    entries: HashMap<K, V>,
}

impl<K: Eq + Hash + Ord + StaticHash, V> StaticMapBuilder<K, V> {

    pub fn insert(&mut self, key: K, value: V) {
        self.entries.insert(key, value);
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        self.entries.get(key)
    }

    pub fn contains_key(&self, key: &K) -> bool {
        self.entries.contains_key(key)
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&K, &V)> {
        self.entries.iter()
    }

    pub fn build(self) -> StaticMap<K, V> {
        let bucket_size = self.entries.len().next_power_of_two();
        let mut buckets = vec![StaticMapBucket::default(); bucket_size];

        let mut entries = self.entries.into_iter().collect::<Vec<_>>();
        entries.sort_by_key(|(k, _)| k.static_hash() % bucket_size as u32);

        let mut bucket_index = 0;
        let mut entry_index = 0;

        while bucket_index < bucket_size {
            let bucket = &mut buckets[bucket_index];
            bucket.index = entry_index;

            let mut count = 0;

            while entry_index < entries.len() && entries[entry_index].0.static_hash() as usize % bucket_size == bucket_index {
                entry_index += 1;
                count += 1;
            }

            bucket.count = count;
            bucket_index += 1;
        }

        let (keys, values) = entries.into_iter().unzip();

        StaticMap::from_parts(keys, values, buckets)
            .expect("Failed to build StaticMap") // should never fail
    }

}

impl<K: Eq + Hash + Ord + StaticHash, V> From<HashMap<K, V>> for StaticMap<K, V> {

    fn from(map: HashMap<K, V>) -> Self {
        StaticMapBuilder {
            entries: map,
        }.build()
    }

}
