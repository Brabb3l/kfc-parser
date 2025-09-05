use std::{borrow::Borrow, fs::File, io::{BufReader, Read, Seek, SeekFrom, Take}, path::{Path, PathBuf}};

use crate::{container::KFCReadError, guid::{ContentHash, ResourceId}};

use super::KFCFile;

pub struct KFCReader {
    file: KFCFile,

    path: PathBuf,
    file_name: String,
    kfc_path: PathBuf,
    dat_extension: String,
}

#[derive(Debug)]
pub struct KFCReaderOptions<'a> {
    pub kfc_extension: &'a str,
    pub dat_extension: &'a str,
}

impl Default for KFCReaderOptions<'_> {
    fn default() -> Self {
        Self {
            kfc_extension: "kfc",
            dat_extension: "dat",
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

        Ok(Self {
            file,

            path: path.to_path_buf(),
            file_name: file_name.to_string(),
            kfc_path,
            dat_extension: options.dat_extension.to_string(),
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
}

impl<R> KFCCursor<R>
where
    R: Borrow<KFCReader>
{

    fn new(
        kfc_reader: R,
    ) -> Result<Self, KFCReadError> {
        let reader = kfc_reader.borrow();
        let reader = BufReader::new(File::open(&reader.kfc_path)?);

        Ok(Self {
            kfc_reader,
            reader,
            container_readers: Vec::new(),
        })
    }

    #[inline]
    pub fn file(&self) -> &KFCFile {
        self.kfc_reader.borrow().file()
    }

    pub fn open_resource(
        &self,
        id: &ResourceId,
    ) -> std::io::Result<Option<Take<BufReader<File>>>> {
        let kfc_reader = self.kfc_reader.borrow();
        let file = &kfc_reader.file;
        let resource = match file.resources().get(id) {
            Some(resource) => resource,
            None => return Ok(None),
        };

        let offset = file.data_offset() + resource.offset;
        let size = resource.size;

        let path = &kfc_reader.kfc_path;
        let mut reader = BufReader::new(File::open(path)?);

        reader.seek(SeekFrom::Start(offset))?;

        Ok(Some(reader.take(size)))
    }

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

        let offset = file.data_offset() + resource.offset;

        dst.resize(resource.size as usize, 0);

        self.reader.seek(SeekFrom::Start(offset))?;
        self.reader.read_exact(dst)?;

        Ok(true)
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
