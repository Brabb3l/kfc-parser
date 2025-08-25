use std::{borrow::Borrow, fs::File, io::{BufWriter, Cursor, Read, Seek, SeekFrom, Write}, path::{Path, PathBuf}};

use crate::{guid::{ContentHash, ResourceId}, io::{WriteExt, WriteSeekExt}, reflection::TypeRegistry};

use super::{header::{ContentEntry, ContainerInfo, ResourceEntry}, KFCFile, KFCReadError, KFCWriteError, StaticMapBuilder};

const RESOURCE_ALIGNMENT: u64 = 16;
const CONTENT_ALIGNMENT: u64 = 4096;

pub struct KFCWriter<F, T> {
    path: PathBuf,
    type_registry: T,

    file: File,
    data_writer: Cursor<Vec<u8>>,
    container_writers: Vec<ContainerWriter>,

    game_version: String,
    resources: StaticMapBuilder<ResourceId, ResourceEntry>,
    contents: StaticMapBuilder<ContentHash, ContentEntry>,

    incremental_data: Option<IncrementalData<F>>,
    options: KFCWriteOptions,
}

struct IncrementalData<F> {
    reference_file: F,

    old_header_space: u64,
    default_data_size: u64,
    default_data_size_unaligned: u64,
}

#[derive(Debug)]
pub struct KFCWriteOptions {
    /// If true, the writer will overwrite existing data in the .dat files,
    /// instead of causing an error.
    /// **NOTE:** For incremental writes, this will only affect non-default .dat files.
    pub overwrite_containers: bool,
    /// The amount of header bytes to reserve for incremental writes.
    /// This is used to prevent moving existing resource data around excessively.
    pub incremental_reserve: u64,
    // TEST: does the game use a i32, u32 or an artifical limit for the dat max size?
    /// The maximum size of a single .dat file before a new one is created.
    pub max_container_size: u32,
    /// Whether to truncate existing .dat files or not.
    /// This is used to remove any leftover data from previous writes.
    /// **NOTE:** For incremental writes, this will only affect non-default .dat files.
    pub truncate_containers: bool,
}

impl Default for KFCWriteOptions {
    fn default() -> Self {
        Self {
            overwrite_containers: false,
            incremental_reserve: 64 * 1024, // 64 KiB
            max_container_size: 1024 * 1024 * 1024, // 1 GiB
            truncate_containers: false,
        }
    }
}

impl<F, T> KFCWriter<F, T>
where
    F: Borrow<KFCFile>,
    T: Borrow<TypeRegistry>,
{

    #[inline]
    pub fn new<P: AsRef<Path>, S: AsRef<str>>(
        path: P,
        type_registry: T,
        game_version: S,
    ) -> Result<Self, KFCReadError> {
        Self::new_with_options(
            path,
            type_registry,
            game_version,
            KFCWriteOptions::default(),
        )
    }

    pub fn new_with_options<P: AsRef<Path>, S: AsRef<str>>(
        path: P,
        type_registry: T,
        game_version: S,
        options: KFCWriteOptions,
    ) -> Result<Self, KFCReadError> {
        let file = File::options().write(true).read(true).open(&path)?;

        Ok(Self {
            path: path.as_ref().into(),
            type_registry,

            file,
            data_writer: Cursor::new(Vec::new()),
            container_writers: Vec::new(),

            game_version: game_version.as_ref().to_string(),
            resources: StaticMapBuilder::default(),
            contents: StaticMapBuilder::default(),

            incremental_data: None,
            options,
        })
    }

    #[inline]
    pub fn new_incremental<P: AsRef<Path>>(
        path: P,
        reference_file: F,
        type_registry: T,
    ) -> Result<Self, KFCReadError> {
        Self::new_incremental_with_options(
            path,
            reference_file,
            type_registry,
            KFCWriteOptions::default(),
        )
    }

    pub fn new_incremental_with_options<P: AsRef<Path>>(
        path: P,
        reference_file: F,
        type_registry: T,
        options: KFCWriteOptions,
    ) -> Result<Self, KFCReadError> {
        let current_file = KFCFile::from_path(&path, true)?;
        let header_space = current_file.data_offset();
        let file = reference_file.borrow();
        let default_data_size = file.data_size() +
            (RESOURCE_ALIGNMENT - (file.data_size() % RESOURCE_ALIGNMENT)) % RESOURCE_ALIGNMENT;
        let default_data_size_unaligned = file.data_size();

        let resources = file.resources().as_builder();
        let contents = file.contents().as_builder();

        drop(current_file);

        let file = File::options().write(true).read(true).open(&path)?;

        Ok(Self {
            path: path.as_ref().into(),
            type_registry,

            file,
            data_writer: Cursor::new(Vec::new()),
            container_writers: Vec::new(),

            game_version: reference_file.borrow().game_version().to_string(),
            resources,
            contents,

            incremental_data: Some(IncrementalData {
                reference_file,

                old_header_space: header_space,
                default_data_size,
                default_data_size_unaligned,
            }),
            options,
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
    pub fn reference_file(&self) -> Option<&KFCFile> {
        self.incremental_data.as_ref()
            .map(|data| data.reference_file.borrow())
    }

    pub fn write_resource(
        &mut self,
        guid: &ResourceId,
        bytes: &[u8]
    ) -> std::io::Result<()> {
        let base_offset = self.incremental_data.as_ref()
            .map(|data| data.default_data_size)
            .unwrap_or(0);
        let offset = self.data_writer.stream_position()? + base_offset;

        self.resources.insert(*guid, ResourceEntry {
            offset,
            size: bytes.len() as u64
        });

        self.data_writer.write_all(bytes)?;
        self.data_writer.align(RESOURCE_ALIGNMENT as usize)?;

        Ok(())
    }

    pub fn write_content(
        &mut self,
        guid: &ContentHash,
        data: &[u8],
    ) -> std::io::Result<()> {
        if guid.size() != data.len() as u32 {
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Content size mismatch"));
        }

        let container_writer = self.get_container_writer()?;
        let index = container_writer.index;

        let writer = container_writer.aquire()?;
        let offset = writer.stream_position()?;

        writer.write_all(data)?;
        writer.align(CONTENT_ALIGNMENT as usize)?;

        self.contents.insert(*guid, ContentEntry::new(offset, 0, index));

        Ok(())
    }

    pub fn finalize(mut self) -> Result<(), KFCWriteError> {
        let base_size = self.incremental_data.as_ref()
            .map(|data| data.default_data_size)
            .unwrap_or(0);
        let size = base_size + self.data_writer.stream_position()?;

        // prepare container infos

        let mut containers = self.incremental_data.as_ref()
            .map(|data| data.reference_file.borrow().containers().to_vec())
            .unwrap_or_default();

        containers.reserve_exact(self.container_writers.len());

        self.container_writers.sort_by_key(|w| w.index);

        for container_writer in &mut self.container_writers {
            container_writer.flush()?;

            containers.push(ContainerInfo {
                size: container_writer.position()?,
                count: container_writer.count,
            });
        }

        // make sure to have a power of two container files

        let required_container_count = containers.len().next_power_of_two();

        while containers.len() < required_container_count {
            let path = self.get_container_file_path(containers.len());

            if !path.exists() || self.options.truncate_containers {
                File::create(path)?;
            }

            containers.push(ContainerInfo {
                size: 0,
                count: 0,
            });
        }

        // header construction

        let mut header_writer = BufWriter::new(Cursor::new(Vec::new()));
        let mut file = KFCFile::default();

        file.set_game_version(self.game_version);
        file.set_resources(self.resources.build(), self.type_registry.borrow());
        file.set_contents(self.contents.build());
        file.set_containers(containers);
        file.set_data_location(0, size);

        file.write(&mut header_writer)?;

        if let Some(incremental_data) = self.incremental_data {
            let header_size = header_writer.stream_position()?;
            let mut available_header_space = incremental_data.old_header_space;
            let mut padding = 0;

            while available_header_space < header_size {
                // add padding to reduce consecutive default data movement
                padding += self.options.incremental_reserve;
                available_header_space += self.options.incremental_reserve;
            }

            file.set_data_location(available_header_space, size);

            // write data

            if padding > 0 {
                copy_within_file(
                    &mut self.file,
                    incremental_data.old_header_space,
                    incremental_data.default_data_size_unaligned,
                    available_header_space
                )?;
            }

            let mut file_writer = BufWriter::new(self.file);
            let data = self.data_writer.into_inner();

            file_writer.seek(SeekFrom::Start(0))?;
            file.write_info(&mut file_writer)?;
            file_writer.padding(available_header_space - header_size)?;

            file_writer.seek(SeekFrom::Current(incremental_data.default_data_size as i64))?;
            file_writer.write_all(&data)?;
        } else {
            file.set_data_location(header_writer.stream_position()?, size);

            // write data

            let mut file_writer = BufWriter::new(self.file);
            let data = self.data_writer.into_inner();

            file_writer.seek(SeekFrom::Start(0))?;
            file.write_info(&mut file_writer)?;
            file_writer.write_all(&data)?;
        }

        Ok(())
    }

    fn get_container_writer(&mut self) -> std::io::Result<&mut ContainerWriter> {
        if !self.container_writers.is_empty() {
            let writer = self.container_writers.last_mut().unwrap();
            let pos = writer.position()?;

            if pos < self.options.max_container_size as u64 {
                return Ok(self.container_writers.last_mut().unwrap());
            }
        }

        // create a new container writer

        let base_index = self.incremental_data.as_ref()
            .map(|data| data.reference_file.borrow().containers().len())
            .unwrap_or(0);
        let next_index = base_index + self.container_writers.len();
        let path = self.get_container_file_path(next_index);

        if !self.options.overwrite_containers && path.exists() {
            // make sure we don't accidentally overwrite an existing file
            return Err(std::io::Error::new(
                std::io::ErrorKind::AlreadyExists,
                format!("Data file already exists: {}", path.display())
            ));
        }

        let writer = BufWriter::new(File::create(&path)?);
        let writer = ContainerWriter::new(next_index, writer);

        self.container_writers.push(writer);

        Ok(self.container_writers.last_mut().unwrap())
    }

    fn get_container_file_path(&self, index: usize) -> PathBuf {
        let mut base_path = self.path.with_extension("").into_os_string();
        base_path.push(format!("_{index:03}.dat"));
        PathBuf::from(base_path)
    }

}

struct ContainerWriter {
    index: usize,
    count: usize,
    writer: BufWriter<File>,
}

impl ContainerWriter {

    #[inline]
    fn new(index: usize, writer: BufWriter<File>) -> Self {
        Self {
            index,
            count: 0,
            writer,
        }
    }

    #[inline]
    fn aquire(&mut self) -> std::io::Result<&mut BufWriter<File>> {
        self.count += 1;
        Ok(&mut self.writer)
    }

    #[inline]
    fn position(&mut self) -> std::io::Result<u64> {
        self.writer.stream_position()
    }

    #[inline]
    fn flush(&mut self) -> std::io::Result<()> {
        self.writer.flush()
    }

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
