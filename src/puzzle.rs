use std::ops::Deref;
use std::sync::{Arc, Mutex};

use num_bigint::BigUint;
use num_traits::ToBytes;
use crate::puzzles::Puzzles;
use crate::reporter::Reporter;
use crate::worker::Worker;

pub enum Event {
    SolutionFound(Solution),
    SolutionNotFound,
}

pub struct PuzzleManager<T: Hasher + Sync + Send> {
    puzzles: Puzzles,
    reporter: Arc<Mutex<Reporter>>,
    randomizer: Arc<Utility<T>>,
}

impl<T: Hasher + Send + Sync + 'static> PuzzleManager<T> {
    pub fn new(randomizer: T) -> anyhow::Result<Self> {
        Ok(
            Self {
                puzzles: Puzzles::new(),
                reporter: Reporter::new(),
                randomizer: Arc::new(Utility::new(randomizer)),
            }
        )
    }

    pub fn get_worker_for_puzzle(&self, number: u8) -> anyhow::Result<Worker<T>> {
        Worker::from_puzzle(
            self.puzzles.get(number).unwrap(),
            self.randomizer.clone(),
            // self.reporter.clone(),
        )
    }
}

#[derive(Debug)]
pub struct Solution(pub BigUint);

impl Deref for Solution {
    type Target = BigUint;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Solution {
    pub fn to_hex(&self) -> String {
        self.to_str_radix(16)
    }

    pub fn to_private_key(&self) -> [u8; 32] {
        let mut buffer = [0; 32];

        for (index, byte) in self.to_le_bytes().into_iter().enumerate() {
            buffer[32 - index - 1] = byte;
        }

        buffer
    }
}

#[derive(Debug)]
pub struct Utility<T> {
    internal: T,
}

impl<T> Deref for Utility<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.internal
    }
}

impl<T> Utility<T> {
    pub fn new(internal: T) -> Self {
        Self { internal }
    }
}

pub trait Hasher {
    fn random_bytes(&self, bytes: usize) -> Vec<u8>;
    fn sha256(&self, bytes: &[u8]) -> [u8; 32];
    fn ripemd160(&self, bytes: &[u8]) -> [u8; 20];
}

