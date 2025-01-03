use std::{
    collections::VecDeque,
    io::{self, BufRead, Read},
};

use anyhow::{anyhow, bail, Ok, Result};
use clap::Parser;
use cli::Cli;
use colored::Colorize;
use itertools::Itertools;
use mpi::{
    traits::{Communicator, Destination, Source},
    Rank,
};

mod bits;
mod cli;
mod delta;
mod huffman;

fn main() {
    //let mut input = vec![];
    //io::stdin().read_to_end(&mut input).unwrap();
    //dbg!(huffman::compress(&input));
    //return;

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

fn process_input(input: &[Box<[u8]>]) -> Result<Option<Box<[Box<[u8]>]>>> {
    let universe = mpi::initialize().ok_or(anyhow!("MPI initialization failed"))?;
    let world = universe.world();
    if world.size() < 2 {
        bail!("Number of processes must be at least 2");
    }
    let mut output = vec![VecDeque::new(); (world.size() - 1) as usize].into_boxed_slice();

    if world.rank() == 0 {
        // Equally distribute data across processes
        for (i, chunk) in input.iter().enumerate() {
            world
                .process_at_rank((i as Rank) % (world.size() - 1) + 1)
                .send(&chunk[..]);
            eprintln!("Sent {chunk:?} to {}", (i as Rank) % (world.size() - 1) + 1);
        }
        // Send empty slice to all processes, which tells them to stop
        (1..world.size()).for_each(|rank| world.process_at_rank(rank).send::<[u8]>(&[]));

        // Wait to receive data from all processes
        for _ in 0..input.len() {
            let (chunk, status) = world.any_process().receive_vec::<u8>();
            eprintln!("Got {chunk:?} from {}", status.source_rank());
            output[(status.source_rank() - 1) as usize].push_back(chunk);
        }
    } else {
        loop {
            // Receive data and stop if an empty Vec was received
            let chunk = world.process_at_rank(0).receive_vec::<u8>().0;
            if chunk.len() == 0 {
                break;
            }
            eprintln!("Received {chunk:?} on {}", world.rank());
            // Process data and send it back to process 0
            world
                .process_at_rank(0)
                .send(process_chunk(&chunk).as_ref());
        }
    }

    eprintln!("Done on {}", world.rank());
    if world.rank() != 0 {
        return Ok(None);
    }

    // Make sure output data is in the correct order
    let mut ordered_output = Vec::with_capacity(input.len());
    'outer: loop {
        for rank in 0..output.len() {
            if let Some(chunk) = output[rank].pop_front() {
                ordered_output.push(chunk.into_boxed_slice());
            } else {
                break 'outer;
            }
        }
    }

    eprintln!("{ordered_output:#?}");
    Ok(Some(ordered_output.into_boxed_slice()))
}

fn process_chunk(input: &[u8]) -> Box<[u8]> {
    input.iter().map(|data| data + 1).collect()
}
