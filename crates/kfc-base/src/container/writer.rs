use std::{borrow::Borrow, fs::File, io::{BufWriter, Cursor, Read, Seek, SeekFrom, Write}, path::{Path, PathBuf}};

use crate::{guid::{BlobGuid, DescriptorGuid}, io::{WriteExt, WriteSeekExt}, reflection::TypeRegistry};

use super::{header::{BlobLink, DatInfo, DescriptorLink}, KFCFile, KFCReadError, KFCWriteError, StaticMapBuilder};

pub struct KFCWriter<F, T> {
    path: PathBuf,
    reference_file: F,
    type_registry: T,

    descriptors: StaticMapBuilder<DescriptorGuid, DescriptorLink>,
    blobs: StaticMapBuilder<BlobGuid, BlobLink>,

    old_header_space: u64,
    default_data_size: u64,
    default_data_size_unaligned: u64,

    data_writer: Cursor<Vec<u8>>,
    file: File,

    dat_infos: Vec<DatInfo>,
    dat_writer: Option<BufWriter<File>>,
}

impl<F, T> KFCWriter<F, T>
where
    F: Borrow<KFCFile>,
    T: Borrow<TypeRegistry>,
{

    pub fn new<P: AsRef<Path>>(
        path: P,
        reference_file: F,
        type_registry: T,
    ) -> Result<Self, KFCReadError> {
        let current_file = KFCFile::from_path(&path, true)?;
        let header_space = current_file.data_offset();
        let file = reference_file.borrow();
        let default_data_size = file.data_size() + (16 - (file.data_size() % 16)) % 16;
        let default_data_size_unaligned = file.data_size();

        let descriptors = file.get_descriptor_map().as_builder();
        let blobs = file.get_blob_map().as_builder();
        let dat_infos = file.get_dat_infos().to_vec();

        drop(current_file);

        let file = File::options().write(true).read(true).open(&path)?;

        Ok(Self {
            path: path.as_ref().into(),
            reference_file,
            type_registry,

            descriptors,
            blobs,

            old_header_space: header_space,
            default_data_size,
            default_data_size_unaligned,

            data_writer: Cursor::new(Vec::new()),
            file,

            dat_infos,
            dat_writer: None,
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
        self.reference_file.borrow()
    }

    pub fn write_descriptor(
        &mut self,
        guid: &DescriptorGuid,
        bytes: &[u8]
    ) -> std::io::Result<()> {
        let offset = self.data_writer.stream_position()? + self.default_data_size;
        self.descriptors.insert(*guid, DescriptorLink {
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

        self.blobs.insert(*guid, BlobLink::new(offset, 0, dat_index));

        Ok(())
    }

    pub fn finalize(mut self) -> Result<(), KFCWriteError> {
        self.finalize_dat_info()?;

        let size = self.data_writer.stream_position()? + self.default_data_size;
        let data = self.data_writer.into_inner();

        // header construction

        let mut header_writer = BufWriter::new(Cursor::new(Vec::new()));
        let mut file = KFCFile::default();

        file.set_game_version(self.reference_file.borrow().game_version().to_string());
        file.set_descriptors(self.descriptors.build(), self.type_registry.borrow());
        file.set_blobs(self.blobs.build());
        file.set_dat_infos(self.reference_file.borrow().get_dat_infos().to_vec());
        file.set_data_location(0, size);

        file.write(&mut header_writer)?;

        let header_size = header_writer.stream_position()?;
        let mut available_header_space = self.old_header_space;
        let mut padding = 0;

        while available_header_space < header_size {
            // add 64KiB padding to reduce consecutive default data movement
            padding += 0x10000;
            available_header_space += 0x10000;
        }

        file.set_data_location(available_header_space, size);

        // write data

        if padding > 0 {
            Self::copy_within_file(&mut self.file, self.old_header_space, self.default_data_size_unaligned, available_header_space)?;
        }

        let mut file_writer = BufWriter::new(self.file);

        file_writer.seek(SeekFrom::Start(0))?;
        file.write_info(&mut file_writer)?;
        file_writer.padding(available_header_space - header_size)?;

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
            let path = self.path.with_extension(format!("_{index:03}.dat"));
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
