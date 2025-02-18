use crate::cli::{Cli, Commands};
use clap::Parser;
use parser::container::KFCFile;
use parser::guid::DescriptorGuid;
use parser::reflection::{TypeCollection, TypeParseError};
use shared::io::{ReadExt, WriterSeekExt};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, BufWriter, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::Mutex;
use walkdir::WalkDir;

mod cli;

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let thread_count = cli.threads;

    match cli.commands {
        Commands::Unpack {
            game_directory,
            output,
            filter,
        } => {
            unpack(game_directory, output, filter, thread_count)?;
        }
        Commands::Repack {
            game_directory,
            input,
            all,
        } => {
            repack(game_directory, input, all)?;
        }
        Commands::ExtractTypes {
            game_directory
        } => {
            extract_types(game_directory)?;
        }
    }

    Ok(())
}

fn unpack(
    game_dir: PathBuf,
    output_dir: PathBuf,
    filter: String,
    thread_count: u8
) -> anyhow::Result<()> {
    let start = std::time::Instant::now();

    if !game_dir.exists() {
        return Err(anyhow::anyhow!("Game directory not found"));
    }

    if !output_dir.exists() {
        return Err(anyhow::anyhow!("Output directory not found"));
    }

    let kfc_file = game_dir.join("enshrouded.kfc");

    if !kfc_file.exists() {
        return Err(anyhow::anyhow!("enshrouded.kfc file not found"));
    }
    
    println!("Unpacking {} to {}", kfc_file.display(), output_dir.display());

    let mut type_collection = TypeCollection::default();
    
    match type_collection.load_from_path(Path::new("reflection_data.json")) {
        Ok(_) => {},
        Err(TypeParseError::Io(e)) => {
            if e.kind() == std::io::ErrorKind::NotFound {
                println!("reflection_data.json not found, attempting to extract types first...");
                extract_types(game_dir.clone())?;
                return unpack(game_dir, output_dir, filter, thread_count);
            }
            
            return Err(e.into());
        }
        Err(e) => return Err(e.into())
    }

    let file = File::open(&kfc_file)?;
    let mut reader = BufReader::new(&file);
    let dir = KFCFile::read(&mut reader)?;

    enum Filter<'a> {
        All,
        ByType(&'a str),
        ByGuid(DescriptorGuid)
    }

    let mut filters = Vec::new();

    for filter in filter.split(',') {
        let filter = filter.trim();

        if filter.is_empty() {
            continue;
        }

        if filter == "*" {
            filters.clear();
            filters.push(Filter::All);
            break;
        } else if let Some(ty) = filter.strip_prefix('t') {
            filters.push(Filter::ByType(ty));
        } else {
            match DescriptorGuid::from_str(filter) {
                Ok(guid) => filters.push(Filter::ByGuid(guid)),
                Err(e) => {
                    return Err(anyhow::anyhow!("Invalid GUID: {}", e));
                }
            }
        }
    }

    let mut guids = HashMap::new();

    for filter in &filters {
        match filter {
            Filter::All => {
                for (guid, link) in dir.descriptor_guids
                    .iter()
                    .zip(dir.descriptor_links.iter())
                {
                    guids.insert(guid, link);
                }
            }
            Filter::ByType(type_name) => {
                let type_hash = type_collection.get_type_by_qualified_name(type_name)
                    .ok_or_else(|| anyhow::anyhow!("Type not found: {}", type_name))?
                    .qualified_hash;

                for (guid, link) in dir.descriptor_guids
                    .iter()
                    .zip(dir.descriptor_links.iter())
                {
                    if type_hash == guid.type_hash {
                        guids.insert(guid, link);
                    }
                }
            }
            Filter::ByGuid(guid) => {
                let link = dir.get_descriptor_link(guid)
                    .ok_or_else(|| anyhow::anyhow!("GUID not found: {}", guid))?;

                guids.insert(guid, link);
            }
        }
    }

    let guids = Mutex::new(guids.into_iter().collect::<Vec<_>>());

    std::thread::scope(|s| {
        let mut handles = Vec::new();
        
        for _ in 0..thread_count {
            let handle = s.spawn::<_, anyhow::Result<()>>(|| {
                let file = File::open(&kfc_file)?;
                let mut reader = BufReader::new(&file);
                let mut data = Vec::with_capacity(1024);

                loop {
                    let (guid, link) = {
                        let mut lock = guids.lock().unwrap();

                        if let Some(entry) = lock.pop() {
                            entry
                        } else {
                            break;
                        }
                    };

                    if data.capacity() < link.size as usize {
                        data.reserve(link.size as usize - data.len());
                    }

                    data.clear();
                    reader.seek(SeekFrom::Start(dir.descriptor_locations[0].offset as u64 + link.offset as u64))?;
                    reader.read_exact_n(link.size as usize, &mut data)?;

                    let descriptor = type_collection.deserialize(guid.type_hash, &data)?;

                    let type_info = type_collection.get_type_by_qualified_hash(guid.type_hash)
                        .ok_or_else(|| anyhow::anyhow!("Type not found: {:x}", guid.type_hash))?;
                    let type_name: &str = &type_info.name;
                    let parent = output_dir.join(type_name);

                    if !parent.exists() {
                        std::fs::create_dir_all(&parent)?;
                    }

                    let path = parent.join(format!("{}.json", guid));
                    let file = File::create(&path)?;
                    let writer = BufWriter::new(file);
                    serde_json::to_writer_pretty(writer, &descriptor)?;
                }

                Ok(())
            });
            
            handles.push(handle);
        }
        
        for handle in handles {
            match handle.join() {
                Ok(Ok(())) => {},
                Ok(Err(e)) => return Err(e),
                Err(e) => return Err(anyhow::anyhow!("Thread panicked: {:?}", e))
            }
        }
        
        Ok(())
    })?;

    let end = std::time::Instant::now();

    println!("Unpacking took {:?}", end - start);

    Ok(())
}

fn repack(
    game_dir: PathBuf,
    input_dir: PathBuf,
    all: bool,
) -> anyhow::Result<()> {
    let start = std::time::Instant::now();

    if !game_dir.exists() {
        return Err(anyhow::anyhow!("Game directory not found"));
    }

    if !input_dir.exists() {
        return Err(anyhow::anyhow!("Input directory not found"));
    }

    let kfc_file = game_dir.join("enshrouded.kfc");
    let kfc_file_bak = game_dir.join("enshrouded.kfc.bak");

    if !kfc_file.exists() {
        return Err(anyhow::anyhow!("enshrouded.kfc file not found"));
    }

    if !kfc_file_bak.exists() {
        println!("Creating backup of enshrouded.kfc");
        std::fs::copy(&kfc_file, &kfc_file_bak)?;
    }

    if let Err(e) = repack0(&input_dir, &game_dir, &kfc_file, &kfc_file_bak, all) {
        println!("Repacking failed, restoring backup");
        std::fs::copy(&kfc_file_bak, &kfc_file)?;
        return Err(e);
    }

    let end = std::time::Instant::now();

    println!("Repacking took {:?}", end - start);

    Ok(())
}

fn repack0(
    input_dir: &Path,
    game_dir: &Path,
    dir_file: &Path,
    dir_bak_file: &Path,
    all: bool
) -> anyhow::Result<()> {
    println!("Repacking {} to {}", input_dir.display(), dir_file.display());
    
    let mut type_collection = TypeCollection::default();

    match type_collection.load_from_path(Path::new("reflection_data.json")) {
        Ok(_) => {},
        Err(TypeParseError::Io(e)) => {
            if e.kind() == std::io::ErrorKind::NotFound {
                println!("reflection_data.json not found, attempting to extract types first...");
                extract_types(game_dir.to_path_buf())?;
                return repack0(input_dir, game_dir, dir_file, dir_bak_file, all);
            }
            
            return Err(e.into());
        }
        Err(e) => return Err(e.into())
    }

    let file = File::open(dir_bak_file)?;
    let mut reader = BufReader::new(&file);
    let mut dir = KFCFile::read(&mut reader)?;

    let files = WalkDir::new(input_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter(|e| e.path().extension().map(|x| x == "json").unwrap_or(false))
        .map(|e| e.path().to_path_buf())
        .collect::<Vec<_>>();

    let mut writer = BufWriter::new(File::create(dir_file)?);
    dir.write(&mut writer)?;

    let base_offset = writer.stream_position()?;

    let mut data: Vec<u8> = Vec::with_capacity(1024);
    let mut guids = dir.descriptor_guids
        .iter()
        .zip(dir.descriptor_links.iter_mut())
        .collect::<std::collections::HashMap<_, _>>();

    for file in files {
        let offset = writer.stream_position()? - base_offset;

        let json = std::fs::read(&file)?;
        let descriptor: serde_json::Value = serde_json::from_slice(&json)?;

        let guid = DescriptorGuid::from_str(file.file_stem().unwrap().to_str().unwrap())
            .map_err(|e| anyhow::anyhow!("Invalid GUID: {}", e))?;

        data.clear();
        type_collection.serialize_into(guid.type_hash, &descriptor, &mut data)?;

        let link = guids.get_mut(&guid)
            .ok_or_else(|| anyhow::anyhow!("GUID not found: {}", guid))?;

        link.offset = offset as u32;
        link.size = data.len() as u32;

        writer.write_all(&data)?;
        writer.align(16)?;
    }

    let size = writer.stream_position()? - base_offset;
    dir.descriptor_locations[0].size = size as u32;
    dir.descriptor_locations[0].offset = base_offset as u32;

    writer.seek(SeekFrom::Start(0))?;
    dir.write(&mut writer)?;

    Ok(())
}

fn extract_types(game_dir: PathBuf) -> anyhow::Result<()> {
    if !game_dir.exists() {
        return Err(anyhow::anyhow!("Input directory not found"));
    }

    let executable = game_dir.join("enshrouded.exe");

    if !executable.exists() {
        return Err(anyhow::anyhow!("enshrouded.exe not found"));
    }

    let mut type_collection = TypeCollection::default();
    let count = type_collection.load_from_executable(executable)?;
    
    println!("Extracted {} types", count);
    
    type_collection.dump_to_path(Path::new("reflection_data.json"))?;

    Ok(())
}

