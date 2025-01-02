use std::io::{self, BufRead, Read};

use anyhow::{anyhow, bail, Ok, Result};
use clap::Parser;
use cli::Cli;
use colored::Colorize;
use itertools::Itertools;
use mpi::{
    traits::{Communicator, Destination, Source},
    Rank,
};

mod cli;

fn main() {
    let args = Cli::parse();

    if let Err(e) = run(&args) {
        eprintln!("{} {e}", "error:".red());
    }
}

fn run(args: &Cli) -> Result<()> {
    let input = read_input(args.size)?;
    let output = process_input(&input);
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

fn process_input(input: &[Box<[u8]>]) -> Result<Box<[Box<[u8]>]>> {
    let universe = mpi::initialize().ok_or(anyhow!("MPI initialization failed"))?;
    let world = universe.world();
    if world.size() < 2 {
        bail!("Number of processes must be at least 2");
    }
    let output = Vec::with_capacity(input.len());

    if world.rank() == 0 {
        for (i, chunk) in input.iter().enumerate() {
            world
                .process_at_rank((i as Rank) % (world.size() - 1) + 1)
                .send(&chunk[..]);
            eprintln!("Sent {chunk:?} to {}", (i as Rank) % (world.size() - 1) + 1);
        }
        (1..world.size()).for_each(|rank| world.process_at_rank(rank).send::<[u8]>(&[]));
    } else {
        loop {
            let chunk = process_chunk(&world.process_at_rank(0).receive_vec::<u8>().0);
            if chunk.as_ref() == &[] {
                break;
            }
            eprintln!("Received {chunk:?} on {}", world.rank());
        }
    }

    eprintln!("Done on {}", world.rank());
    Ok(output.into_boxed_slice())
}

fn process_chunk(input: &[u8]) -> Box<[u8]> {
    input.into()
}
