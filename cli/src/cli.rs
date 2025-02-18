use std::path::PathBuf;
use clap_derive::{Parser, Subcommand};

#[derive(Parser)]
#[command(version)]
pub(crate) struct Cli {
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
        
        /// Output directory
        #[arg(short, long)]
        output: PathBuf,

        /// Comma separated filter by type (prefixed with t) or guid
        #[arg(short, long, default_value = "*")]
        filter: String,
    },

    /// Repack enshrouded files (will backup the origin .kfc to .kfc.bak)
    Repack {
        /// Game directory (should contain enshrouded.kfc and enshrouded._XXX.dat files)
        #[arg(short, long)]
        game_directory: PathBuf,
        
        /// Input directory containing unpacked files
        #[arg(short, long)]
        input: PathBuf,
        
        /// Repack the whole file from scratch instead of extending the existing one
        #[arg(long)]
        all: bool,
    },
    
    /// Extract type information from enshrouded files
    ExtractTypes {
        /// Game directory (should contain enshrouded.exe)
        #[arg(short, long)]
        game_directory: PathBuf,
    },
    
    /// Restore the original enshrouded files
    Restore {
        /// Game directory (should contain enshrouded.kfc.bak)
        #[arg(short, long)]
        game_directory: PathBuf,
    },
}