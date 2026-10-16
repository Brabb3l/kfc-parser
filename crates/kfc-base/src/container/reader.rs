use std::{borrow::Borrow, fs::File, io::{BufReader, Read, Seek, SeekFrom, Take}, path::{Path, PathBuf}, sync::{Mutex, OnceLock}};

use crate::{container::KFCReadError, guid::{ContentHash, ResourceId}};

use super::{header::ResourceEntry, KFCFile};

/// Resource sizes are stored in 24 bits, so sizes of 16 MiB and larger are
/// truncated to their lower 24 bits.
const RESOURCE_SIZE_MODULUS: u64 = 0x1000000;
const RESOURCE_ALIGNMENT: u64 = 16;

#[derive(Default)]
struct ResourceChunkCacheEntry {
    data: OnceLock<Box<[u8]>>,
    loading: Mutex<()>,
}

impl ResourceChunkCacheEntry {
    fn get_or_load(&self, load: impl FnOnce() -> std::io::Result<Vec<u8>>) -> std::io::Result<&[u8]> {
        if let Some(data) = self.data.get() {
            return Ok(data);
        }

        // Serialize only this chunk's first load. Hits never acquire a mutex,
        // and failed loads leave the entry empty so another cursor can retry.
        let _loading = self.loading.lock()
            .map_err(|_| std::io::Error::other("Resource chunk cache lock poisoned"))?;
        if let Some(data) = self.data.get() {
            return Ok(data);
        }

        let data = load()?.into_boxed_slice();
        Ok(self.data.get_or_init(|| data))
    }
}

pub struct KFCReader {
    file: KFCFile,
    /// Sorted, distinct resource offsets followed by the end of the resource data.
    resource_bounds: Vec<u64>,
    resource_chunks_ordered: bool,
    /// Lazy decompressed data shared by every cursor, retained until this reader
    /// is dropped. The maximum payload is the sum of uncompressed chunk sizes,
    /// rather than that sum multiplied by the number of cursors.
    chunk_cache: Vec<ResourceChunkCacheEntry>,

    path: PathBuf,
    file_name: String,
    kfc_path: PathBuf,
    dat_extension: String,
    resource_path: PathBuf,
}

#[derive(Debug)]
pub struct KFCReaderOptions<'a> {
    pub kfc_extension: &'a str,
    pub dat_extension: &'a str,
    pub resource_extension: &'a str,
}

impl Default for KFCReaderOptions<'_> {
    fn default() -> Self {
        Self {
            kfc_extension: "kfc",
            dat_extension: "dat",
            resource_extension: "kfc_resources",
        }
    }
}

impl KFCReader {

    pub fn new(
        path: impl AsRef<Path>,
        file_name: impl AsRef<str>,
    ) -> Result<Self, KFCReadError> {
        Self::new_with_options(
            path,
            file_name,
            KFCReaderOptions::default()
        )
    }

    pub fn new_with_options(
        path: impl AsRef<Path>,
        file_name: impl AsRef<str>,
        options: KFCReaderOptions,
    ) -> Result<Self, KFCReadError> {
        let path = path.as_ref();
        let file_name = file_name.as_ref();

        let kfc_path = path.join(format!(
            "{}.{}",
            file_name,
            options.kfc_extension
        ));
        let file = KFCFile::from_path(&kfc_path, false)?;

        let resource_path = path.join(format!(
            "{}.{}",
            file_name,
            options.resource_extension
        ));

        let resource_bounds = resource_bounds(&file);
        let resource_chunks_ordered = file.resource_chunks().windows(2).all(|pair| {
            pair[0].uncompressed_offset.checked_add(pair[0].uncompressed_size)
                .is_some_and(|end| end <= pair[1].uncompressed_offset)
        });
        let chunk_cache = (0..file.resource_chunks().len())
            .map(|_| ResourceChunkCacheEntry::default())
            .collect();

        Ok(Self {
            file,
            resource_bounds,
            resource_chunks_ordered,
            chunk_cache,

            path: path.to_path_buf(),
            file_name: file_name.to_string(),
            kfc_path,
            dat_extension: options.dat_extension.to_string(),
            resource_path,
        })
    }

    #[inline]
    pub fn path(&self) -> &Path {
        &self.path
    }

    #[inline]
    pub fn file(&self) -> &KFCFile {
        self.file.borrow()
    }

    #[inline]
    pub fn kfc_path(&self) -> &Path {
        &self.kfc_path
    }

    /// Returns the actual size of a resource.
    ///
    /// The game stores resource sizes in 24 bits. For resources of 16 MiB and
    /// larger the stored size is therefore too small by a multiple of 16 MiB.
    /// The actual size is recovered from the offset of the next resource.
    pub fn resource_size(&self, guid: &ResourceId) -> Option<u64> {
        self.file.resources()
            .get(guid)
            .map(|entry| actual_resource_size(&self.resource_bounds, entry))
    }

    #[inline]
    pub fn new_cursor(&self) -> Result<KFCCursor<&Self>, KFCReadError> {
        KFCCursor::new(self)
    }

    #[inline]
    pub fn into_cursor(self) -> Result<KFCCursor<Self>, KFCReadError> {
        KFCCursor::new(self)
    }

}

pub struct KFCCursor<R> {
    kfc_reader: R,
    reader: BufReader<File>,
    container_readers: Vec<Option<BufReader<File>>>,

    buffer: Vec<u8>,
}

impl<R> KFCCursor<R>
where
    R: Borrow<KFCReader>
{

    fn new(
        kfc_reader: R,
    ) -> Result<Self, KFCReadError> {
        let reader = kfc_reader.borrow();
        let reader = BufReader::new(File::open(&reader.resource_path)?);

        Ok(Self {
            kfc_reader,
            reader,
            container_readers: Vec::new(),

            buffer: Vec::new(),
        })
    }

    #[inline]
    pub fn file(&self) -> &KFCFile {
        self.kfc_reader.borrow().file()
    }

    /// See [`KFCReader::resource_size`].
    #[inline]
    pub fn resource_size(&self, guid: &ResourceId) -> Option<u64> {
        self.kfc_reader.borrow().resource_size(guid)
    }

    // pub fn open_resource(
    //     &self,
    //     id: &ResourceId,
    // ) -> std::io::Result<Option<Take<BufReader<File>>>> {
    //     let kfc_reader = self.kfc_reader.borrow();
    //     let file = &kfc_reader.file;
    //     let resource = match file.resources().get(id) {
    //         Some(resource) => resource,
    //         None => return Ok(None),
    //     };
    //
    //     let offset = file.data_offset() + resource.offset;
    //     let size = resource.size;
    //
    //     let path = &kfc_reader.kfc_path;
    //     let mut reader = BufReader::new(File::open(path)?);
    //
    //     reader.seek(SeekFrom::Start(offset))?;
    //
    //     Ok(Some(reader.take(size)))
    // }

    pub fn read_resource(
        &mut self,
        id: &ResourceId
    ) -> std::io::Result<Option<Vec<u8>>> {
        let mut data = Vec::new();

        if !self.read_resource_into(id, &mut data)? {
            return Ok(None);
        }

        Ok(Some(data))
    }

    pub fn read_resource_into(
        &mut self,
        guid: &ResourceId,
        dst: &mut Vec<u8>
    ) -> std::io::Result<bool> {
        let kfc_reader = self.kfc_reader.borrow();
        let file = &kfc_reader.file;
        let resource = match file.resources().get(guid) {
            Some(resource) => resource,
            None => return Ok(false),
        };

        let resource_size = actual_resource_size(&kfc_reader.resource_bounds, resource);

        if resource_size == 0 {
            return Ok(true);
        }

        let resource_end = resource.offset.checked_add(resource_size)
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidData, "Resource range overflow"))?;
        let chunk_start = resource_chunk_index(
            file.resource_chunks(), kfc_reader.resource_chunks_ordered, resource.offset,
        );

        let chunk_start = match chunk_start {
            Some(chunk) => chunk,
            None => return Ok(false),
        };

        // The resource end is exclusive. Looking up the end itself would also
        // decompress the next chunk when the resource ends at a chunk boundary.
        let chunk_end = resource_chunk_index(
            file.resource_chunks(), kfc_reader.resource_chunks_ordered, resource_end - 1,
        );

        let chunk_end = match chunk_end {
            Some(chunk) => chunk,
            None => return Ok(false),
        };

        dst.reserve_exact(resource_size as usize);

        let mut remaining_size = resource_size;
        let mut resource_offset = resource.offset;

        for i in chunk_start..=chunk_end {
            let base_offset = self.kfc_reader.borrow()
                .file()
                .resource_chunks()[i]
                .uncompressed_offset;

            let chunk_data = self.decompress_chunk(i)?;

            let offset = resource_offset.checked_sub(base_offset)
                .filter(|&offset| offset < chunk_data.len() as u64)
                .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidData, "Resource chunk range has a gap or overlap"))?
                as usize;
            let size = std::cmp::min((chunk_data.len() - offset) as u64, remaining_size);

            dst.extend_from_slice(&chunk_data[offset .. offset + size as usize]);
            remaining_size -= size;
            resource_offset += size;
        }

        if remaining_size != 0 {
            return Err(std::io::Error::new(std::io::ErrorKind::UnexpectedEof, "Incomplete resource chunk range"));
        }

        Ok(true)
    }

    fn decompress_chunk(
        &mut self,
        index: usize
    ) -> std::io::Result<&[u8]> {
        let kfc_reader = self.kfc_reader.borrow();
        let chunk = &kfc_reader.file.resource_chunks()[index];
        let reader = &mut self.reader;
        let buffer = &mut self.buffer;

        kfc_reader.chunk_cache[index].get_or_load(|| {
            buffer.resize(chunk.compressed_size as usize, 0);
            reader.seek(SeekFrom::Start(chunk.offset))?;
            reader.read_exact(buffer)?;

            let mut decompressed_data = Vec::with_capacity(chunk.uncompressed_size as usize);
            zstd::stream::copy_decode(&mut &buffer[..], &mut decompressed_data)?;
            if decompressed_data.len() as u64 != chunk.uncompressed_size {
                return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Decompressed chunk size does not match the index"));
            }
            Ok(decompressed_data)
        })
    }

    pub fn open_content(
        &self,
        hash: &ContentHash
    ) -> std::io::Result<Option<Take<BufReader<File>>>> {
        let kfc_reader = self.kfc_reader.borrow();
        let entry = match kfc_reader.file.contents().get(hash) {
            Some(entry) => entry,
            None => return Ok(None),
        };

        let offset = entry.offset;
        let size = hash.size() as u64;
        let index = entry.container_index;

        let mut container_reader = self.open_container_reader(index)?;

        container_reader.seek(SeekFrom::Start(offset))?;

        Ok(Some(container_reader.take(size)))
    }

    pub fn read_content(
        &mut self,
        hash: &ContentHash
    ) -> std::io::Result<Option<Vec<u8>>> {
        let mut data = Vec::new();

        if !self.read_content_into(hash, &mut data)? {
            return Ok(None);
        }

        Ok(Some(data))
    }

    pub fn read_content_into(
        &mut self,
        hash: &ContentHash,
        dst: &mut Vec<u8>
    ) -> std::io::Result<bool> {
        let kfc_reader = self.kfc_reader.borrow();
        let entry = match kfc_reader.file.contents().get(hash) {
            Some(entry) => entry,
            None => return Ok(false),
        };

        let offset = entry.offset;
        dst.resize(hash.size() as usize, 0);

        let container_reader = self.get_container_reader(entry.container_index)?;

        container_reader.seek(SeekFrom::Start(offset))?;
        container_reader.read_exact(dst)?;

        Ok(true)
    }

    #[inline]
    pub fn kfc_path(&self) -> &Path {
        self.kfc_reader.borrow().kfc_path.as_ref()
    }

    pub fn dat_path(&self, index: usize) -> PathBuf {
        let kfc_reader = self.kfc_reader.borrow();

        // Format: FILE_NAME_{INDEX}.dat where INDEX is 3 digits with leading zeros
        let name = format!(
            "{}_{:03}.{}",
            kfc_reader.file_name,
            index,
            kfc_reader.dat_extension
        );
        kfc_reader.path.join(name)
    }

    fn get_container_reader(
        &mut self,
        index: usize
    ) -> std::io::Result<&mut BufReader<File>> {
        if index >= self.container_readers.len() {
            self.container_readers.resize_with(index + 1, || None);
        }

        if self.container_readers[index].is_none() {
            self.container_readers[index] = Some(
                self.open_container_reader(index)?
            );
        }

        Ok(self.container_readers[index].as_mut().unwrap())
    }

    fn open_container_reader(
        &self,
        index: usize
    ) -> std::io::Result<BufReader<File>> {
        Ok(BufReader::new(File::open(self.dat_path(index))?))
    }

}

fn resource_chunk_index(
    chunks: &[super::header::ResourceChunkInfo],
    ordered: bool,
    offset: u64,
) -> Option<usize> {
    let contains = |chunk: &super::header::ResourceChunkInfo| {
        chunk.uncompressed_offset <= offset
            && chunk.uncompressed_offset.checked_add(chunk.uncompressed_size)
                .is_some_and(|end| offset < end)
    };
    if ordered {
        let index = chunks.partition_point(|chunk| chunk.uncompressed_offset <= offset)
            .checked_sub(1)?;
        contains(&chunks[index]).then_some(index)
    } else {
        // Keep supporting tables that are not ordered by uncompressed offset.
        chunks.iter().position(contains)
    }
}

fn resource_bounds(file: &KFCFile) -> Vec<u64> {
    let mut bounds = file.resources()
        .values()
        .iter()
        .map(|entry| entry.offset)
        .collect::<Vec<_>>();

    let data_end = file.resource_chunks()
        .iter()
        .map(|chunk| chunk.uncompressed_offset + chunk.uncompressed_size)
        .max()
        .unwrap_or(0);

    bounds.sort_unstable();
    bounds.dedup();
    bounds.push(data_end);
    bounds
}

/// Returns the size of the resource, corrected for sizes that were truncated
/// to 24 bits.
///
/// The space up to the next resource must match the stored size plus a
/// multiple of 16 MiB and less than the resource alignment, otherwise the
/// stored size is used as is (e.g. unused space left behind by patching).
fn actual_resource_size(bounds: &[u64], entry: &ResourceEntry) -> u64 {
    let next = match bounds.binary_search(&entry.offset) {
        Ok(index) => bounds.get(index + 1),
        Err(index) => bounds.get(index),
    };

    let Some(&next) = next else {
        return entry.size;
    };

    let Some(gap) = next.checked_sub(entry.offset + entry.size) else {
        return entry.size;
    };

    let missing = gap - gap % RESOURCE_SIZE_MODULUS;

    if missing > 0 && gap - missing < RESOURCE_ALIGNMENT {
        entry.size + missing
    } else {
        entry.size
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(offset: u64, size: u64) -> ResourceEntry {
        ResourceEntry { offset, size }
    }

    #[test]
    fn test_actual_resource_size() {
        let bounds = vec![0, 32, 32 + 0x2000000 + 16, 0x3000000];

        // regular resources keep their stored size
        assert_eq!(actual_resource_size(&bounds, &entry(0, 24)), 24);
        // truncated by 2 * 16 MiB, followed by alignment padding
        assert_eq!(actual_resource_size(&bounds, &entry(32, 12)), 12 + 0x2000000);
        // gap that does not match a truncated size is left alone
        assert_eq!(actual_resource_size(&bounds, &entry(32 + 0x2000000 + 16, 100)), 100);
        // offset not in bounds and past the end
        assert_eq!(actual_resource_size(&bounds, &entry(0x4000000, 8)), 8);
    }

    #[test]
    fn concurrent_chunk_requests_load_once_and_share_the_same_allocation() {
        use std::sync::{Barrier, atomic::{AtomicUsize, Ordering}};

        let cache = ResourceChunkCacheEntry::default();
        let loads = AtomicUsize::new(0);
        let start = Barrier::new(8);
        let pointers = std::thread::scope(|scope| {
            let handles = (0..8).map(|_| {
                scope.spawn(|| {
                    start.wait();
                    let data = cache.get_or_load(|| {
                        loads.fetch_add(1, Ordering::Relaxed);
                        Ok(vec![42; 1024])
                    }).unwrap();
                    assert_eq!(data, &[42; 1024]);
                    data.as_ptr() as usize
                })
            }).collect::<Vec<_>>();
            handles.into_iter().map(|handle| handle.join().unwrap()).collect::<Vec<_>>()
        });

        assert_eq!(loads.load(Ordering::Relaxed), 1);
        assert!(pointers.iter().all(|pointer| *pointer == pointers[0]));
    }

    #[test]
    fn failed_chunk_load_can_be_retried() {
        let cache = ResourceChunkCacheEntry::default();
        let failure = cache.get_or_load(|| Err(std::io::Error::new(
            std::io::ErrorKind::Interrupted, "transient read failure",
        )));
        assert_eq!(failure.unwrap_err().kind(), std::io::ErrorKind::Interrupted);

        assert_eq!(cache.get_or_load(|| Ok(vec![1, 2, 3])).unwrap(), [1, 2, 3]);
        assert_eq!(cache.get_or_load(|| panic!("cached data must not be loaded again")).unwrap(), [1, 2, 3]);
    }

    #[test]
    fn chunk_lookup_uses_exclusive_ends_and_rejects_gaps() {
        use super::super::header::ResourceChunkInfo;

        let chunk = |offset, size| ResourceChunkInfo {
            uncompressed_offset: offset,
            uncompressed_size: size,
            ..Default::default()
        };
        let chunks = [chunk(0, 8), chunk(8, 8), chunk(20, 4)];
        for ordered in [true, false] {
            assert_eq!(resource_chunk_index(&chunks, ordered, 0), Some(0));
            assert_eq!(resource_chunk_index(&chunks, ordered, 7), Some(0));
            assert_eq!(resource_chunk_index(&chunks, ordered, 8), Some(1));
            assert_eq!(resource_chunk_index(&chunks, ordered, 15), Some(1));
            assert_eq!(resource_chunk_index(&chunks, ordered, 16), None);
            assert_eq!(resource_chunk_index(&chunks, ordered, 23), Some(2));
            assert_eq!(resource_chunk_index(&chunks, ordered, 24), None);
        }

        let unordered = [chunk(20, 4), chunk(0, 8), chunk(8, 8)];
        assert_eq!(resource_chunk_index(&unordered, false, 4), Some(1));
    }

    #[test]
    fn resource_reads_cross_boundaries_and_reuse_chunks_across_cursors() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        use super::super::header::{ResourceBundleEntry, ResourceChunkInfo};

        static NEXT_DIRECTORY: AtomicUsize = AtomicUsize::new(0);
        struct TestDirectory(PathBuf);
        impl Drop for TestDirectory {
            fn drop(&mut self) {
                std::fs::remove_dir_all(&self.0).unwrap();
            }
        }

        let directory = TestDirectory(std::env::temp_dir().join(format!(
            "kfc-reader-{}-{}", std::process::id(), NEXT_DIRECTORY.fetch_add(1, Ordering::Relaxed),
        )));
        std::fs::create_dir(&directory.0).unwrap();
        let bytes = (0..128).collect::<Vec<u8>>();
        let mut blob = Vec::new();
        let chunks = bytes.chunks(64).enumerate().map(|(index, data)| {
            let compressed = zstd::encode_all(data, 0).unwrap();
            let chunk = ResourceChunkInfo {
                offset: blob.len() as u64,
                size: compressed.len() as u64,
                compressed_size: compressed.len() as u64,
                uncompressed_offset: index as u64 * 64,
                uncompressed_size: data.len() as u64,
            };
            blob.extend_from_slice(&compressed);
            chunk
        }).collect();
        let first = ResourceId::NONE.with_type_hash(1);
        let crossing = first.with_part_index(1);
        let mut file = KFCFile::default();
        let bundles = std::collections::HashMap::from([(1, ResourceBundleEntry::default())]).into();
        file.set_resources(
            std::collections::HashMap::from([(first, entry(0, 64)), (crossing, entry(60, 8))]).into(),
            &crate::reflection::TypeRegistry::default(), Some(&bundles),
        ).unwrap();
        file.set_resource_chunks(chunks);
        file.write(&mut File::create(directory.0.join("test.kfc")).unwrap()).unwrap();
        std::fs::write(directory.0.join("test.kfc_resources"), &blob).unwrap();

        let reader = KFCReader::new(&directory.0, "test").unwrap();
        {
            let mut cursor = reader.new_cursor().unwrap();
            let mut output = vec![255];
            assert!(cursor.read_resource_into(&first, &mut output).unwrap());
            assert_eq!(&output[1..], &bytes[..64]);
            assert_eq!(output[0], 255);
            // Ending at offset 64 must not load the next chunk.
            assert!(reader.chunk_cache[0].data.get().is_some());
            assert!(reader.chunk_cache[1].data.get().is_none());
            assert_eq!(cursor.read_resource(&crossing).unwrap().unwrap(), bytes[60..68]);
        }

        // A fresh cursor needs no disk data for chunks loaded by the old one.
        std::fs::write(directory.0.join("test.kfc_resources"), []).unwrap();
        let mut cursor = reader.new_cursor().unwrap();
        assert_eq!(cursor.read_resource(&first).unwrap().unwrap(), bytes[..64]);
        assert_eq!(cursor.read_resource(&crossing).unwrap().unwrap(), bytes[60..68]);
    }
}
