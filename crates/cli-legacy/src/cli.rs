use std::path::PathBuf;
use clap_derive::{Parser, Subcommand};

#[derive(Parser)]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub commands: Commands,

    /// How many threads to use for exporting
    #[arg(short, long, default_value = "8")]
    pub threads: u8,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Unpack enshrouded files
    Unpack {
        /// Game directory (should contain enshrouded.kfc and enshrouded._XXX.dat files)
        #[arg(short, long)]
        game_directory: PathBuf,

        /// File name override (defaults to `enshrouded` and `enshrouded_server`)
        #[arg(long)]
        file_name: Option<String>,

        /// Output directory
        #[arg(short, long, required_unless_present = "stdout", conflicts_with = "stdout")]
        output: Option<PathBuf>,

        /// Comma separated filter by type (prefixed with t) or guid
        #[arg(short, long, default_value = "*")]
        filter: String,

        /// Write unpacked content to stdout (newline-separated)
        #[arg(short, long, required_unless_present = "output", conflicts_with = "output")]
        stdout: bool,
    },

    /// Repack enshrouded files (will backup the origin .kfc to .kfc.bak)
    Repack {
        /// Game directory (should contain enshrouded.kfc and enshrouded._XXX.dat files)
        #[arg(short, long)]
        game_directory: PathBuf,

        /// File name override (defaults to `enshrouded` and `enshrouded_server`)
        #[arg(long)]
        file_name: Option<String>,

        /// Input directory containing unpacked files
        #[arg(short, long, required_unless_present = "stdin", conflicts_with = "stdin")]
        input: Option<PathBuf>,

        /// Read unpacked content from stdin (newline-separated)
        #[arg(short, long, required_unless_present = "input", conflicts_with = "input")]
        stdin: bool,
    },

    /// Extract type information from enshrouded files
    ExtractTypes {
        /// Game directory (should contain enshrouded.exe)
        #[arg(short, long)]
        game_directory: PathBuf,

        /// File name override (defaults to `enshrouded` and `enshrouded_server`)
        #[arg(long)]
        file_name: Option<String>,
    },

    /// Restore the original enshrouded files
    Restore {
        /// Game directory (should contain enshrouded.kfc.bak)
        #[arg(short, long)]
        game_directory: PathBuf,

        /// File name override (defaults to `enshrouded` and `enshrouded_server`)
        #[arg(long)]
        file_name: Option<String>,
    },

    /// CLI for impact files
    #[command(subcommand)]
    Impact(CommandImpact),
}

#[derive(Subcommand)]
pub enum CommandImpact {
    /// Creates a descriptor file from a disassembled impact program
    Assemble {
        /// The shared name of the impact program files
        /// The files should be named as follows:
        /// - `file_name.impact`
        /// - `file_name.shutdown.impact`
        /// - `file_name.data.json`
        #[arg(short, long, verbatim_doc_comment)]
        input: PathBuf,

        /// An optional file name for the new impact program descriptor (will fallback to file_name.json)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// An optional guid to use for the new impact program descriptor
        #[arg(short, long)]
        guid: Option<String>,
    },

    /// Disassembles an impact program from a descriptor file into a more human-readable format
    Disassemble {
        /// The impact program descriptor file
        #[arg(short, long)]
        input: PathBuf,

        /// The output directory for the disassembled impact program (will fallback to input's directory)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Extracts all nodes from the reflection data
    ExtractNodes,
}
