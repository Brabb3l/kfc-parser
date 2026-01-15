use std::path::PathBuf;
use clap_derive::{Parser, Subcommand};

#[derive(Parser)]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub commands: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Create a new mod (dialog)
    Create {
        /// Game directory (should contain enshrouded.kfc and enshrouded._XXX.dat files)
        #[arg(short, long)]
        game_directory: PathBuf,

        /// File name override (defaults to `enshrouded` and `enshrouded_server`)
        #[arg(long)]
        file_name: Option<String>,
    },

    /// Run the mod loader with the given flags
    Run {
        /// Allow mods to patch the game files (offline)
        #[arg(short, long)]
        patch: bool,

        /// Allow mods to export files (offline)
        #[arg(short, long)]
        export: bool,

        /// Launch the game after setting up the mod environment (online)
        #[arg(short, long)]
        runtime: bool,

        /// Game directory (should contain enshrouded.kfc and enshrouded._XXX.dat files)
        #[arg(short, long)]
        game_directory: PathBuf,

        /// Export directory for mods to export files to
        #[arg(long, requires_if("export", "true"))]
        export_directory: Option<PathBuf>,

        /// File name override (defaults to `enshrouded` and `enshrouded_server`)
        #[arg(long)]
        file_name: Option<String>,

        /// Force patching even if already patched
        #[arg(short, long, default_value_t = false)]
        force: bool,
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
}
