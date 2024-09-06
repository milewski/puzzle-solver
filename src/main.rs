use std::fs;
use clap::{Parser};

use crate::puzzle::PuzzleManager;
use crate::randomizer::Randomizer;
use crate::worker::Device;

mod puzzles;
mod reporter;
mod puzzle;
mod worker;
mod randomizer;
pub mod sha256;
mod secp256k1;

/// Solver for puzzle 1 to 160
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Puzzle number from 1 to 160
    #[arg(short, long, value_parser = clap::value_parser ! (u8).range(1..=160))]
    puzzle: u8,

    #[command(subcommand)]
    mode: Device,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let puzzle = PuzzleManager::new(Randomizer {})?;
    let worker = puzzle.get_worker_for_puzzle(args.puzzle)?;

    println!("Working on puzzle #{:?} via {}.", args.puzzle, match args.mode {
        Device::CPU { .. } => "CPU",
        #[cfg(feature = "cuda")]
        Device::GPU { .. } => "GPU"
    });

    if let Some(solution) = worker.work(args.mode) {
        println!("Solution found: {:0>64}", solution.to_hex());
        fs::write("solution.txt", solution.to_hex())?;
    }

    Ok(())
}
