use anyhow::{Ok, Result};
use clap::Parser;
use cli::Cli;
use colored::Colorize;

mod cli;

fn main() {
    let args = Cli::parse();

    if let Err(e) = run(&args) {
        eprintln!("{} {e}", "error:".red());
    }
}

fn run(args: &Cli) -> Result<()> {
    Ok(())
}
