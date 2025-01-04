use std::{
    collections::VecDeque,
    io::{self, Read, Write},
};

use anyhow::{anyhow, bail, Ok, Result};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use clap::Parser;
use cli::{Action, Algorithm, Cli};
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
    if let Err(e) = run(&Cli::parse()) {
        eprintln!("{} {e}", "error:".red());
    }
}

fn run(args: &Cli) -> Result<()> {
    let universe = mpi::initialize().ok_or(anyhow!("MPI initialization failed"))?;
    let world = universe.world();
    if world.size() < 2 {
        bail!("Number of processes must be at least 2");
    }

    let input = match world.rank() {
        0 => Some(read_input(args)?),
        _ => None,
    };
    let mut output = vec![VecDeque::new(); (world.size() - 1) as usize];

    if world.rank() == 0 {
        // Equally distribute data across processes
        for (i, chunk) in input.as_ref().unwrap().iter().enumerate() {
            world
                .process_at_rank((i as Rank) % (world.size() - 1) + 1)
                .send(&chunk[..]);
        }
        // Send empty slice to all processes, which tells them to stop
        (1..world.size()).for_each(|rank| world.process_at_rank(rank).send::<[u8]>(&[]));

        // Wait to receive data from all processes
        for _ in 0..input.as_ref().unwrap().len() {
            let (chunk, status) = world.any_process().receive_vec::<u8>();
            output[(status.source_rank() - 1) as usize].push_back(chunk);
        }
    } else {
        loop {
            // Receive data and stop if an empty Vec was received
            let chunk = world.process_at_rank(0).receive_vec::<u8>().0;
            if chunk.is_empty() {
                break;
            }
            // Process data and send it back to process 0
            world.process_at_rank(0).send(&process_chunk(&chunk, args)?);
        }
    }

    if world.rank() != 0 {
        return Ok(());
    }

    // Make sure output data is in the correct order
    let mut ordered_output = Vec::with_capacity(input.as_ref().unwrap().len());
    'outer: loop {
        for rank in 0..output.len() {
            if let Some(chunk) = output[rank].pop_front() {
                ordered_output.push(chunk);
            } else {
                break 'outer;
            }
        }
    }

    let mut stdout = io::stdout();
    for chunk in ordered_output {
        if let Action::Compress(_) = args.action {
            stdout.write_u64::<BigEndian>(chunk.len() as u64)?;
        }
        stdout.write_all(&chunk)?;
    }
    if let Action::Compress(_) = args.action {
        stdout.write_u64::<BigEndian>(0)?;
    }
    Ok(())
}

fn read_input(args: &Cli) -> Result<Vec<Vec<u8>>> {
    let mut stdin = io::stdin().lock();

    Ok(if let Action::Compress(args) = &args.action {
        let mut input = vec![];
        stdin.read_to_end(&mut input)?;

        if let Some(size) = args.size {
            input
                .into_iter()
                .chunks(size.into())
                .into_iter()
                .map(|c| c.collect())
                .collect()
        } else {
            vec![input]
        }
    } else {
        let mut input = vec![];
        loop {
            let len = stdin.read_u64::<BigEndian>()?;
            if len == 0 {
                break;
            }
            let mut chunk = vec![0; len as usize];
            stdin.read_exact(&mut chunk)?;
            input.push(chunk);
        }
        input
    })
}

fn process_chunk(input: &[u8], args: &Cli) -> Result<Vec<u8>> {
    Ok(match (&args.action, &args.algorithm) {
        (Action::Compress(_), Algorithm::Delta) => delta::compress(input),
        (Action::Decompress, Algorithm::Delta) => delta::decompress(input),
        (Action::Compress(_), Algorithm::Huffman) => huffman::compress(input),
        (Action::Decompress, Algorithm::Huffman) => huffman::decompress(input)?,
    })
}
