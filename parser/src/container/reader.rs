use std::{fs::File, io::{BufReader, Read, Seek, SeekFrom}, path::{Path, PathBuf}};

use serde_json::Value as JsonValue;

use crate::{guid::{BlobGuid, DescriptorGuid}, reflection::{ReadError, TypeCollection}};

use super::KFCFile;

pub struct KFCReader<'a, 'b> {
    pub path: PathBuf,
    pub file: &'a KFCFile,
    pub type_collection: &'b TypeCollection,

    reader: BufReader<File>,
    dat_readers: Vec<Option<BufReader<File>>>,
}

impl<'a, 'b> KFCReader<'a, 'b> {

    pub fn new(
        path: &Path,
        file: &'a KFCFile,
        type_collection: &'b TypeCollection,
    ) -> std::io::Result<Self> {
        Ok(Self {
            path: path.to_path_buf(),
            file,
            type_collection,
            reader: BufReader::new(File::open(path)?),
            dat_readers: Vec::new(),
        })
    }

    pub fn read_descriptor(
        &mut self,
        guid: &DescriptorGuid
    ) -> Result<Option<JsonValue>, ReadError> {
        let data = match self.read_descriptor_bytes(guid)? {
            Some(data) => data,
            None => return Ok(None),
        };

        Ok(Some(self.type_collection.deserialize_descriptor(guid, &data)?))
    }

    pub fn read_descriptor_into(
        &mut self,
        guid: &DescriptorGuid,
        buf: &mut Vec<u8>
    ) -> Result<Option<JsonValue>, ReadError> {
        if !self.read_descriptor_bytes_into(guid, buf)? {
            return Ok(None);
        }

        Ok(Some(self.type_collection.deserialize_descriptor(guid, buf)?))
    }

    pub fn read_descriptor_bytes(
        &mut self,
        guid: &DescriptorGuid
    ) -> std::io::Result<Option<Vec<u8>>> {
        let mut data = Vec::new();

        if !self.read_descriptor_bytes_into(guid, &mut data)? {
            return Ok(None);
        }

        Ok(Some(data))
    }

    pub fn read_descriptor_bytes_into(
        &mut self,
        guid: &DescriptorGuid,
        dst: &mut Vec<u8>
    ) -> std::io::Result<bool> {
        let link = match self.file.get_descriptor_link(guid) {
            Some(link) => link,
            None => return Ok(false),
        };

        let offset = self.file.data_offset() + link.offset;
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
        let link = match self.file.get_blob_link(guid) {
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
