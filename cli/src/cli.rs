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
        /// Input directory (should contain enshrouded.kfc and enshrouded._XXX.dat files)
        #[arg(short, long)]
        input: String,
        
        /// Output directory
        #[arg(short, long)]
        output: String,

        /// Comma separated filter by type (prefixed with t) or guid
        #[arg(short, long, default_value = "*")]
        filter: String,
    },

    /// Repack enshrouded files (will backup the origin .kfc to .kfc.bak)
    Repack {
        /// Input directory containing unpacked files
        #[arg(short, long)]
        input: String,
        
        /// Game directory (should contain enshrouded.kfc and enshrouded._XXX.dat files)
        #[arg(short, long)]
        game_directory: String,
    },
    
    /// Extract type information from enshrouded files
    ExtractTypes {
        /// Input directory (should contain enshrouded.exe)
        #[arg(short, long)]
        input: String,
    }
}