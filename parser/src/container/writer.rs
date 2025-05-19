use std::{fs::File, io::{BufWriter, Cursor, Read, Seek, SeekFrom, Write}, path::{Path, PathBuf}};

use shared::io::{WriteExt, WriteSeekExt};
use serde_json::Value as JsonValue;

use crate::{guid::{BlobGuid, DescriptorGuid}, reflection::{TypeCollection, WriteError}};

use super::{header::{BlobLink, DatInfo, DescriptorLink}, KFCFile, KFCReadError, KFCWriteError, StaticMapBuilder};

pub struct KFCWriter<'a, 'b> {
    path: PathBuf,
    reference_file: &'a KFCFile,
    type_collection: &'b TypeCollection,

    descriptors: StaticMapBuilder<DescriptorGuid, DescriptorLink>,
    blobs: StaticMapBuilder<BlobGuid, BlobLink>,

    header_space: u64,
    default_data_size: u64,
    default_data_size_unaligned: u64,

    data_writer: Cursor<Vec<u8>>,
    file: File,

    dat_infos: Vec<DatInfo>,
    dat_writer: Option<BufWriter<File>>,
}

impl<'a, 'b> KFCWriter<'a, 'b> {

    pub fn new(
        path: &Path,
        reference_file: &'a KFCFile,
        type_collection: &'b TypeCollection,
    ) -> Result<Self, KFCReadError> {
        let current_file = KFCFile::from_path(path)?;
        let header_space = current_file.data_offset();
        let default_data_size = reference_file.data_size() + (16 - (reference_file.data_size() % 16)) % 16;
        let default_data_size_unaligned = reference_file.data_size();

        drop(current_file);

        let file = File::options().write(true).read(true).open(path)?;

        Ok(Self {
            path: path.to_path_buf(),
            reference_file,
            type_collection,

            descriptors: reference_file.get_descriptor_map().as_builder(),
            blobs: reference_file.get_blob_map().as_builder(),

            header_space,
            default_data_size,
            default_data_size_unaligned,

            data_writer: Cursor::new(Vec::new()),
            file,

            dat_infos: reference_file.get_dat_infos().to_vec(),
            dat_writer: None,
        })
    }

    pub fn write_descriptor(
        &mut self,
        value: &JsonValue
    ) -> Result<(), WriteError> {
        let (guid, data) = self.type_collection.serialize_descriptor(value)?;

        Ok(self.write_descriptor_bytes(&guid, &data)?)
    }

    pub fn write_descriptor_with_guid(
        &mut self,
        guid: &DescriptorGuid,
        value: &JsonValue
    ) -> Result<(), WriteError> {
        let (_, data) = self.type_collection.serialize_descriptor(value)?;

        Ok(self.write_descriptor_bytes(guid, &data)?)
    }

    pub fn write_descriptor_bytes(
        &mut self,
        guid: &DescriptorGuid,
        bytes: &[u8]
    ) -> std::io::Result<()> {
        let offset = self.data_writer.stream_position()? + self.default_data_size;
        self.descriptors.insert(guid.clone(), DescriptorLink {
            offset,
            size: bytes.len() as u64
        });

        self.data_writer.write_all(bytes)?;
        self.data_writer.align(16)?;

        Ok(())
    }

    pub fn write_blob(
        &mut self,
        guid: &BlobGuid,
        data: &[u8],
    ) -> std::io::Result<()> {
        if guid.size() != data.len() as u32 {
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Blob size mismatch"));
        }

        let (dat_writer, dat_index) = self.get_dat_writer()?;
        let offset = dat_writer.stream_position()?;

        dat_writer.write_all(data)?;
        dat_writer.align(4096)?;

        self.blobs.insert(guid.clone(), BlobLink::new(offset, 0, dat_index));

        Ok(())
    }

    pub fn finalize(mut self) -> Result<(), KFCWriteError> {
        self.finalize_dat_info()?;

        let size = self.data_writer.stream_position()? + self.default_data_size;
        let data = self.data_writer.into_inner();

        let mut dat_infos = self.reference_file.get_dat_infos().to_vec();
        dat_infos.extend(self.dat_infos);

        // header construction

        let mut header_writer = BufWriter::new(Cursor::new(Vec::new()));
        let mut file = KFCFile::default();

        file.set_game_version(self.reference_file.game_version().to_string());
        file.set_descriptors(self.descriptors.build(), self.type_collection);
        file.set_blobs(self.blobs.build());
        file.set_dat_infos(self.reference_file.get_dat_infos().to_vec());
        file.set_data_location(0, size);

        file.write(&mut header_writer)?;

        let mut header_size = header_writer.stream_position()?.max(self.header_space);
        let padding = if header_size > self.header_space {
            // add 64KiB padding to reduce consecutive default data movement
            header_size += 0x10000;
            0x10000
        } else {
            0
        };
        let overflow = header_size as i64 - self.header_space as i64;

        file.set_data_location(header_size, size);

        // write data

        if padding > 0 {
            Self::copy_within_file(&mut self.file, self.header_space, self.default_data_size_unaligned, header_size)?;
        } else if overflow < 0 {
            // zero out unused data
            self.file.seek(SeekFrom::Start(header_size))?;
            self.file.padding(-overflow as u64)?;
        }

        let mut file_writer = BufWriter::new(self.file);

        // TODO: Maybe add a function to only update the data location instead of reserializing the kfc file
        file_writer.seek(SeekFrom::Start(0))?;
        file.write(&mut file_writer)?;

        if padding > 0 {
            file_writer.padding(padding)?;
        }

        file_writer.seek(SeekFrom::Current(self.default_data_size as i64))?;
        file_writer.write_all(&data)?;

        Ok(())
    }

    fn copy_within_file(
        file: &mut File,
        src: u64,
        len: u64,
        dst: u64,
    ) -> std::io::Result<()> {
        if src == dst {
            return Ok(());
        }

        if src < dst {
            const BUFFER_SIZE: u64 = 8192;
            let mut buf = vec![0u8; BUFFER_SIZE as usize];
            let mut remaining = len;

            file.seek(SeekFrom::End(0))?;

            while remaining > 0 {
                let chunk_len = BUFFER_SIZE.min(remaining);
                let src_off = src + remaining - chunk_len;
                let dst_off = dst + remaining - chunk_len;
                let chunk = &mut buf[..chunk_len as usize];

                file.seek(SeekFrom::Start(src_off))?;
                file.read_exact(chunk)?;

                file.seek(SeekFrom::Start(dst_off))?;
                file.write_all(chunk)?;

                remaining -= chunk_len;
            }
        } else {
            const BUFFER_SIZE: u64 = 8192;
            let mut buf = vec![0u8; BUFFER_SIZE as usize];
            let mut remaining = len;

            while remaining > 0 {
                let chunk_len = BUFFER_SIZE.min(remaining);
                let src_off = src + len - remaining;
                let dst_off = dst + len - remaining;
                let chunk = &mut buf[..chunk_len as usize];

                file.seek(SeekFrom::Start(src_off))?;
                file.read_exact(chunk)?;

                file.seek(SeekFrom::Start(dst_off))?;
                file.write_all(chunk)?;

                remaining -= chunk_len;
            }
        }

        Ok(())
    }

    // TODO: Support splitting data into multiple dat files
    fn get_dat_writer(&mut self) -> std::io::Result<(&mut BufWriter<File>, usize)> {
        let index = 0;

        if index >= self.dat_infos.len() {
            self.finalize_dat_info()?;

            self.dat_infos.push(DatInfo::default());

            // Format: FILE_NAME_{INDEX}.dat where INDEX is 3 digits with leading zeros
            let path = self.path.with_extension(format!("_{:03}.dat", index));
            self.dat_writer = Some(BufWriter::new(File::create(path)?));
        }

        self.dat_infos[index].count += 1;

        Ok((self.dat_writer.as_mut().unwrap(), index))
    }

    fn finalize_dat_info(&mut self) -> std::io::Result<()> {
        if let Some(writer) = self.dat_writer.as_mut() {
            writer.flush()?;
            self.dat_infos.last_mut().unwrap().size = writer.stream_position()?;
            self.dat_writer = None;
        }

        Ok(())
    }

}
