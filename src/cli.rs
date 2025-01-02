use clap::{Parser, ValueEnum};
use derive_more::derive::Display;

#[derive(Debug, Parser)]
pub struct Cli {
    #[arg(long, short, help = "The compression algorithm to use", default_value_t = Algorithm::default())]
    pub algorithm: Algorithm,

    #[arg(
        long,
        short,
        help = "The amount of bytes each process will compress. If not specified, input is separated on newlines"
    )]
    pub size: Option<usize>,
}

#[derive(Debug, Display, Clone, Default, ValueEnum)]
pub enum Algorithm {
    #[default]
    #[display("delta")]
    Delta,
}
