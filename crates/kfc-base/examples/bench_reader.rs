//! Read-only resource benchmark. Build with `cargo build -p kfc-base --release
//! --example bench_reader`, then run the resulting executable directly.
//!
//! Usage: bench_reader GAME_DIR [--file-name NAME] [--threads N]
//!        [--order index|offset] [--limit N] [--passes N]
//!
//! `index` follows the on-disk hash index backwards, modelling the CLI's
//! shared work queue without its random HashSet iteration order. `offset`
//! assigns each worker a contiguous range in resource offset order.
//! Each pass creates fresh cursors on the same KFCReader. The operating
//! system's file cache and any cache owned by the reader survive passes.

use std::{
    env,
    error::Error,
    hint::black_box,
    io,
    path::{Path, PathBuf},
    sync::atomic::{AtomicUsize, Ordering},
    thread,
    time::Instant,
};

use kfc_base::{container::KFCReader, guid::ResourceId};

const USAGE: &str = "Usage: bench_reader GAME_DIR [--file-name NAME] [--threads N] \
                    [--order index|offset] [--limit N] [--passes N]";

#[derive(Clone, Copy)]
enum Order {
    Index,
    Offset,
}

impl Order {
    fn name(self) -> &'static str {
        match self {
            Self::Index => "index",
            Self::Offset => "offset",
        }
    }
}

struct Options {
    game_dir: PathBuf,
    file_name: Option<String>,
    threads: usize,
    order: Order,
    limit: Option<usize>,
    passes: usize,
}

#[derive(Default)]
struct Totals {
    resources: usize,
    bytes: u64,
}

fn invalid_input(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, message.into())
}

fn parse_options() -> Result<Options, io::Error> {
    let mut args = env::args_os().skip(1);
    let game_dir = args.next().ok_or_else(|| invalid_input(USAGE))?;
    if game_dir == "--help" || game_dir == "-h" {
        println!("{USAGE}");
        std::process::exit(0);
    }

    let mut options = Options {
        game_dir: PathBuf::from(game_dir),
        file_name: None,
        threads: 1,
        order: Order::Index,
        limit: None,
        passes: 1,
    };

    while let Some(flag) = args.next() {
        let flag = flag
            .into_string()
            .map_err(|_| invalid_input("Option name is not valid UTF-8"))?;
        let value = args
            .next()
            .ok_or_else(|| invalid_input(format!("Missing value for {flag}. {USAGE}")))?
            .into_string()
            .map_err(|_| invalid_input(format!("Value for {flag} is not valid UTF-8")))?;
        match flag.as_str() {
            "--file-name" | "--file_name" => options.file_name = Some(value),
            "--order" => {
                options.order = match value.as_str() {
                    "index" => Order::Index,
                    "offset" => Order::Offset,
                    _ => return Err(invalid_input("--order must be index or offset")),
                };
            }
            "--threads" | "--limit" | "--passes" => {
                let number: usize = value
                    .parse()
                    .map_err(|_| invalid_input(format!("Invalid number for {flag}: {value}")))?;
                if number == 0 {
                    return Err(invalid_input(format!("{flag} must be greater than zero")));
                }
                match flag.as_str() {
                    "--threads" => options.threads = number,
                    "--limit" => options.limit = Some(number),
                    "--passes" => options.passes = number,
                    _ => unreachable!(),
                }
            }
            _ => return Err(invalid_input(format!("Unknown option: {flag}. {USAGE}"))),
        }
    }
    Ok(options)
}

fn find_file_name(game_dir: &Path) -> Result<String, io::Error> {
    for name in ["enshrouded", "enshrouded_server"] {
        if game_dir.join(format!("{name}.kfc")).is_file() {
            return Ok(name.into());
        }
    }

    let mut names = Vec::new();
    for entry in std::fs::read_dir(game_dir)? {
        let path = entry?.path();
        if path.extension().is_some_and(|extension| extension == "kfc") && path.is_file() {
            if let Some(name) = path.file_stem().and_then(|name| name.to_str()) {
                names.push(name.to_owned());
            }
        }
    }
    names.sort_unstable();
    if names.len() == 1 {
        return Ok(names.remove(0));
    }
    Err(invalid_input(
        "Specify --file-name when the game directory has no recognized or unique .kfc file",
    ))
}

fn run_pass(
    reader: &KFCReader,
    ids: &[ResourceId],
    options: &Options,
) -> Result<Totals, io::Error> {
    let next = AtomicUsize::new(0);
    thread::scope(|scope| {
        let mut handles = Vec::with_capacity(options.threads);
        for worker in 0..options.threads {
            let next = &next;
            handles.push(scope.spawn(move || -> Result<Totals, io::Error> {
                let mut cursor = reader
                    .new_cursor()
                    .map_err(|error| io::Error::other(error.to_string()))?;
                let mut buffer = Vec::new();
                let mut totals = Totals::default();
                let mut position = ids.len() * worker / options.threads;
                let end = ids.len() * (worker + 1) / options.threads;

                loop {
                    let index = match options.order {
                        Order::Index => {
                            let claimed = next.fetch_add(1, Ordering::Relaxed);
                            if claimed >= ids.len() {
                                break;
                            }
                            ids.len() - 1 - claimed
                        }
                        Order::Offset => {
                            if position >= end {
                                break;
                            }
                            let index = position;
                            position += 1;
                            index
                        }
                    };
                    let id = &ids[index];
                    buffer.clear();
                    if !cursor.read_resource_into(id, &mut buffer)? {
                        return Err(io::Error::new(
                            io::ErrorKind::NotFound,
                            format!("Resource from the index could not be read: {id:?}"),
                        ));
                    }
                    black_box(buffer.as_slice());
                    totals.resources += 1;
                    totals.bytes += buffer.len() as u64;
                }
                Ok(totals)
            }));
        }

        let mut totals = Totals::default();
        for handle in handles {
            let result = handle
                .join()
                .map_err(|_| io::Error::other("Benchmark worker panicked"))??;
            totals.resources += result.resources;
            totals.bytes += result.bytes;
        }
        if totals.resources != ids.len() {
            return Err(io::Error::other(
                "Benchmark did not read every selected resource",
            ));
        }
        Ok(totals)
    })
}

fn main() -> Result<(), Box<dyn Error>> {
    let options = parse_options()?;
    let file_name = match options.file_name.as_deref() {
        Some(name) => name.to_owned(),
        None => find_file_name(&options.game_dir)?,
    };
    let reader = KFCReader::new(&options.game_dir, file_name)?;
    let resources = reader.file().resources();
    let mut ids = resources.keys().to_vec();
    // Apply the limit first so both orders read the same deterministic subset.
    if let Some(limit) = options.limit {
        ids.truncate(limit);
    }
    if matches!(options.order, Order::Offset) {
        ids.sort_by_key(|id| resources.get(id).expect("ID came from this index").offset);
    }
    let chunks = reader.file().resource_chunks();
    let uncompressed_bytes: u64 = chunks.iter().map(|chunk| chunk.uncompressed_size).sum();

    for pass in 1..=options.passes {
        let start = Instant::now();
        let totals = run_pass(&reader, &ids, &options)?;
        let elapsed_secs = start.elapsed().as_secs_f64();
        println!(
            "{{\"elapsed_secs\":{elapsed_secs:.9},\"resources\":{},\"bytes\":{},\"threads\":{},\"order\":\"{}\",\"pass\":{pass},\"chunk_count\":{},\"uncompressed_bytes\":{uncompressed_bytes}}}",
            totals.resources,
            totals.bytes,
            options.threads,
            options.order.name(),
            chunks.len(),
        );
    }
    Ok(())
}
