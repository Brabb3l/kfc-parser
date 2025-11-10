use std::{borrow::Borrow, fs::File, io::{BufWriter, Cursor, Seek, SeekFrom, Write}, path::{Path, PathBuf}};

use crate::{container::header::ResourceChunkInfo, guid::{ContentHash, ResourceId}, io::{WriteExt, WriteSeekExt}, reflection::TypeRegistry};

use super::{header::{ContentEntry, ContainerInfo, ResourceEntry}, KFCFile, KFCReadError, KFCWriteError, StaticMapBuilder};

const RESOURCE_ALIGNMENT: u64 = 16;
const CONTENT_ALIGNMENT: u64 = 4096;
const RESOURCE_CHUNK_ALIGNMENT: u64 = 4096;
const RESOURCE_CHUNK_SIZE: u64 = 8 * 1024 * 1024; // 8 MiB

pub struct KFCWriter<F, T> {
    type_registry: T,

    path: PathBuf,
    file_name: String,

    file: File,

    chunk_writer: Cursor<Vec<u8>>,
    resource_writer: BufWriter<File>,
    uncompressed_offset: u64,
    uncompressed_chunk_offset: u64,
    resource_chunks: Vec<ResourceChunkInfo>,

    container_writers: Vec<ContainerWriter>,

    game_version: String,
    resources: StaticMapBuilder<ResourceId, ResourceEntry>,
    contents: StaticMapBuilder<ContentHash, ContentEntry>,

    incremental_data: Option<IncrementalData<F>>,
    options: KFCWriteOptions,
}

struct IncrementalData<F> {
    reference_file: F,
}

#[derive(Debug)]
pub struct KFCWriteOptions {
    /// The extension to use for kfc files.
    pub kfc_extension: String,
    /// The extension to use for kfc resource files.
    pub resource_extension: String,
    /// The extension to use for dat files.
    pub dat_extension: String,
    /// If true, the writer will overwrite existing data in the .dat files,
    /// instead of causing an error.
    /// **NOTE:** For incremental writes, this will only affect non-default .dat files.
    pub overwrite_containers: bool,
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
            kfc_extension: "kfc".to_string(),
            resource_extension: "kfc_resources".to_string(),
            dat_extension: "dat".to_string(),
            overwrite_containers: false,
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
    pub fn new(
        path: impl AsRef<Path>,
        file_name: impl AsRef<str>,
        type_registry: T,
        game_version: impl AsRef<str>,
    ) -> Result<Self, KFCReadError> {
        Self::new_with_options(
            path,
            file_name,
            type_registry,
            game_version,
            KFCWriteOptions::default(),
        )
    }

    pub fn new_with_options(
        path: impl AsRef<Path>,
        file_name: impl AsRef<str>,
        type_registry: T,
        game_version: impl AsRef<str>,
        options: KFCWriteOptions,
    ) -> Result<Self, KFCReadError> {
        let kfc_path = path.as_ref().join(format!(
            "{}.{}",
            file_name.as_ref(),
            options.kfc_extension
        ));
        let file = File::options().write(true).read(true).open(&kfc_path)?;

        let resources_path = path.as_ref().join(format!(
            "{}.{}",
            file_name.as_ref(),
            options.resource_extension
        ));
        let resource_file = File::options().write(true).open(&resources_path)?;
        let resource_writer = BufWriter::new(resource_file);

        Ok(Self {
            type_registry,

            path: path.as_ref().to_path_buf(),
            file_name: file_name.as_ref().to_string(),

            file,

            chunk_writer: Cursor::new(Vec::new()),
            resource_writer,
            uncompressed_offset: 0,
            uncompressed_chunk_offset: 0,
            resource_chunks: Vec::new(),

            container_writers: Vec::new(),

            game_version: game_version.as_ref().to_string(),
            resources: StaticMapBuilder::default(),
            contents: StaticMapBuilder::default(),

            incremental_data: None,
            options,
        })
    }

    #[inline]
    pub fn new_incremental(
        path: impl AsRef<Path>,
        file_name: impl AsRef<str>,
        reference_file: F,
        type_registry: T,
    ) -> Result<Self, KFCReadError> {
        Self::new_incremental_with_options(
            path,
            file_name,
            reference_file,
            type_registry,
            KFCWriteOptions::default(),
        )
    }

    pub fn new_incremental_with_options(
        path: impl AsRef<Path>,
        file_name: impl AsRef<str>,
        reference_file: F,
        type_registry: T,
        options: KFCWriteOptions,
    ) -> Result<Self, KFCReadError> {
        let kfc_path = path.as_ref().join(format!(
            "{}.{}",
            file_name.as_ref(),
            options.kfc_extension
        ));
        let current_file = KFCFile::from_path(&kfc_path, true)?;

        let file = reference_file.borrow();

        let resources = file.resources().as_builder();
        let contents = file.contents().as_builder();

        drop(current_file);

        let file = File::options().write(true).read(true).open(&kfc_path)?;

        let resources_path = path.as_ref().join(format!(
            "{}.{}",
            file_name.as_ref(),
            options.resource_extension
        ));
        let resource_file = File::options().write(true).open(&resources_path)?;
        let mut resource_writer = BufWriter::new(resource_file);

        let previous_resource_size = reference_file.borrow()
            .resource_chunks()
            .iter()
            .map(|chunk| chunk.offset + chunk.size)
            .max()
            .unwrap_or(0);

        let previous_uncompressed_size = reference_file.borrow()
            .resource_chunks()
            .iter()
            .map(|chunk| chunk.uncompressed_offset + chunk.uncompressed_size)
            .max()
            .unwrap_or(0);

        resource_writer.seek(SeekFrom::Start(previous_resource_size))?;

        Ok(Self {
            type_registry,

            path: path.as_ref().into(),
            file_name: file_name.as_ref().to_string(),

            file,
            chunk_writer: Cursor::new(Vec::new()),
            resource_writer,
            uncompressed_offset: previous_uncompressed_size,
            uncompressed_chunk_offset: previous_uncompressed_size,
            resource_chunks: Vec::new(),

            container_writers: Vec::new(),

            game_version: reference_file.borrow().game_version().to_string(),
            resources,
            contents,

            incremental_data: Some(IncrementalData {
                reference_file,
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
        mut bytes: &[u8]
    ) -> std::io::Result<()> {
        self.resources.insert(*guid, ResourceEntry {
            offset: self.uncompressed_offset,
            size: bytes.len() as u64
        });

        let previous_offset = self.uncompressed_offset;

        self.uncompressed_offset += bytes.len() as u64;
        self.uncompressed_offset += (RESOURCE_ALIGNMENT - (self.uncompressed_offset % RESOURCE_ALIGNMENT)) % RESOURCE_ALIGNMENT;

        let mut aligned_size = self.uncompressed_offset - previous_offset;

        // write resource data or compress it if needed

        while aligned_size + self.chunk_writer.position() >= RESOURCE_CHUNK_SIZE {
            let available_size = RESOURCE_CHUNK_SIZE - self.chunk_writer.position();
            let split_index = (available_size as usize).min(bytes.len());

            let (chunk_bytes, remaining_bytes) = bytes.split_at(split_index);

            self.chunk_writer.write_all(chunk_bytes)?;
            self.chunk_writer.align(RESOURCE_CHUNK_SIZE as usize)?;

            // compress and submit chunk

            self.submit_resource_data()?;

            // continue with remaining bytes

            bytes = remaining_bytes;
            aligned_size -= available_size;
        }

        if aligned_size > 0 {
            self.chunk_writer.write_all(bytes)?;

            let remaining = aligned_size - bytes.len() as u64;

            if remaining > 0 {
                self.chunk_writer.padding(remaining)?;
            }
        }

        Ok(())
    }

    fn submit_resource_data(&mut self) -> std::io::Result<()> {
        let offset = self.resource_writer.stream_position()?;
        let chunk_data = self.chunk_writer.get_ref();

        zstd::stream::copy_encode(
            &mut &chunk_data[..],
            &mut self.resource_writer,
            0
        )?;

        let compressed_size = self.resource_writer.stream_position()? - offset;

        self.resource_writer.align(RESOURCE_CHUNK_ALIGNMENT as usize)?;

        let size = self.resource_writer.stream_position()? - offset;
        let uncompressed_offset = self.uncompressed_chunk_offset;
        let uncompressed_size = chunk_data.len() as u64;

        self.resource_chunks.push(ResourceChunkInfo {
            offset,
            size,
            compressed_size,
            uncompressed_offset,
            uncompressed_size,
        });

        self.uncompressed_chunk_offset += uncompressed_size;

        // reset chunk writer

        self.chunk_writer.set_position(0);
        self.chunk_writer.get_mut().clear();

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

        if self.contents.contains_key(guid) {
            // if the content already exists, we don't need to write it again since they are unique
            return Ok(());
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
        self.submit_resource_data()?;

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
            let path = self.dat_path(containers.len());

            if !path.exists() || self.options.truncate_containers {
                File::create(path)?;
            }

            containers.push(ContainerInfo {
                size: 0,
                count: 0,
            });
        }

        // prepare resource chunks

        let mut chunks = self.incremental_data.as_ref()
            .map(|data| data.reference_file.borrow().resource_chunks().to_vec())
            .unwrap_or_default();

        chunks.extend(self.resource_chunks.iter().cloned());

        // header construction

        let mut file_writer = BufWriter::new(&mut self.file);
        let mut file = KFCFile::default();

        file.set_game_version(self.game_version);
        file.set_resources(self.resources.build(), self.type_registry.borrow());
        file.set_contents(self.contents.build());
        file.set_containers(containers);
        file.set_resource_chunks(chunks);

        file.write(&mut file_writer)?;

        Ok(())
    }

    pub fn dat_path(&self, index: usize) -> PathBuf {
        // Format: FILE_NAME_{INDEX}.dat where INDEX is 3 digits with leading zeros
        let name = format!(
            "{}_{:03}.{}",
            self.file_name,
            index,
            self.options.dat_extension
        );
        self.path.join(name)
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
        let path = self.dat_path(next_index);

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
