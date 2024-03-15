use num_bigint::BigUint;
use num_traits::One;
use rand::RngCore;

use crate::puzzle::{PuzzleManager, PuzzleRange, RandomizerTrait};

mod puzzle;

struct Randomizer {}

impl RandomizerTrait for Randomizer {
    fn random_buffer<const N: usize>(&self) -> [u8; N] {
        let mut data = [0u8; N];
        rand::thread_rng().fill_bytes(&mut data);
        data
    }

    fn sha256(&self, bytes: &[u8]) -> [u8; 32] {
        sha256::digest(bytes).as_bytes().try_into().unwrap()
    }

    fn ripemd160(&self, bytes: &[u8]) -> [u8; 20] {
        todo!()
    }
}


fn main() {
    let mut puzzle: PuzzleManager<u8, Randomizer> = PuzzleManager::new(
        15,
        PuzzleRange {
            min: BigUint::one(),
            max: BigUint::one(),
        },
        Randomizer {}
    );

    puzzle.start();

    // puzzle.start(|event| {
    //     match event {
    //         Event::SolutionFound => {
    //             println!("Yeah!");
    //         },
    //         Event::ReportHashRate => {
    //             println!("Yeah!");
    //         }
    //     }
    // })

    // let randomizer = GenericRandomizer::new(Randomizer {});
    //
    // let puzzle = puzzle::Puzzle::new(
    //     randomizer,
    //     1,
    //     String::new(),
    //     String::new(),
    //     None
    // );
}