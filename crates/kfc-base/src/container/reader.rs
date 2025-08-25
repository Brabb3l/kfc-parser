use std::{borrow::Borrow, fs::File, io::{BufReader, Read, Result, Seek, SeekFrom}, path::{Path, PathBuf}};

use crate::{guid::{ContentHash, ResourceId}, reflection::TypeRegistry};

use super::KFCFile;

pub struct KFCReader<F, T> {
    path: PathBuf,
    file: F,
    type_registry: T,

    reader: BufReader<File>,
    container_readers: Vec<Option<BufReader<File>>>,
}

#[derive(Debug, Default)]
pub struct KFCReaderOptions<'a> {
    pub file_name: Option<std::borrow::Cow<'a, str>>,
}

impl<F, T> KFCReader<F, T>
where
    F: Borrow<KFCFile>,
    T: Borrow<TypeRegistry>,
{

    pub fn new<P: AsRef<Path>>(
        path: P,
        file: F,
        type_registry: T,
    ) -> Result<Self> {
        Self::new_with_options(path, file, type_registry, KFCReaderOptions::default())
    }

    pub fn new_with_options<P: AsRef<Path>>(
        path: P,
        file: F,
        type_registry: T,
        options: KFCReaderOptions,
    ) -> Result<Self> {
        let reader = BufReader::new(File::open(&path)?);
        let path = if let Some(file_name) = options.file_name {
            path.as_ref().with_file_name(file_name.as_ref())
        } else {
            path.as_ref().to_path_buf()
        };

        Ok(Self {
            path,
            file,
            type_registry,
            reader,
            container_readers: Vec::new(),
        })
    }

    #[inline]
    pub fn path(&self) -> &Path {
        &self.path
    }

    #[inline]
    pub fn type_registry(&self) -> &TypeRegistry {
        self.type_registry.borrow()
    }

    #[inline]
    pub fn file(&self) -> &KFCFile {
        self.file.borrow()
    }

    pub fn read_resource(
        &mut self,
        guid: &ResourceId
    ) -> Result<Option<Vec<u8>>> {
        let mut data = Vec::new();

        if !self.read_resource_into(guid, &mut data)? {
            return Ok(None);
        }

        Ok(Some(data))
    }

    pub fn read_resource_into(
        &mut self,
        guid: &ResourceId,
        dst: &mut Vec<u8>
    ) -> Result<bool> {
        let file = self.file.borrow();
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

    pub fn read_content(&mut self, guid: &ContentHash) -> Result<Option<Vec<u8>>> {
        let mut data = Vec::new();

        if !self.read_content_into(guid, &mut data)? {
            return Ok(None);
        }

        Ok(Some(data))
    }

    pub fn read_content_into(
        &mut self,
        guid: &ContentHash,
        dst: &mut Vec<u8>
    ) -> Result<bool> {
        let entry = match self.file.borrow().contents().get(guid) {
            Some(entry) => entry,
            None => return Ok(false),
        };

        let offset = entry.offset;
        dst.resize(guid.size() as usize, 0);

        let container_reader = self.get_container_reader(entry.container_index)?;

        container_reader.seek(SeekFrom::Start(offset))?;
        container_reader.read_exact(dst)?;

        Ok(true)
    }

    fn get_container_reader(
        &mut self,
        index: usize
    ) -> Result<&mut BufReader<File>> {
        if index >= self.container_readers.len() {
            self.container_readers.resize_with(index + 1, || None);
        }

        if self.container_readers[index].is_none() {
            // Format: FILE_NAME_{INDEX}.dat where INDEX is 3 digits with leading zeros
            let path = self.path.with_file_name(format!("{}_{:03}.dat", self.path.file_stem().unwrap().to_string_lossy(), index));
            self.container_readers[index] = Some(BufReader::new(File::open(path)?));
        }

        Ok(self.container_readers[index].as_mut().unwrap())
    }

}
