use std::fs::File;
use std::path::PathBuf;
use clap::Parser;
use parser::container::KFCDir;
use shared::io::Reader;
use crate::cli::Cli;
use crate::exporter::{KFCMultiExporter, TaskMode};

pub mod exporter;
mod cli;

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let types: Vec<String> = if cli.filter.is_empty() {
        Vec::new()
    } else {
        cli.filter.split(',').map(|s| s.trim().to_string()).collect()
    };

    let threads = cli.threads as u32;

    let enshrouded_dir = PathBuf::from(cli.input);
    let output_dir = PathBuf::from(&cli.output);

    if !enshrouded_dir.exists() {
        return Err(anyhow::anyhow!("Input directory not found"));
    }

    if !output_dir.exists() {
        return Err(anyhow::anyhow!("Output directory not found"));
    }

    let dir_file = enshrouded_dir.join("enshrouded.kfc_dir");
    let data_file = enshrouded_dir.join("enshrouded.kfc_data");

    if !dir_file.exists() {
        return Err(anyhow::anyhow!("enshrouded.kfc_dir file not found"));
    }

    if !data_file.exists() {
        return Err(anyhow::anyhow!("enshrouded.kfc_data not found"));
    }

    let dir_file = File::open(dir_file)?;
    let dir = KFCDir::read(&mut Reader::new(dir_file))?;

    match cli.commands {
        cli::Commands::Unpack => {
            let mut exporter = KFCMultiExporter::new(dir, data_file, threads);

            exporter.export(
                &cli.output,
                TaskMode::TRY_UNPACK,
                types
            )?;
            exporter.join();
        }
        cli::Commands::Crpf { bin, debug, kbf, parsed, content } => {
            let mut mode = TaskMode::NONE;
            if bin { mode |= TaskMode::EXPORT_BIN_DATA; }
            if debug { mode |= TaskMode::EXPORT_CRPF_DEBUG; }
            if kbf { mode |= TaskMode::EXPORT_CRPF_KBF; }
            if parsed { mode |= TaskMode::EXPORT_CRPF_PLAIN; }
            if content { mode |= TaskMode::EXPORT_CRPF; }

            let mut exporter = KFCMultiExporter::new(dir, data_file, threads);

            exporter.export(
                &cli.output,
                mode,
                types
            )?;
            exporter.join();
        }
    }

    // let mut exporter = test::KFCMultiExporter2::new(dir, data_file, 32);
    //
    // exporter.export(test::TaskMode::TRY_UNPACK)?;
    // let state = exporter.join();
    //
    // for (name, (count, min_c, max_c)) in state.type_names.iter() {
    //     println!("{}: {} {} {}", name, count, min_c, max_c);
    // }


    println!("Done");

    Ok(())
}
