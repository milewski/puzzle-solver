use std::sync::atomic::Ordering::Relaxed;

use bitcoin::hashes::Hash;
use rand::RngCore;

use core::puzzle::{Event, Hasher, PuzzleManager};

struct Randomizer {}

impl Hasher for Randomizer {
    fn random_bytes(&self, bits: usize) -> Vec<u8> {
        let mut buffer = vec![0u8; bits];
        rand::thread_rng().fill_bytes(&mut buffer);
        buffer
    }

    fn sha256(&self, bytes: &[u8]) -> [u8; 32] {
        bitcoin::hashes::sha256::Hash::hash(&bytes).to_byte_array()
    }

    fn ripemd160(&self, bytes: &[u8]) -> [u8; 20] {
        bitcoin::hashes::ripemd160::Hash::hash(bytes).to_byte_array()
    }
}

fn main() -> anyhow::Result<()> {
    let mut puzzle = PuzzleManager::new(Randomizer {})?;

    puzzle.start_all_cores(66, |event, signal| {
        match event {
            Event::SolutionFound(solution) => {
                println!("Yeah! {}", solution.to_hex());
                signal.store(true, Relaxed)
            }
            _ => {}
        }
    })
}
