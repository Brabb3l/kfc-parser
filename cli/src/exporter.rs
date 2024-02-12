use std::collections::HashMap;
use std::fs::File;
use std::io;
use std::io::{Cursor, Read, Seek, SeekFrom, Write};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use std::thread::JoinHandle;
use bitflags::bitflags;

use crossbeam_channel::Sender;

use parser::container::{KFCDir, KFCDirEntry};
use parser::crpf::Crpf;
use parser::file::FromCrpf;
use parser::file::structs::{ItemIconRegistryResource, RenderMaterialResource, SoundResource};
use shared::io::Reader;


pub struct KFCMultiExporter {
    kfc_reader: KFCDataReader<File>,
    task_tx: Sender<Task>,
    pending_count: Arc<AtomicU32>,
    handles: Vec<JoinHandle<()>>,
    total_tasks: AtomicU32,
}

pub struct KFCDataReader<T> {
    pub reader: Reader<T>,
    pub files: HashMap<u64, KFCDirEntry>,
}

bitflags! {
    #[derive(Default, PartialEq, Eq, Clone, Copy)]
    pub struct TaskMode: u32 {
        const NONE = 0;
        const EXPORT_CRPF = 1;
        const EXPORT_CRPF_DEBUG = 2;
        const EXPORT_CRPF_KBF = 4;
        const EXPORT_CRPF_PLAIN = 8;
        const TRY_UNPACK = 16;
        const EXPORT_BIN_DATA = 32;
    }
}

pub struct Task {
    entry: KFCDirEntry,
    output: String,
    mode: TaskMode,
    types: Vec<String>,
}

impl KFCMultiExporter {
    pub fn new(
        kfc_dir: KFCDir,
        data_file: PathBuf,
        thread_count: u32
    ) -> Self {
        let kfc_reader = KFCDataReader::new(File::open(&data_file).unwrap(), &kfc_dir);
        let mut handles = Vec::new();
        let (task_tx, task_rx) = crossbeam_channel::unbounded();
        let pending_count = Arc::new(AtomicU32::new(0));

        for _ in 0..thread_count {
            let handle = std::thread::spawn({
                let kfc_dir = kfc_dir.clone();
                let pending_count = pending_count.clone();
                let task_rx = task_rx.clone();
                let data_file = data_file.clone();

                move || {
                    let data_input_bytes = File::open(data_file).unwrap();
                    let mut data_reader = KFCDataReader::new(data_input_bytes, &kfc_dir);

                    while let Ok(task) = task_rx.recv() {
                        Self::unpack(&mut data_reader, task, pending_count.clone()).unwrap();
                        pending_count.fetch_sub(1, Ordering::SeqCst);
                    }
                }
            });

            handles.push(handle);
        }

        Self {
            kfc_reader,
            task_tx,
            pending_count,
            handles,
            total_tasks: AtomicU32::new(0),
        }
    }

    pub fn add_task(
        &self,
        file_hash: u64,
        output: String,
        mode: TaskMode,
        types: Vec<String>
    ) {
        self.total_tasks.fetch_add(1, Ordering::SeqCst);
        self.pending_count.fetch_add(1, Ordering::SeqCst);
        self.task_tx.send(Task {
            entry: self.kfc_reader.file_info(file_hash).unwrap().clone(),
            output,
            mode,
            types,
        }).unwrap();
    }

    pub fn export(&mut self, output: &str, mode: TaskMode, types: Vec<String>) -> anyhow::Result<()> {
        for entry in self.kfc_reader.files.values() {
            self.kfc_reader.reader.seek(SeekFrom::Start(entry.offset))?;

            let mut magic = [0; 4];
            self.kfc_reader.reader.read_exact(&mut magic)?;

            if &magic == b"CRPF" {
                self.add_task(entry.name_hash, output.to_string(), mode, types.clone());
            }
        }

        Ok(())
    }

    fn unpack(
        data_reader: &mut KFCDataReader<File>,
        task: Task,
        _pending_count: Arc<AtomicU32>
    ) -> anyhow::Result<()> {
        // prepare crpf data

        let mut crpf_data = vec![0; task.entry.compressed_size as usize];

        data_reader.reader.seek(SeekFrom::Start(task.entry.offset))?;
        data_reader.reader.read_exact(&mut crpf_data)?;

        let crpf = Crpf::read(&mut Reader::new(Cursor::new(&crpf_data)))?;
        let crpf_node = crpf.parse()?;

        let type_name = crpf.get_name(crpf.ctcb.type_entries[0].name0_offset)?;

        if !task.types.is_empty() && !task.types.contains(type_name) {
            return Ok(());
        }

        // prepare naming

        let name = crpf.name.clone().replace('/', "_");
        let name = format!("{}/{}/{} ({})", task.output, type_name, name, task.entry.name_hash);

        // write stuff

        if task.mode.contains(TaskMode::EXPORT_CRPF) {
            if let Some(parent) = PathBuf::from(&name).parent() {
                std::fs::create_dir_all(parent)?;
            }

            let name = format!("{}.crpf", &name);

            println!("Exporting {}", name);

            let mut output = File::create(&name)?;
            output.write_all(&crpf_data)?;
        }

        if task.mode.contains(TaskMode::EXPORT_CRPF_DEBUG) {
            if let Some(parent) = PathBuf::from(&name).parent() {
                std::fs::create_dir_all(parent)?;
            }

            let debug_name = format!("{}.crpf.debug.txt", &name);

            println!("Exporting {}", debug_name);

            let mut debug_output = File::create(&debug_name)?;
            debug_output.write_all(format!("{:#?}", crpf).as_bytes())?;
        }

        if task.mode.contains(TaskMode::EXPORT_CRPF_KBF) {
            if let Some(parent) = PathBuf::from(&name).parent() {
                std::fs::create_dir_all(parent)?;
            }

            let kbf_name = format!("{}.crpf.kbf", &name);

            println!("Exporting {}", kbf_name);

            let mut kbf_output = File::create(&kbf_name)?;
            kbf_output.write_all(&crpf.kbf.data)?;
        }

        if task.mode.contains(TaskMode::EXPORT_CRPF_PLAIN) {
            if let Some(parent) = PathBuf::from(&name).parent() {
                std::fs::create_dir_all(parent)?;
            }

            let plain_name = format!("{}.crpf.txt", &name);

            println!("Exporting {}", plain_name);

            let mut plain_output = File::create(&plain_name)?;
            plain_output.write_all(format!("{:#?}", crpf_node).as_bytes())?;
        }

        if task.mode.contains(TaskMode::EXPORT_BIN_DATA) {
            if let Some(parent) = PathBuf::from(&name).parent() {
                std::fs::create_dir_all(parent)?;
            }

            for (i, data_guid) in crpf.data_guids.iter().enumerate() {
                let mut data = vec![0; data_reader.seek_file(data_guid.hash())? as usize];
                data_reader.read_file(data_guid.hash(), &mut data)?;

                let data_name = format!("{}.data.{}.bin", &name, i);

                println!("Exporting {}", data_name);

                let mut data_output = File::create(&data_name)?;
                data_output.write_all(&data)?;
            }
        }

        if task.mode.contains(TaskMode::TRY_UNPACK) {
            if type_name == "SoundResource" {
                if let Some(parent) = PathBuf::from(&name).parent() {
                    std::fs::create_dir_all(parent)?;
                }

                let sound_resource = SoundResource::parse(&crpf_node, &crpf, None)?;
                let output = format!("{}.wav", &name);

                println!("Exporting {}", output);

                let mut output = io::BufWriter::new(File::create(output)?);

                data_reader.seek_file(sound_resource.data_hash.hash())?;
                sound_resource.export_content(&mut data_reader.reader, &mut output)?;
            } else if type_name == "RenderMaterialResource" {
                if let Some(parent) = PathBuf::from(&name).parent() {
                    std::fs::create_dir_all(parent)?;
                }

                let render_material_resource = RenderMaterialResource::parse(&crpf_node, &crpf, None)?;

                for image in render_material_resource.images {
                    let output = format!("{}.{}.png", &name, image.debug_name);

                    println!("Exporting {}", output);

                    let mut data = vec![0; data_reader.seek_file(image.data.hash())? as usize];
                    data_reader.read_file(image.data.hash(), &mut data)?;
                    image.export_content(&data).unwrap().save(&output)?;
                }
            } else if type_name == "ItemIconRegistryResource" {
                if let Some(parent) = PathBuf::from(&name).parent() {
                    std::fs::create_dir_all(parent)?;
                }

                let item_icon_registry_resource = ItemIconRegistryResource::parse(&crpf_node, &crpf, None)?;

                for (i, icon) in item_icon_registry_resource.icons.iter().enumerate() {
                    let output = format!("{}.{}.png", &name, i);

                    println!("Exporting {}", output);

                    let mut data = vec![0; data_reader.seek_file(icon.ui_texture.data.hash())? as usize];
                    data_reader.read_file(icon.ui_texture.data.hash(), &mut data)?;
                    icon.ui_texture.export_content(&data).unwrap().save(&output)?;
                }
            } else {
                // println!("CRPF \"{}\" is type of {}", name, type_name);
            }
        }

        // println!("{} remaining tasks", pending_count.load(Ordering::SeqCst));

        Ok(())
    }

    pub fn join(self) {
        drop(self.task_tx);

        for handle in self.handles {
            handle.join().unwrap();
        }
    }
}

impl<T: Read + Seek> KFCDataReader<T> {
    pub fn new(reader: T, dir: &KFCDir) -> Self {
        let mut files = HashMap::new();

        for entry in &dir.entries {
            files.insert(entry.name_hash, entry.clone());
        }

        Self {
            reader: Reader::new(reader),
            files
        }
    }

    pub fn seek_file(&mut self, hash: u64) -> std::io::Result<u64> {
        if let Some(entry) = self.files.get(&hash) {
            self.reader.seek(SeekFrom::Start(entry.offset))?;
            Ok(entry.compressed_size as u64)
        } else {
            Ok(0)
        }
    }

    pub fn read_file(&mut self, hash: u64, buf: &mut [u8]) -> std::io::Result<usize> {
        let size = self.seek_file(hash)?;
        self.reader.read_exact(buf)?;
        Ok(size as usize)
    }

    pub fn file_info(&self, hash: u64) -> Option<&KFCDirEntry> {
        self.files.get(&hash)
    }
}
