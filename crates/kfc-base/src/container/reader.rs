use std::{borrow::Borrow, collections::HashMap, fs::File, io::{BufReader, Read, Seek, SeekFrom, Take}, path::{Path, PathBuf}};

use crate::{container::KFCReadError, guid::{ContentHash, ResourceId}};

use super::KFCFile;

pub struct KFCReader {
    file: KFCFile,

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

        Ok(Self {
            file,

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

        // TEMPORARY: until the game devs fix this resource size issue
        const RID: ResourceId = ResourceId::parse_qualified("509feadb-4c60-425f-9c7c-deeefd9b6920_21b2a090_3").unwrap();

        let resource_size = if guid == &RID && resource.size < 0x1000000 {
            resource.size + 0x1000000
        } else {
            resource.size
        };
        // END TEMPORARY

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
