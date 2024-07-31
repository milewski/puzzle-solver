use bitcoin::hashes::Hash;
use num_bigint::BigUint;
use num_traits::FromPrimitive;
use rand::RngCore;

use crate::puzzle::PuzzleManager;
use crate::randomizer::Randomizer;

mod puzzles;
mod reporter;
mod puzzle;
mod worker;
mod randomizer;

fn main() -> anyhow::Result<()> {
    let mut puzzle = PuzzleManager::new(Randomizer {})?;
    let worker = puzzle.get_worker_for_puzzle(66)?;
    let increments = BigUint::from(1_000_000u32);

    loop {
        if let Some(solution) = worker.work(&increments) {
            break println!("{:?}", solution.to_hex());
        }
    }

    // let worker = Worker::f(
    //     *puzzle.puzzles.get_puzzle(66).unwrap(),
    //     puzzle.randomizer.clone(),
    //     puzzle.reporter.clone(),
    //
    // puzzle.start(66, |event| {
    //     match event {
    //         Event::SolutionFound(solution) => {
    //             fs::write("solution.txt", solution.to_hex()).unwrap();
    //             println!("Yeah! {}", solution.to_hex());
    //             // signal.store(true, Relaxed)
    //         }
    //         _ => {}
    //     }
    // })


    Ok(())
}
