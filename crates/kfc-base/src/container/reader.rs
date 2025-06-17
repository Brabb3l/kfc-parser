use std::{borrow::Borrow, fs::File, io::{BufReader, Read, Seek, SeekFrom}, path::{Path, PathBuf}};

use crate::{guid::{BlobGuid, DescriptorGuid}, reflection::TypeRegistry};

use super::KFCFile;

pub struct KFCReader<F, T> {
    path: PathBuf,
    file: F,
    type_registry: T,

    reader: BufReader<File>,
    dat_readers: Vec<Option<BufReader<File>>>,
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
    ) -> std::io::Result<Self> {
        Ok(Self {
            path: path.as_ref().into(),
            file,
            type_registry,
            reader: BufReader::new(File::open(path)?),
            dat_readers: Vec::new(),
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn type_registry(&self) -> &TypeRegistry {
        self.type_registry.borrow()
    }

    pub fn file(&self) -> &KFCFile {
        self.file.borrow()
    }

    pub fn read_descriptor(
        &mut self,
        guid: &DescriptorGuid
    ) -> std::io::Result<Option<Vec<u8>>> {
        let mut data = Vec::new();

        if !self.read_descriptor_into(guid, &mut data)? {
            return Ok(None);
        }

        Ok(Some(data))
    }

    pub fn read_descriptor_into(
        &mut self,
        guid: &DescriptorGuid,
        dst: &mut Vec<u8>
    ) -> std::io::Result<bool> {
        let file = self.file.borrow();
        let link = match file.get_descriptor_link(guid) {
            Some(link) => link,
            None => return Ok(false),
        };

        let offset = file.data_offset() + link.offset;
        dst.resize(link.size as usize, 0);
        self.reader.seek(SeekFrom::Start(offset))?;
        self.reader.read_exact(dst)?;

        Ok(true)
    }

    pub fn read_blob(&mut self, guid: &BlobGuid) -> std::io::Result<Option<Vec<u8>>> {
        let mut data = Vec::new();

        if !self.read_blob_into(guid, &mut data)? {
            return Ok(None);
        }

        Ok(Some(data))
    }

    pub fn read_blob_into(
        &mut self,
        guid: &BlobGuid,
        dst: &mut Vec<u8>
    ) -> std::io::Result<bool> {
        let link = match self.file.borrow().get_blob_link(guid) {
            Some(link) => link,
            None => return Ok(false),
        };

        let offset = link.offset;
        dst.resize(guid.size() as usize, 0);

        let dat_reader = self.get_dat_reader(link.dat_index)?;

        dat_reader.seek(SeekFrom::Start(offset))?;
        dat_reader.read_exact(dst)?;

        Ok(true)
    }

    fn get_dat_reader(&mut self, index: usize) -> std::io::Result<&mut BufReader<File>> {
        if index >= self.dat_readers.len() {
            self.dat_readers.resize_with(index + 1, || None);
        }

        if self.dat_readers[index].is_none() {
            // Format: FILE_NAME_{INDEX}.dat where INDEX is 3 digits with leading zeros
            let path = self.path.with_file_name(format!("{}_{:03}.dat", self.path.file_stem().unwrap().to_string_lossy(), index));
            self.dat_readers[index] = Some(BufReader::new(File::open(path)?));
        }

        Ok(self.dat_readers[index].as_mut().unwrap())
    }

}
