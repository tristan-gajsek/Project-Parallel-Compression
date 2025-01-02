use std::io::{self, BufRead, Read};

use anyhow::{Ok, Result};
use clap::Parser;
use cli::Cli;
use colored::Colorize;
use itertools::Itertools;

mod cli;

fn main() {
    let args = Cli::parse();

    if let Err(e) = run(&args) {
        eprintln!("{} {e}", "error:".red());
    }
}

fn run(args: &Cli) -> Result<()> {
    println!("{:#?}", read_input(args.size));
    Ok(())
}

fn read_input(size: Option<usize>) -> Result<Box<[Box<[u8]>]>> {
    let mut stdin = io::stdin().lock();

    let chunks = if let Some(size) = size {
        let mut input = vec![];
        stdin.read_to_end(&mut input)?;
        input
            .into_iter()
            .chunks(size)
            .into_iter()
            .map(|chunk| chunk.collect())
            .collect()
    } else {
        stdin
            .lines()
            .map(|line| {
                line.map(|line| line.into_bytes().into())
                    .map_err(anyhow::Error::from)
            })
            .collect::<Result<_>>()?
    };

    Ok(chunks)
}
