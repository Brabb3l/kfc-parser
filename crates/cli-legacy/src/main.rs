#![allow(deprecated)]

use clap::Parser;
use colored::Colorize;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use kfc::container::{KFCFile, KFCReader, KFCWriter};
use kfc::resource::value::Value;
use kfc::guid::ResourceId;
use kfc::reflection::{LookupKey, TypeRegistry};
use kfc::content::impact::bytecode::{ImpactAssembler, ImpactProgramData};
use kfc::content::impact::{ImpactProgram, TypeRegistryImpactExt};
use thiserror::Error;
use std::collections::HashSet;
use std::env::current_exe;
use std::fs::File;
use std::io::{BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::process::exit;
use std::sync::atomic::{AtomicBool, AtomicU32};
use std::sync::Mutex;
use walkdir::WalkDir;

use crate::cli::{Cli, CommandImpact, Commands};
use crate::logging::*;

mod cli;
mod logging;
mod util;

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
    let cli = Cli::parse();
    let thread_count = cli.threads;

    let result = match cli.commands {
        Commands::Unpack {
            game_directory,
            file_name,
            output,
            filter,
            stdout,
        } => {
            set_logging(!stdout);
            unpack(
                &game_directory,
                file_name.as_deref(),
                output.as_deref(),
                stdout,
                filter,
                thread_count
            )
        }
        Commands::Repack {
            game_directory,
            file_name,
            input,
            stdin,
        } => {
            repack(
                &game_directory,
                file_name.as_deref(),
                input.as_deref(),
                stdin,
                thread_count
            )
        }
        Commands::ExtractTypes {
            game_directory,
            file_name,
        } => {
            extract_types(&game_directory, file_name.as_deref())
        }
        Commands::Restore {
            game_directory,
            file_name,
        } => {
            revert_repack(
                &game_directory,
                file_name.as_deref(),
                false
            )
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
    file_name: Option<&str>,
    output_dir: Option<&Path>,
    stdout: bool,
    filter: String,
    thread_count: u8
) -> Result<(), Error> {
    if !game_dir.exists() {
        fatal!("Game directory does not exist: {}", game_dir.display());
    }

    if output_dir.map(|x| !x.exists()).unwrap_or(false) {
        fatal!("Output directory does not exist: {}", output_dir.unwrap().display());
    }

    let file_name = get_file_name(game_dir, file_name)?;
    let file_path = get_file(game_dir, Some(&file_name), "kfc")?;
    let type_registry = load_type_registry(Some(game_dir), Some(&file_name), true)?;

    let file = match KFCFile::from_path(&file_path, false) {
        Ok(dir) => dir,
        Err(e) => fatal!("Failed to read {}: {}", file_path.display(), e)
    };

    enum Filter<'a> {
        All,
        ByType(&'a str),
        ByGuid(ResourceId)
    }

    let mut filters = Vec::new();

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
            match ResourceId::parse_qualified(entry) {
                Some(guid) => filters.push(Filter::ByGuid(guid)),
                None => fatal!("`{}` is not a valid guid", entry),
            }
        }
    }

    let mut guids = HashSet::new();

    for filter in &filters {
        match filter {
            Filter::All => {
                guids = file.resources().keys().iter().collect();
                break;
            }
            Filter::ByType(type_name) => {
                let type_hash = match type_registry.get_by_name(LookupKey::Qualified(type_name)) {
                    Some(t) => t.qualified_hash,
                    None => {
                        fatal!("Type not found: {}", type_name);
                    }
                };

                for guid in file.resources().keys() {
                    if type_hash == guid.type_hash() {
                        guids.insert(guid);
                    }
                }
            }
            Filter::ByGuid(guid) => {
                if file.resources().contains_key(guid) {
                    guids.insert(guid);
                } else {
                    fatal!("GUID not found: {}", guid);
                }
            }
        }
    }

    let kfc_reader = match KFCReader::new(game_dir, &file_name) {
        Ok(reader) => reader,
        Err(e) => fatal!("Failed to open {}: {}", file_path.display(), e)
    };

    if let Some(output_dir) = output_dir {
        info!("Unpacking {} to {}", file_path.display(), output_dir.display());

        unpack_files(
            &kfc_reader,
            &type_registry,
            output_dir,
            guids,
            thread_count
        )
    } else if stdout {
        unpack_stdout(
            &kfc_reader,
            &type_registry,
            guids,
            thread_count
        )
    } else {
        fatal!("No output directory specified")
    }
}

fn unpack_files(
    kfc_reader: &KFCReader,
    type_registry: &TypeRegistry,
    output_dir: &Path,
    guids: HashSet<&ResourceId>,
    thread_count: u8
) -> Result<(), Error> {
    let pb = ProgressBar::new(guids.len() as u64);
    pb.set_style(ProgressStyle::default_bar()
        .template(&format!("{} Unpacking... [{{bar:40}}] {{pos:>7}}/{{len:7}} {{msg}}", "info:".blue().bold()))
        .unwrap()
        .progress_chars("##-"));

    let total = guids.len() as u32;
    let pending_guids = Mutex::new(guids.into_iter().collect::<Vec<_>>());
    let failed_unpacks = AtomicU32::new(0);
    let start = std::time::Instant::now();
    let names = Mutex::new(HashSet::new());

    std::thread::scope(|s| {
        let mut handles = Vec::new();

        for i in 0..thread_count {
            let failed_unpacks = &failed_unpacks;
            let pending_guids = &pending_guids;
            let output_dir = &output_dir;
            let pb = &pb;
            let names = &names;

            let handle = s.spawn(move || {
                let mut buf = Vec::with_capacity(1024);
                let mut reader = match kfc_reader.new_cursor() {
                    Ok(file) => file,
                    Err(e) => {
                        pb.suspend(|| {
                            error!("Failed to open {}: {}", kfc_reader.kfc_path().display(), e);
                            error!("Worker #{} has been suspended", i);
                        });
                        return;
                    }
                };

                loop {
                    let guid = {
                        let mut lock = pending_guids.lock().unwrap();

                        if let Some(entry) = lock.pop() {
                            entry
                        } else {
                            break;
                        }
                    };

                    let result: anyhow::Result<()> = (|| {
                        buf.clear();
                        let descriptor = match crate::util::read_descriptor_into(
                            &mut reader,
                            type_registry,
                            guid,
                            &mut buf
                        )? {
                            Some(d) => d,
                            None => {
                                pb.suspend(|| {
                                    warn!("Skipping descriptor (not found): {}", guid.to_qualified_string());
                                });
                                return Ok(());
                            }
                        };

                        let r#type = type_registry.get_by_hash(LookupKey::Qualified(guid.type_hash()))
                            .ok_or_else(|| anyhow::anyhow!("Type not found: {:0>8x}", guid.type_hash()))?;
                        let type_name: &str = &r#type.name;
                        let parent = output_dir.join(type_name);

                        if !parent.exists() {
                            std::fs::create_dir_all(&parent)?;
                        }

                        let name = guid.to_qualified_string();
                        let mut file_name = format!("{}.json", name);
                        let mut file_names = names.lock().unwrap();
                        let mut i = 0;

                        while file_names.contains(&file_name) {
                            i += 1;
                            file_name = format!("{}.{}.json", name, i);
                        }

                        file_names.insert(file_name.clone());

                        drop(file_names);

                        let path = parent.join(&file_name);
                        let file = match File::create(&path) {
                            Ok(file) => file,
                            Err(e) => {
                                pb.suspend(|| {
                                    error!("Failed to create file with name `{}`: {}", file_name, e);
                                });
                                return Ok(());
                            }
                        };
                        let writer = BufWriter::new(file);
                        serde_json::to_writer_pretty(writer, &descriptor)?;

                        Ok(())
                    })();

                    match result {
                        Ok(()) => {},
                        Err(e) => {
                            failed_unpacks.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

                            pb.suspend(|| {
                                error!("Error occurred while unpacking `{}`: {}", guid.to_qualified_string(), e);
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

fn unpack_stdout(
    kfc_reader: &KFCReader,
    type_registry: &TypeRegistry,
    guids: HashSet<&ResourceId>,
    thread_count: u8
) -> Result<(), Error> {
    let pending_guids = Mutex::new(guids.into_iter().collect::<Vec<_>>());
    let (tx, rx) = std::sync::mpsc::sync_channel(1024);

    std::thread::scope(|s| {
        let mut handles = Vec::new();

        for _ in 0..thread_count {
            let tx = tx.clone();
            let pending_guids = &pending_guids;

            let handle = s.spawn(move || {
                let mut buf = Vec::with_capacity(1024);
                let mut reader = match kfc_reader.new_cursor() {
                    Ok(file) => file,
                    Err(e) => {
                        let error = serde_json::json!({
                            "$error": "KFCReaderError",
                            "$message": format!("Failed to open {}: {}", kfc_reader.kfc_path().display(), e),
                        }).to_string();

                        tx.send(error).unwrap();

                        return;
                    }
                };

                loop {
                    let guid = {
                        let mut lock = pending_guids.lock().unwrap();

                        if let Some(entry) = lock.pop() {
                            entry
                        } else {
                            break;
                        }
                    };

                    buf.clear();
                    let descriptor = match crate::util::read_descriptor_into(
                        &mut reader,
                        type_registry,
                        guid,
                        &mut buf
                    ) {
                        Ok(Some(d)) => match serde_json::to_string(&d) {
                            Ok(data) => data,
                            Err(e) => {
                                serde_json::json!({
                                    "$guid": guid.to_qualified_string(),
                                    "$error": "SerializationError",
                                    "$message": e.to_string(),
                                }).to_string()
                            }
                        },
                        Ok(None) => serde_json::json!({
                            "$guid": guid.to_qualified_string(),
                            "$error": "NotFound",
                        }).to_string(),
                        Err(e) => serde_json::json!({
                            "$guid": guid.to_qualified_string(),
                            "$error": "ReadError",
                            "$message": e.to_string(),
                        }).to_string()
                    };
                    let result = descriptor.to_string();

                    tx.send(result).unwrap();
                }
            });

            handles.push(handle);
        }

        drop(tx);

        let writer_handle = s.spawn(move || {
            let mut writer = std::io::stdout();

            while let Ok(data) = rx.recv() {
                match writer.write_all(data.as_bytes()) {
                    Ok(()) => {},
                    Err(e) => panic!("Failed to write to stdout: {}", e),
                }

                match writer.write_all(b"\n") {
                    Ok(()) => {},
                    Err(e) => panic!("Failed to write to stdout: {}", e),
                }
            }
        });

        match writer_handle.join() {
            Ok(()) => {},
            Err(e) => {
                fatal!("Writer thread panicked: {:?}", e)
            }
        }

        for handle in handles {
            handle.join().unwrap()
        }

        Ok(())
    })
}

fn validate_backup(
    kfc_path: &Path,
    kfc_path_bak: &Path
) -> Result<bool, Error> {
    let version_bak = match KFCFile::get_version_tag(kfc_path_bak) {
        Ok(file) => file,
        Err(_) => return Ok(false),
    };

    let version = match KFCFile::get_version_tag(kfc_path) {
        Ok(file) => file,
        Err(e) => fatal!("Failed to read {}: {}", kfc_path.display(), e)
    };

    Ok(version == version_bak)
}

fn repack(
    game_dir: &Path,
    file_name: Option<&str>,
    input_dir: Option<&Path>,
    stdin: bool,
    thread_count: u8
) -> Result<(), Error> {
    if !game_dir.exists() {
        fatal!("Game directory does not exist: {}", game_dir.display());
    }

    if input_dir.map(|x| !x.exists()).unwrap_or(false) {
        fatal!("Input directory does not exist: {}", input_dir.unwrap().display());
    }

    let kfc_path = get_file(game_dir, file_name, "kfc")?;
    let kfc_path_bak = get_file_opt(game_dir, file_name, "kfc.bak")?;

    if kfc_path_bak.exists() && !validate_backup(&kfc_path, &kfc_path_bak)? {
        warn!("Backup file is not valid, deleting it...");

        if !kfc_path_bak.is_file() {
            fatal!("Backup path is not a file, please remove it manually: {}", kfc_path_bak.display())
        }

        if let Err(e) = std::fs::remove_file(&kfc_path_bak) {
            fatal!("Failed to delete backup file: {}", e);
        }
    }

    if !kfc_path_bak.exists() {
        info!("Creating backup of {}...", kfc_path.display());

        match std::fs::copy(&kfc_path, &kfc_path_bak) {
            Ok(_) => {},
            Err(e) => fatal!("Failed to create backup: {}", e)
        }
    }

    let type_registry = load_type_registry(Some(game_dir), file_name, true)?;

    let mut ref_kfc_file = match KFCFile::from_path(&kfc_path_bak, false) {
        Ok(file) => file,
        Err(e) => fatal!("Failed to read {}: {}", kfc_path_bak.display(), e)
    };

    if let Some(input_dir) = input_dir {
        info!("Repacking {} to {}", input_dir.display(), kfc_path.display());

        let files = WalkDir::new(input_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .filter(|e| e.path().extension().map(|x| x == "json").unwrap_or(false))
            .map(|e| e.path().to_path_buf())
            .collect::<Vec<_>>();

        let result = repack_files(
            game_dir,
            file_name,
            &mut ref_kfc_file,
            files,
            &type_registry,
            thread_count
        );

        match result {
            Ok(()) => Ok(()),
            Err(e) => {
                error!("{}", e);
                revert_repack(game_dir, file_name, true)
            }
        }
    } else if stdin {
        info!("Repacking from stdin to {}", kfc_path.display());

        let result = repack_stdin(
            game_dir,
            file_name,
            &mut ref_kfc_file,
            &type_registry,
            thread_count
        );

        match result {
            Ok(()) => Ok(()),
            Err(e) => {
                error!("{}", e);
                revert_repack(game_dir, file_name, true)
            }
        }
    } else {
        fatal!("No input specified");
    }
}

fn repack_files(
    game_dir: &Path,
    file_name: Option<&str>,
    ref_kfc_file: &mut KFCFile,
    files: Vec<PathBuf>,
    type_registry: &TypeRegistry,
    thread_count: u8
) -> Result<(), Error> {
    if files.is_empty() {
        fatal!("No files found to repack");
    }

    let file_name = get_file_name(game_dir, file_name)?;
    let file_path = get_file(game_dir, Some(&file_name), "kfc")?;

    let mut writer = match KFCWriter::new_incremental(
        game_dir,
        &file_name,
        ref_kfc_file,
        type_registry
    ) {
        Ok(writer) => writer,
        Err(e) => fatal!("Failed to open {}: {}", file_path.display(), e)
    };

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
                        let reader = BufReader::new(File::open(&file)?);
                        let descriptor = serde_json::from_reader::<_, Value>(reader)?;
                        let result = crate::util::serialize_descriptor(type_registry, &descriptor)?;

                        tx.send(result).unwrap();

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
                match writer.write_resource(&guid, &data) {
                    Ok(()) => {},
                    Err(e) => {
                        failed_repacks.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

                        mpb.suspend(|| {
                            error!("Error occurred while writing `{}`: {}", guid.to_qualified_string(), e);
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

    match writer.finalize() {
        Ok(()) => {},
        Err(e) => {
            return Err(Error(format!("Failed to write to {}: {}", file_path.display(), e)));
        }
    }

    let failed_repacks = failed_repacks.load(std::sync::atomic::Ordering::Relaxed);
    let end = std::time::Instant::now();

    info!("Repacked a total of {}/{} descriptors in {:?}", total - failed_repacks, total, end - start);

    Ok(())
}

fn repack_stdin(
    game_dir: &Path,
    file_name: Option<&str>,
    ref_kfc_file: &mut KFCFile,
    type_registry: &TypeRegistry,
    thread_count: u8
) -> Result<(), Error> {
    let file_name = get_file_name(game_dir, file_name)?;
    let file_path = get_file(game_dir, Some(&file_name), "kfc")?;

    let mut writer = match KFCWriter::new_incremental(
        game_dir,
        &file_name,
        ref_kfc_file,
        type_registry
    ) {
        Ok(writer) => writer,
        Err(e) => fatal!("Failed to open {}: {}", file_path.display(), e)
    };

    let (tx_in, rx_in) = crossbeam_channel::unbounded::<(usize, String)>();
    let (tx, rx) = std::sync::mpsc::sync_channel(1024);
    let failed_repacks = AtomicU32::new(0);
    let total = AtomicU32::new(0);
    let start = std::time::Instant::now();

    let result = std::thread::scope(|s| {
        let failed_repacks = &failed_repacks;
        let writer = &mut writer;
        let total = &total;

        let mut handles = Vec::new();

        for _ in 0..thread_count {
            let tx = tx.clone();
            let rx_in = rx_in.clone();

            let handle = s.spawn(move || {
                for (i, descriptor_str) in rx_in.iter() {
                    let descriptor = match serde_json::from_str::<Value>(&descriptor_str) {
                        Ok(d) => d,
                        Err(e) => {
                            failed_repacks.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                            error!("Error occurred while parsing descriptor {}: {}", i, e);
                            continue;
                        }
                    };

                    let result = match crate::util::serialize_descriptor(type_registry, &descriptor) {
                        Ok(result) => result,
                        Err(e) => {
                            failed_repacks.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                            error!("Error occurred while serializing descriptor {}: {}", i, e);
                            continue;
                        }
                    };

                    tx.send(result).unwrap();
                }
            });

            handles.push(handle);
        }

        drop(tx);
        drop(rx_in);

        let in_handle = s.spawn(move || {
            let stdin = std::io::stdin();

            for (i, line) in stdin.lines().enumerate() {
                let line = match line {
                    Ok(line) => line,
                    Err(e) => {
                        error!("Error occurred while reading from stdin: {}", e);
                        break;
                    }
                };

                if line.is_empty() {
                    continue;
                }

                let result = tx_in.send((i, line));

                total.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

                if result.is_err() {
                    break;
                }
            }
        });

        let writer_handle = s.spawn(move || {
            while let Ok((guid, data)) = rx.recv() {
                match writer.write_resource(&guid, &data) {
                    Ok(()) => {},
                    Err(e) => {
                        failed_repacks.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                        error!("Error occurred while writing `{}`: {}", guid.to_qualified_string(), e);
                    }
                }
            }
        });

        match writer_handle.join() {
            Ok(()) => {},
            Err(e) => {
                fatal!("Writer thread panicked: {:?}", e)
            }
        }

        match in_handle.join() {
            Ok(()) => {},
            Err(e) => {
                fatal!("Stdin reader thread panicked: {:?}", e)
            }
        }

        for handle in handles {
            handle.join().unwrap()
        }

        Ok(())
    });

    match result {
        Ok(()) => {},
        Err(e) => {
            return Err(e);
        }
    }

    match writer.finalize() {
        Ok(()) => {},
        Err(e) => {
            return Err(Error(format!("Failed to write to {}: {}", file_path.display(), e)));
        }
    }

    let total = total.load(std::sync::atomic::Ordering::Relaxed);
    let failed_repacks = failed_repacks.load(std::sync::atomic::Ordering::Relaxed);
    let end = std::time::Instant::now();

    info!("Repacked a total of {}/{} descriptors in {:?}", total - failed_repacks, total, end - start);

    Ok(())
}

fn revert_repack(
    game_dir: &Path,
    file_name: Option<&str>,
    because_of_error: bool,
) -> Result<(), Error> {
    let kfc_file = get_file(game_dir, file_name, "kfc")?;
    let kfc_file_bak = get_file(game_dir, file_name, "kfc.bak")?;

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

fn extract_types(
    game_dir: &Path,
    file_name: Option<&str>,
) -> Result<(), Error> {
    let executable = get_file(game_dir, file_name, "exe")?;
    let mut type_registry = match TypeRegistry::load_from_executable(executable) {
        Ok(count) => count,
        Err(e) => {
            fatal!("Failed to extract types: {}", e)
        }
    };

    info!("Extracted a total of {} types", type_registry.len());

    let file_name = get_file(game_dir, file_name, "kfc")?;
    let version_tag = match KFCFile::get_version_tag(&file_name) {
        Ok(file) => file,
        Err(e) => fatal!("Failed to read {}: {}", file_name.display(), e)
    };

    type_registry.version = version_tag;

    let path = current_exe().unwrap()
        .parent().unwrap()
        .join("reflection_data.json");

    match dump_types_to_path(&type_registry, path, false) {
        Ok(_) => {},
        Err(e) => {
            fatal!("Failed to dump reflection data: {}", e)
        }
    }

    info!("Reflection data has been written to reflection_data.json");

    Ok(())
}

fn load_type_registry(
    game_dir: Option<&Path>,
    file_name: Option<&str>,
    retry: bool
) -> Result<TypeRegistry, Error> {
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
            load_types_from_path(
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

    let type_registry = match result {
        Ok(type_registry) => {
            if let Some(game_dir) = game_dir {
                let kfc_path = get_file(game_dir, file_name, "kfc")?;
                let version_tag = match KFCFile::get_version_tag(&kfc_path) {
                    Ok(file) => file,
                    Err(e) => fatal!("Failed to read {}: {}", kfc_path.display(), e)
                };

                if type_registry.version != version_tag {
                    if retry {
                        warn!("reflection_data.json is outdated, attempting to extract types again...");
                        extract_types(game_dir, file_name)?;
                        return load_type_registry(Some(game_dir), file_name, false);
                    }

                    fatal!("reflection_data.json is outdated, please extract types again");
                }
            }

            type_registry
        },
        Err(TypeParseError::Io(e)) => {
            if let Some(game_dir) = game_dir {
                if e.kind() == std::io::ErrorKind::NotFound && retry {
                    warn!("reflection_data.json not found, attempting to extract types first...");
                    extract_types(game_dir, file_name)?;
                    return load_type_registry(Some(game_dir), file_name, false);
                } else {
                    fatal!("Failed to load reflection_data.json: {}", e);
                }
            } else if e.kind() == std::io::ErrorKind::NotFound {
                fatal!("reflection_data.json not found, please extract types first");
            } else {
                fatal!("Failed to load reflection_data.json: {}", e);
            }
        }
        Err(TypeParseError::Json(e)) => {
            if let Some(game_dir) = game_dir {
                if retry {
                    warn!("reflection_data.json is invalid, attempting to extract types again...");
                    extract_types(game_dir, file_name)?;
                    return load_type_registry(Some(game_dir), file_name, false);
                }
            }

            fatal!("Failed to load reflection_data.json: {}", e)
        }
    };

    info!("Loaded a total of {} types", type_registry.len());

    Ok(type_registry)
}

fn assemble_impact(
    input_file: &Path,
    output_file: Option<&Path>,
    guid: Option<&str>,
) -> Result<(), Error> {
    let type_registry = load_type_registry(None, None, true)?;
    let file_name = input_file.file_stem().unwrap().to_str().unwrap();

    let guid_str = guid
        .or_else(|| output_file.map(|f| f.file_stem().unwrap().to_str().unwrap()))
        .unwrap_or(file_name);

    let guid = match ResourceId::parse_qualified(guid_str) {
        Some(guid) => guid,
        None => {
            if let Some(guid) = guid {
                fatal!("`{}` is not a valid descriptor guid", guid);
            } else if output_file.is_some() {
                fatal!("Output file name must be a valid descriptor guid");
            } else {
                fatal!("Input file name must be a valid descriptor guid");
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

    let assembler = ImpactAssembler::new(&type_registry);

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
        &type_registry,
        guid.guid().into(),
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
    let type_registry = load_type_registry(None, None, true)?;
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

    let impact_data = match ImpactProgramData::from_program(&type_registry, &impact_program) {
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

    let assembler = ImpactAssembler::new(&type_registry);

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
    let type_registry = load_type_registry(None, None, true)?;
    let nodes = type_registry.get_impact_nodes()
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

fn get_file(
    game_dir: &Path,
    file_name: Option<&str>,
    extension: &str
) -> Result<PathBuf, Error> {
    let file_name = get_file_name(game_dir, file_name)?;
    let file = game_dir.join(format!("{}.{}", file_name, extension));

    if !file.exists() {
        fatal!("File not found: {}", file.display());
    }

    Ok(file)
}

fn get_file_opt(
    game_dir: &Path,
    file_name: Option<&str>,
    extension: &str
) -> Result<PathBuf, Error> {
    let file_name = get_file_name(game_dir, file_name)?;
    let file = game_dir.join(format!("{}.{}", file_name, extension));

    Ok(file)
}

fn get_file_name(
    game_dir: &Path,
    file_name: Option<&str>
) -> Result<String, Error> {
    if let Some(file_name) = file_name {
        Ok(file_name.into())
    } else if game_dir.join("enshrouded.kfc").exists() || game_dir.join("enshrouded.exe").exists() {
        Ok("enshrouded".into())
    } else if game_dir.join("enshrouded_server.kfc").exists() || game_dir.join("enshrouded_server.exe").exists() {
        Ok("enshrouded_server".into())
    } else {
        // try to find file with .kfc or .exe extension
        for file in std::fs::read_dir(game_dir).into_iter().flatten().flatten() {
            if let Some(ext) = file.path().extension() {
                if ext == "kfc" || ext == "exe" {
                    let file = file.path();
                    let file_name = file.file_stem()
                        .and_then(|s| s.to_str());

                    if let Some(file_name) = file_name {
                        return Ok(file_name.into());
                    }
                }
            }
        }

        fatal!("Unable to find the kfc/exe file in game directory, please specify it with --file_name");
    }
}

fn load_types_from_path(
    path: impl AsRef<Path>
) -> Result<TypeRegistry, TypeParseError> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let json = serde_json::from_reader::<_, TypeRegistry>(reader)?;

    Ok(json)
}

fn dump_types_to_path(
    type_registry: &TypeRegistry,
    path: impl AsRef<Path>,
    pretty: bool
) -> Result<(), TypeParseError> {
    let file = File::create(path)?;
    let writer = BufWriter::new(file);

    if pretty {
        serde_json::to_writer_pretty(writer, type_registry)?;
    } else {
        serde_json::to_writer(writer, &type_registry)?;
    }

    Ok(())
}

#[derive(Debug, Error)]
pub enum TypeParseError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}
