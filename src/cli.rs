use std::num::NonZeroUsize;

use clap::{Args, Parser, Subcommand, ValueEnum};
use derive_more::derive::Display;

#[derive(Debug, Parser)]
pub struct Cli {
    #[arg(long, short, help = "The compression algorithm to use", default_value_t = Algorithm::default())]
    pub algorithm: Algorithm,

    #[command(subcommand)]
    pub action: Action,
}

#[derive(Debug, Clone, Subcommand)]
pub enum Action {
    #[command(alias = "c", about = "Compress the contents of stdin")]
    Compress(CompressArgs),

    #[command(alias = "d", about = "Decompress the contents of stdin")]
    Decompress,
}

#[derive(Debug, Clone, Args)]
pub struct CompressArgs {
    #[arg(
        long,
        short,
        help = "The amount of bytes each process will compress. If not specified, one process will compress everything"
    )]
    pub size: Option<NonZeroUsize>,
}

#[derive(Debug, Display, Clone, Default, ValueEnum)]
pub enum Algorithm {
    #[display("delta")]
    Delta,

    #[default]
    #[display("huffman")]
    Huffman,
}
