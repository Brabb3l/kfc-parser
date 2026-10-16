use std::{borrow::Borrow, collections::HashMap, fs::File, io::{BufReader, Read, Seek, SeekFrom, Take}, path::{Path, PathBuf}};

use crate::{container::KFCReadError, guid::{ContentHash, ResourceId}};

use super::{header::ResourceEntry, KFCFile};

/// Resource sizes are stored in 24 bits, so sizes of 16 MiB and larger are
/// truncated to their lower 24 bits.
const RESOURCE_SIZE_MODULUS: u64 = 0x1000000;
const RESOURCE_ALIGNMENT: u64 = 16;

pub struct KFCReader {
    file: KFCFile,
    /// Sorted, distinct resource offsets followed by the end of the resource data.
    resource_bounds: Vec<u64>,

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

        Ok(Self {
            file,
            resource_bounds,

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

    chunk_cache: HashMap<usize, Vec<u8>>,
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

            chunk_cache: HashMap::new(),
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

        let chunk_start = file.resource_chunks()
            .iter()
            .position(|chunk| {
                (chunk.uncompressed_offset..chunk.uncompressed_offset + chunk.uncompressed_size)
                    .contains(&resource.offset)
            });

        let chunk_start = match chunk_start {
            Some(chunk) => chunk,
            None => return Ok(false),
        };

        let chunk_end = file.resource_chunks()
            .iter()
            .rposition(|chunk| {
                (chunk.uncompressed_offset..=chunk.uncompressed_offset + chunk.uncompressed_size)
                    .contains(&(resource.offset + resource_size))
            });

        let chunk_end = match chunk_end {
            Some(chunk) => chunk,
            None => return Ok(false),
        };

        dst.reserve_exact(resource_size.saturating_sub(dst.len() as u64) as usize);

        let mut remaining_size = resource_size;
        let resource_offset = resource.offset;

        for i in chunk_start..=chunk_end {
            let base_offset = self.kfc_reader.borrow()
                .file()
                .resource_chunks()[i]
                .uncompressed_offset;

            let chunk_data = self.decompress_chunk(i)?;

            let size = if resource_offset > base_offset {
                let available = chunk_data.len() as u64 - (resource_offset - base_offset);
                std::cmp::min(available, remaining_size)
            } else {
                let available = chunk_data.len() as u64;
                std::cmp::min(available, remaining_size)
            };

            let offset = if resource_offset > base_offset {
                (resource_offset - base_offset) as usize
            } else {
                0
            };

            dst.extend_from_slice(&chunk_data[offset .. offset + size as usize]);
            remaining_size -= size;
        }

        Ok(true)
    }

    fn decompress_chunk(
        &mut self,
        index: usize
    ) -> std::io::Result<&[u8]> {
        self.decompress_chunk_impl(index)?;
        Ok(self.chunk_cache.get(&index).unwrap())
    }

    fn decompress_chunk_impl(
        &mut self,
        index: usize
    ) -> std::io::Result<()> {
        match self.chunk_cache.get(&index) {
            Some(_) => Ok(()),
            None => {
                let kfc_reader = self.kfc_reader.borrow();
                let chunk = &kfc_reader.file.resource_chunks()[index];

                self.buffer.clear();
                self.buffer.resize(chunk.compressed_size as usize, 0);

                self.reader.seek(SeekFrom::Start(chunk.offset))?;
                self.reader.read_exact(&mut self.buffer[..chunk.compressed_size as usize])?;

                let mut decompressed_data = Vec::with_capacity(chunk.uncompressed_size as usize);
                zstd::stream::copy_decode(
                    &mut &self.buffer[..],
                    &mut decompressed_data
                )?;

                self.chunk_cache.insert(index, decompressed_data);

                Ok(())
            }
        }
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
}
