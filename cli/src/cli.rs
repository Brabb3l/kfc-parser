use clap_derive::{Parser, Subcommand};

#[derive(Parser)]
#[command(version)]
pub(crate) struct Cli {
    #[command(subcommand)]
    pub commands: Commands,

    /// Input directory (should contain enshrouded.kfc_dir and enshrouded.kfc_data)
    #[arg(short, long)]
    pub input: String,

    /// Output directory
    #[arg(short, long)]
    pub output: String,

    /// Filter files by type (separated by commas)
    #[arg(short, long, default_value = "")]
    pub filter: String,

    /// How many threads to use for exporting
    #[arg(short, long, default_value = "32")]
    pub threads: u8,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Unpack enshrouded files
    Unpack,

    /// Export CRPF files
    Crpf {
        /// Export raw CRPF files
        #[arg(short, long, default_value = "false")]
        bin: bool,

        /// Export CRPF debug files
        #[arg(short, long, default_value = "false")]
        debug: bool,

        /// Export CRPF-KBF files
        #[arg(short, long, default_value = "false")]
        kbf: bool,

        /// Export parsed CRPF files
        #[arg(short, long, default_value = "false")]
        parsed: bool,

        /// Export linked content files
        #[arg(short, long, default_value = "false")]
        content: bool,
    }
}