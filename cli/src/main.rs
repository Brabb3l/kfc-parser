use clap::Parser;
use colored::Colorize;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use parser::container::KFCFile;
use parser::data::impact::bytecode::{ImpactAssembler, ImpactProgramData};
use parser::data::impact::ImpactProgram;
use parser::guid::DescriptorGuid;
use parser::reflection::{TypeCollection, TypeParseError};
use shared::io::{ReadExt, WriterSeekExt};
use std::collections::HashMap;
use std::env::current_exe;
use std::fs::File;
use std::io::{BufReader, BufWriter, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::process::exit;
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, AtomicU32};
use std::sync::Mutex;
use walkdir::WalkDir;

use crate::cli::{Cli, CommandImpact, Commands};
use crate::logging::*;

mod cli;
mod logging;
// mod test;

macro_rules! fatal {
    ($($arg:tt)*) => {
        return Err(Error(format!($($arg)*)))
    };
}

#[derive(Debug)]
struct Error(String);

impl std::error::Error for Error {}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

fn main() {
    // test::test().unwrap();
    let cli = Cli::parse();
    let thread_count = cli.threads;

    let result = match cli.commands {
        Commands::Unpack {
            game_directory,
            output,
            filter,
        } => {
            unpack(&game_directory, &output, filter, thread_count)
        }
        Commands::Repack {
            game_directory,
            input,
            all,
        } => {
            repack(game_directory, input, all, thread_count)
        }
        Commands::ExtractTypes {
            game_directory
        } => {
            extract_types(game_directory)
        }
        Commands::Restore {
            game_directory,
        } => {
            revert_repack(&game_directory, false)
        }
        Commands::Impact(impact) => match impact {
            CommandImpact::Assemble {
                input,
                output,
                guid,
            } => {
                assemble_impact(&input, output.as_deref(), guid.as_deref())
            }
            CommandImpact::Disassemble {
                input,
                output,
            } => {
                disassemble_impact(&input, output.as_deref())
            }
            CommandImpact::ExtractNodes => {
                extract_nodes()
            }
        }
    };
    
    match result {
        Ok(()) => {},
        Err(e) => {
            error!("{}", e);
            exit(1);
        }
    }
}

fn unpack(
    game_dir: &Path,
    output_dir: &Path,
    filter: String,
    thread_count: u8
) -> Result<(), Error> {
    if !game_dir.exists() {
        fatal!("Game directory does not exist: {}", game_dir.display());
    }

    if !output_dir.exists() {
        fatal!("Output directory does not exist: {}", output_dir.display());
    }

    let kfc_file = game_dir.join("enshrouded.kfc");

    if !kfc_file.exists() {
        fatal!("enshrouded.kfc file not found: {}", kfc_file.display());
    }
    
    let type_collection = load_type_collection(Some(game_dir), true)?;

    info!("Unpacking {} to {}", kfc_file.display(), output_dir.display());

    let file = match File::open(&kfc_file) {
        Ok(file) => file,
        Err(e) => fatal!("Failed to open enshrouded.kfc: {}", e)
    };
    let mut reader = BufReader::new(&file);
    let dir = match KFCFile::read(&mut reader) {
        Ok(dir) => dir,
        Err(e) => fatal!("Failed to parse enshrouded.kfc: {}", e)
    };

    enum Filter<'a> {
        All,
        ByType(&'a str),
        ByGuid(DescriptorGuid)
    }

    let mut filters = Vec::new();
    let mut has_guids = false;

    for filter in filter.split(',') {
        let entry = filter.trim();

        if entry.is_empty() {
            continue;
        }

        if entry == "*" {
            filters.clear();
            filters.push(Filter::All);
            break;
        } else if let Some(ty) = entry.strip_prefix('t') {
            filters.push(Filter::ByType(ty));
        } else {
            has_guids = true;

            match DescriptorGuid::from_str(entry) {
                Ok(guid) => filters.push(Filter::ByGuid(guid)),
                Err(e) => {
                    fatal!("`{}` is not a valid guid: {}", entry, e);
                }
            }
        }
    }

    let mut guids = HashMap::new();
    let link_map = if has_guids {
        dir.descriptor_guids.iter().zip(dir.descriptor_links.iter()).collect::<HashMap<_, _>>()
    } else {
        HashMap::new()
    };

    for filter in &filters {
        match filter {
            Filter::All => {
                if has_guids {
                    guids = link_map;
                } else {
                    guids = dir.descriptor_guids.iter().zip(dir.descriptor_links.iter()).collect::<HashMap<_, _>>();
                }

                break;
            }
            Filter::ByType(type_name) => {
                let type_hash = match type_collection.get_type_by_qualified_name(type_name) {
                    Some(t) => t.qualified_hash,
                    None => {
                        fatal!("Type not found: {}", type_name);
                    }
                };

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
                if let Some(link) = link_map.get(guid) {
                    guids.insert(guid, link);
                } else {
                    fatal!("GUID not found: {}", guid);
                }
            }
        }
    }

    let pb = ProgressBar::new(guids.len() as u64);
    pb.set_style(ProgressStyle::default_bar()
        .template(&format!("{} Unpacking... [{{bar:40}}] {{pos:>7}}/{{len:7}} {{msg}}", "info:".blue().bold()))
        .unwrap()
        .progress_chars("##-"));

    let total = guids.len() as u32;
    let pending_guids = Mutex::new(guids.into_iter().collect::<Vec<_>>());
    let failed_unpacks = AtomicU32::new(0);
    let start = std::time::Instant::now();

    std::thread::scope(|s| {
        let mut handles = Vec::new();

        for i in 0..thread_count {
            let dir = &dir;
            let failed_unpacks = &failed_unpacks;
            let pending_guids = &pending_guids;
            let kfc_file = &kfc_file;
            let type_collection = &type_collection;
            let output_dir = &output_dir;
            let pb = &pb;

            let handle = s.spawn(move || {
                let file = match File::open(kfc_file) {
                    Ok(file) => file,
                    Err(e) => {
                        pb.suspend(|| {
                            error!("Failed to open enshrouded.kfc: {}", e);
                            error!("Worker #{} has been suspended", i);
                        });
                        return;
                    }
                };

                let mut reader = BufReader::new(&file);
                let mut data = Vec::with_capacity(1024);

                loop {
                    let (guid, link) = {
                        let mut lock = pending_guids.lock().unwrap();

                        if let Some(entry) = lock.pop() {
                            entry
                        } else {
                            break;
                        }
                    };

                    let result: anyhow::Result<()> = (|| {
                        if data.capacity() < link.size as usize {
                            data.reserve(link.size as usize - data.len());
                        }

                        data.clear();
                        reader.seek(SeekFrom::Start(dir.descriptor_locations[0].offset as u64 + link.offset as u64))?;
                        reader.read_exact_n(link.size as usize, &mut data)?;

                        let descriptor = type_collection.deserialize_by_hash(guid.type_hash, &data)?;

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

                        Ok(())
                    })();

                    match result {
                        Ok(()) => {},
                        Err(e) => {
                            failed_unpacks.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

                            pb.suspend(|| {
                                error!("Error occurred while unpacking `{}`: {}", guid, e);
                            });
                        }
                    }

                    pb.inc(1);
                }
            });

            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap()
        }
    });

    pb.finish_and_clear();

    let failed_unpacks = failed_unpacks.load(std::sync::atomic::Ordering::Relaxed);
    let end = std::time::Instant::now();

    info!("Unpacked a total of {}/{} descriptors in {:?}", total - failed_unpacks, total, end - start);
    
    Ok(())
}

fn repack(
    game_dir: PathBuf,
    input_dir: PathBuf,
    all: bool,
    thread_count: u8
) -> Result<(), Error> {
    if !game_dir.exists() {
        fatal!("Game directory does not exist: {}", game_dir.display());
    }

    if !input_dir.exists() {
        fatal!("Input directory does not exist: {}", input_dir.display());
    }

    let kfc_file = game_dir.join("enshrouded.kfc");
    let kfc_file_bak = game_dir.join("enshrouded.kfc.bak");

    if !kfc_file.exists() {
        fatal!("enshrouded.kfc file not found: {}", kfc_file.display());
    }

    // TODO: Check version tag to verify if the bak file is still valid

    if !kfc_file_bak.exists() {
        info!("Creating backup of enshrouded.kfc...");

        match std::fs::copy(&kfc_file, &kfc_file_bak) {
            Ok(_) => {},
            Err(e) => fatal!("Failed to create backup: {}", e)
        }
    }

    let type_collection = load_type_collection(Some(&game_dir), true)?;

    info!("Repacking {} to {}", input_dir.display(), kfc_file.display());

    let file = match File::open(&kfc_file_bak) {
        Ok(file) => file,
        Err(e) => fatal!("Failed to open enshrouded.kfc.bak: {}", e)
    };
    let mut reader = BufReader::new(&file);
    let mut dir = match KFCFile::read(&mut reader) {
        Ok(dir) => dir,
        Err(e) => fatal!("Failed to parse enshrouded.kfc.bak: {}", e)
    };

    let files = WalkDir::new(input_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter(|e| e.path().extension().map(|x| x == "json").unwrap_or(false))
        .filter(|e| !e.path().file_stem().and_then(|x| x.to_str()).unwrap_or("").contains('.'))
        .map(|e| e.path().to_path_buf())
        .collect::<Vec<_>>();
    
    let result = repack0(&kfc_file, &mut dir, files, &type_collection, all, thread_count);
    
    match result {
        Ok(()) => Ok(()),
        Err(e) => {
            error!("{}", e);
            revert_repack(&game_dir, true)
        }
    }
}

fn repack0(
    kfc_file: &Path,
    dir: &mut KFCFile,
    files: Vec<PathBuf>,
    type_collection: &TypeCollection,
    all: bool,
    thread_count: u8
) -> Result<(), Error> {
    if files.is_empty() {
        fatal!("No files found to repack");
    }
    
    let mut descriptor_offset: u64 = dir.descriptor_locations[0].offset as u64;
    let mut writer = if all {
        let file = match File::create(kfc_file) {
            Ok(file) => file,
            Err(e) => fatal!("Failed to create enshrouded.kfc: {}", e)
        };
        let mut writer = BufWriter::new(file);

        match (|| -> anyhow::Result<()> {
            dir.write(&mut writer)?;
            writer.align(16)?;
            descriptor_offset = writer.stream_position()?;

            Ok(())
        })() {
            Ok(()) => {},
            Err(e) => fatal!("Failed to write to enshrouded.kfc: {}", e)
        }

        writer
    } else {
        let file = match File::options().write(true).open(kfc_file) {
            Ok(file) => file,
            Err(e) => fatal!("Failed to open enshrouded.kfc: {}", e)
        };
        let mut writer = BufWriter::new(file);

        match (|| -> anyhow::Result<()> {
            writer.seek(SeekFrom::End(0))?;
            writer.align(16)?;
            
            Ok(())
        })() {
            Ok(()) => {},
            Err(e) => fatal!("Failed to write to enshrouded.kfc: {}", e)
        }

        writer
    };

    let mut guid_links = dir.descriptor_guids.iter().zip(dir.descriptor_links.iter_mut()).collect::<HashMap<_, _>>();
    let total = files.len() as u64;
    
    let mpb = MultiProgress::new();
    
    let pb_serialize = mpb.add(ProgressBar::new(total));
    pb_serialize.set_style(ProgressStyle::default_bar()
        .template(&format!("{} Serializing... [{{bar:40}}] {{pos:>7}}/{{len:7}} {{msg}}", "info:".blue().bold()))
        .unwrap()
        .progress_chars("##-"));

    let pb_write = mpb.add(ProgressBar::new(total));
    pb_write.set_style(ProgressStyle::default_bar()
        .template(&format!("{} Writing... [{{bar:40}}] {{pos:>7}}/{{len:7}} {{msg}}", "info:".blue().bold()))
        .unwrap()
        .progress_chars("##-"));
    
    let total = files.len() as u32;
    let pending_files = Mutex::new(files.into_iter().collect::<Vec<_>>());
    let (tx, rx) = std::sync::mpsc::sync_channel(1024);
    let failed_repacks = AtomicU32::new(0);
    let start = std::time::Instant::now();
    
    let result = std::thread::scope(|s| {
        let failed_repacks = &failed_repacks;
        let pending_files = &pending_files;
        let type_collection = &type_collection;
        let writer = &mut writer;
        let pb_serialize = &pb_serialize;
        let pb_write = &pb_write;
        let mpb = &mpb;

        let mut handles = Vec::new();

        for _ in 0..thread_count {
            let tx = tx.clone();
            
            let handle = s.spawn(move || {
                loop {
                    let file = {
                        let mut lock = pending_files.lock().unwrap();

                        if let Some(file) = lock.pop() {
                            file
                        } else {
                            break;
                        }
                    };

                    let result: anyhow::Result<()> = (|| {
                        let guid = match DescriptorGuid::from_str(
                            file.file_stem().unwrap().to_str().unwrap()
                        ) {
                            Ok(guid) => guid,
                            Err(_) => {
                                mpb.suspend(|| {
                                    warn!("Skipping file (invalid guid): {}", file.display());
                                });

                                return Ok(());
                            }
                        };

                        let json = std::fs::read(&file)?;
                        let descriptor: serde_json::Value = serde_json::from_slice(&json)?;

                        let mut data = Vec::with_capacity(1024);
                        type_collection.serialize_into_by_hash(guid.type_hash, &descriptor, &mut data)?;

                        tx.send((guid, data)).unwrap();

                        Ok(())
                    })();

                    match result {
                        Ok(()) => {},
                        Err(e) => {
                            failed_repacks.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

                            mpb.suspend(|| {
                                error!("Error occurred while repacking `{}`: {}", file.display(), e);
                            });
                        }
                    }
                    
                    pb_serialize.inc(1);
                }
            });

            handles.push(handle);
        }
        
        drop(tx);
        
        let writer_handle = s.spawn(move || {
            while let Ok((guid, data)) = rx.recv() {
                let link = match guid_links.get_mut(&guid) {
                    Some(link) => link,
                    None => {
                        failed_repacks.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                        
                        mpb.suspend(|| {
                            error!("No data link found for GUID: {}", guid);
                        });
                        
                        continue;
                    }
                };

                let result: std::io::Result<_> = (|| {
                    let offset = writer.stream_position()? - descriptor_offset;

                    link.offset = offset as u32;
                    link.size = data.len() as u32;

                    writer.write_all(&data)?;
                    writer.align(16)?;
                    
                    Ok(())
                })();
                
                match result {
                    Ok(()) => {},
                    Err(e) => {
                        failed_repacks.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                        
                        mpb.suspend(|| {
                            error!("Error occurred while writing `{}`: {}", guid, e);
                        });
                    }
                }
                
                pb_write.inc(1);
            }
        });
        
        match writer_handle.join() {
            Ok(()) => {},
            Err(e) => {
                {
                    let mut lock = pending_files.lock().unwrap();
                    lock.clear();
                }
                
                for handle in handles {
                    handle.join().unwrap()
                }

                fatal!("Writer thread panicked: {:?}", e)
            }
        }

        for handle in handles {
            handle.join().unwrap()
        }
        
        Ok(())
    });
    
    pb_serialize.finish_and_clear();
    pb_write.finish_and_clear();
    
    match result {
        Ok(()) => {},
        Err(e) => {
            return Err(e);
        }
    }
    
    match (|| -> anyhow::Result<()> {
        let size = writer.stream_position()? - descriptor_offset;
        dir.descriptor_locations[0].size = size as u32;
        dir.descriptor_locations[0].offset = descriptor_offset as u32;

        writer.seek(SeekFrom::Start(0))?;
        dir.write(&mut writer)?;
        
        Ok(())
    })() {
        Ok(()) => {},
        Err(e) => {
            return Err(Error(format!("Failed to write to enshrouded.kfc: {}", e)));
        }
    }
    
    let failed_repacks = failed_repacks.load(std::sync::atomic::Ordering::Relaxed);
    let end = std::time::Instant::now();
    
    info!("Repacked a total of {}/{} descriptors in {:?}", total - failed_repacks, total, end - start);
    
    Ok(())
}

fn revert_repack(
    game_dir: &Path,
    because_of_error: bool,
) -> Result<(), Error> {
    let kfc_file = game_dir.join("enshrouded.kfc");
    let kfc_file_bak = game_dir.join("enshrouded.kfc.bak");
    
    if !kfc_file_bak.exists() {
        fatal!("Backup file not found: {}", kfc_file_bak.display());
    }

    if because_of_error {
        error!("An error occurred during repack, reverting changes...");
    } else {
        info!("Reverting repack...");
    }

    match std::fs::copy(&kfc_file_bak, &kfc_file) {
        Ok(_) => {},
        Err(e) => {
            error!("Failed to revert repack: {}", e);
            
            if kfc_file_bak.exists() {
                fatal!("Please manually restore from backup file: {}", kfc_file_bak.display());
            } else {
                fatal!("Backup file is missing, please verify integrity of the game files");
            }
        }
    }
    
    info!("Reverted repack successfully");
    
    Ok(())
}

fn extract_types(game_dir: PathBuf) -> Result<(), Error> {
    let executable = game_dir.join("enshrouded.exe");

    if !executable.exists() {
        fatal!("enshrouded.exe not found: {}", executable.display());
    }

    let mut type_collection = TypeCollection::default();
    let count = match type_collection.load_from_executable(executable, true) {
        Ok(count) => count,
        Err(e) => {
            fatal!("Failed to extract types: {}", e)
        }
    };

    info!("Extracted a total of {} types", count);

    let path = current_exe().unwrap()
        .parent().unwrap()
        .join("reflection_data.json");

    match type_collection.dump_to_path(path, true) {
        Ok(_) => {},
        Err(e) => {
            fatal!("Failed to dump reflection data: {}", e)
        }
    }

    info!("Reflection data has been written to reflection_data.json");
    
    Ok(())
}

fn load_type_collection(
    game_dir: Option<&Path>,
    retry_not_found: bool
) -> Result<TypeCollection, Error> {
    let mut type_collection = TypeCollection::default();
    let flag = AtomicBool::new(false);
    let pb = ProgressBar::no_length();
    
    pb.set_style(ProgressStyle::with_template(
        &format!("{} {{spinner:.green}} {{msg}}", "info:".blue().bold())
    ).unwrap());
    pb.set_message("Loading reflection data...");

    let result = std::thread::scope(|s| {
        let progress_handle = s.spawn(|| {
            while !flag.load(std::sync::atomic::Ordering::Relaxed) {
                std::thread::sleep(std::time::Duration::from_millis(50));
                pb.tick();
            }
        });

        let loading_handle = s.spawn(|| {
            type_collection.load_from_path(
                current_exe().unwrap()
                    .parent().unwrap()
                    .join("reflection_data.json")
                    .as_path()
            )
        });

        // Wait for the loading to finish and just unwrap potential panic
        let result = loading_handle.join().unwrap();

        flag.store(true, std::sync::atomic::Ordering::Relaxed);
        progress_handle.join().unwrap();

        result
    });

    pb.finish_and_clear();

    let type_count = match result {
        Ok(n) => n,
        Err(TypeParseError::Io(e)) => {
            if let Some(game_dir) = game_dir {
                if e.kind() == std::io::ErrorKind::NotFound && retry_not_found {
                    warn!("reflection_data.json not found, attempting to extract types first...");
                    extract_types(game_dir.to_path_buf())?;
                    return load_type_collection(Some(game_dir), false);
                } else {
                    fatal!("Failed to load reflection_data.json: {}", e);
                }
            } else if e.kind() == std::io::ErrorKind::NotFound {
                fatal!("reflection_data.json not found, please extract types first");
            } else {
                fatal!("Failed to load reflection_data.json: {}", e);
            }
        }
        Err(e) => fatal!("Failed to load reflection_data.json: {}", e)
    };

    info!("Loaded a total of {} types", type_count);
    
    Ok(type_collection)
}

fn assemble_impact(
    input_file: &Path,
    output_file: Option<&Path>,
    guid: Option<&str>,
) -> Result<(), Error> {
    let type_collection = load_type_collection(None, true)?;
    let file_name = input_file.file_stem().unwrap().to_str().unwrap();

    let guid_str = guid
        .or_else(|| output_file.map(|f| f.file_stem().unwrap().to_str().unwrap()))
        .unwrap_or(file_name);

    let guid = match DescriptorGuid::from_str(guid_str) {
        Ok(guid) => guid,
        Err(e) => {
            if let Some(guid) = guid {
                fatal!("`{}` is not a valid descriptor guid: {}", guid, e);
            } else if output_file.is_some() {
                fatal!("Output file name must be a valid descriptor guid: {}", e);
            } else {
                fatal!("Input file name must be a valid descriptor guid: {}", e);
            }
        }
    };

    // read files

    let impact_file = input_file.with_file_name(format!("{}.impact", file_name));
    let shutdown_file = input_file.with_file_name(format!("{}.shutdown.impact", file_name));
    let data_file = input_file.with_file_name(format!("{}.data.json", file_name));

    let impact_content = match std::fs::read_to_string(&impact_file) {
        Ok(content) => content,
        Err(e) => fatal!("Failed to read `{}.impact`: {}", file_name, e)
    };
    let shutdown_content = match std::fs::read_to_string(&shutdown_file) {
        Ok(content) => content,
        Err(e) => fatal!("Failed to read `{}.shutdown.impact`: {}", file_name, e)
    };
    let data_content = match std::fs::read_to_string(&data_file) {
        Ok(content) => content,
        Err(e) => fatal!("Failed to read `{}.data.json`: {}", file_name, e)
    };

    // parse data

    let program_data = match serde_json::from_str::<ImpactProgramData>(&data_content) {
        Ok(data) => data,
        Err(e) => fatal!("Failed to parse `{}.data.json`: {}", file_name, e)
    };

    // assemble bytecode

    let assembler = ImpactAssembler::new(&type_collection);

    let impact_ops = match assembler.parse_text(&program_data, &impact_content) {
        Ok(ops) => ops,
        Err(e) => fatal!("Failed to parse `{}.impact`: {}", file_name, e)
    };

    let shutdown_ops = match assembler.parse_text(&program_data, &shutdown_content) {
        Ok(ops) => ops,
        Err(e) => fatal!("Failed to parse `{}.shutdown.impact`: {}", file_name, e)
    };

    // create program

    let impact_program = match program_data.into_program(
        &type_collection,
        guid.as_blob_guid(),
        ImpactAssembler::assemble(&impact_ops),
        ImpactAssembler::assemble(&shutdown_ops),
    ) {
        Ok(program) => program,
        Err(e) => fatal!("Failed to create program: {}", e)
    };

    // info

    info!("Impact program info:");
    info!(" - {} + {} ops", impact_ops.len(), shutdown_ops.len());
    info!(" - {} data entries ({} bytes)", impact_program.data_layout.len(), impact_program.data.len());

    // write to file

    let output_file = output_file.unwrap_or(input_file)
        .with_extension("json");

    let writer = match File::create(&output_file) {
        Ok(file) => BufWriter::new(file),
        Err(e) => fatal!("Failed to create output file: {}", e)
    };

    match serde_json::to_writer_pretty(writer, &impact_program) {
        Ok(()) => {},
        Err(e) => fatal!("Failed to write to output file: {}", e)
    }

    info!("Impact program has been written to {}", output_file.display());

    Ok(())
}

fn disassemble_impact(
    input_file: &Path,
    output_file: Option<&Path>,
) -> Result<(), Error> {
    let type_collection = load_type_collection(None, true)?;
    let output_file = output_file.unwrap_or(input_file);
    let file_name = output_file.file_stem().unwrap().to_str().unwrap();

    let impact_file = output_file.with_file_name(format!("{}.impact", file_name));
    let shutdown_file = output_file.with_file_name(format!("{}.shutdown.impact", file_name));
    let data_file = output_file.with_file_name(format!("{}.data.json", file_name));

    // read program

    let reader = match File::open(input_file) {
        Ok(file) => BufReader::new(file),
        Err(e) => fatal!("Failed to open input file: {}", e)
    };

    let impact_program = match serde_json::from_reader::<_, ImpactProgram>(reader) {
        Ok(program) => program,
        Err(e) => fatal!("Failed to parse input file: {}", e)
    };

    // extract and write data

    let impact_data = match ImpactProgramData::from_program(&type_collection, &impact_program) {
        Ok(data) => data,
        Err(e) => fatal!("Failed to parse data from program: {}", e)
    };

    let data_writer = match File::create(&data_file) {
        Ok(file) => BufWriter::new(file),
        Err(e) => fatal!("Failed to create `{}.data.json`: {}", file_name, e)
    };

    match serde_json::to_writer_pretty(data_writer, &impact_data) {
        Ok(()) => {},
        Err(e) => fatal!("Failed to write `{}.data.json`: {}", file_name, e)
    }

    // disassemble bytecode

    let assembler = ImpactAssembler::new(&type_collection);

    let mut impact_file_writer = match File::create(&impact_file) {
        Ok(file) => BufWriter::new(file),
        Err(e) => fatal!("Failed to create `{}.impact`: {}", file_name, e)
    };

    if let Err(e) =  assembler.write_text(
        &mut impact_file_writer,
        &impact_program,
        &ImpactAssembler::disassemble(&impact_program.code),
    ) {
        fatal!("Failed to write `{}.impact`: {}", file_name, e);
    }

    let mut shutdown_file_writer = match File::create(&shutdown_file) {
        Ok(file) => BufWriter::new(file),
        Err(e) => fatal!("Failed to create `{}.shutdown.impact`: {}", file_name, e)
    };

    if let Err(e) = assembler.write_text(
        &mut shutdown_file_writer,
        &impact_program,
        &ImpactAssembler::disassemble(&impact_program.code_shutdown),
    ) {
        fatal!("Failed to write `{}.shutdown.impact`: {}", file_name, e);
    }

    // info

    info!("Impact program has been written to:");
    info!(" - {}", impact_file.display());
    info!(" - {}", shutdown_file.display());
    info!(" - {}", data_file.display());

    Ok(())
}

fn extract_nodes() -> Result<(), Error> {
    let type_collection = load_type_collection(None, true)?;
    let nodes = type_collection.get_impact_nodes()
        .into_values()
        .collect::<Vec<_>>();
    
    let output_path = current_exe().unwrap()
        .parent().unwrap()
        .join("impact_nodes.json");
    
    let writer = match File::create(&output_path) {
        Ok(file) => BufWriter::new(file),
        Err(e) => fatal!("Failed to create `impact_nodes.json`: {}", e),
    };

    match serde_json::to_writer_pretty(writer, &nodes) {
        Ok(()) => {},
        Err(e) => fatal!("Failed to write `impact_nodes.json`: {}", e)
    }

    info!("Impact node data has been written to {}", std::path::absolute(output_path).unwrap().display());
    
    Ok(())
}
