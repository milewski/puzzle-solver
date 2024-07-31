use std::ops::Deref;
use std::sync::{Arc, Mutex};
use std::sync::atomic::AtomicBool;
use std::thread::{available_parallelism, JoinHandle, spawn};

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

    pub fn start(&self, number: u8, notifier: impl Fn(Event) + Send + Sync + 'static) -> anyhow::Result<()> {
        Ok(
            self.get_worker_for_puzzle(number)?.work_forever(Arc::new(notifier))
        )
    }

    pub fn get_worker_for_puzzle(&self, number: u8) -> anyhow::Result<Worker<T>> {
        Worker::from_puzzle(
            self.puzzles.get(number).unwrap(),
            self.randomizer.clone(),
            // self.reporter.clone(),
        )
    }

    pub fn start_all_cores(&self, number: u8, notifier: impl Fn(Event, Arc<AtomicBool>) + Send + Sync + 'static) -> anyhow::Result<()> {
        let signal = Arc::new(AtomicBool::new(false));
        let signal_notifier = signal.clone();

        let notifier = Arc::new(move |event| notifier(event, signal_notifier.clone()));

        if let Ok(cores) = available_parallelism() {
            let handles: Vec<JoinHandle<()>> = (0..=cores.get())
                .map(|_| {
                    let mut worker = self.get_worker_for_puzzle(number).unwrap();
                    let notifier = notifier.clone();
                    let signal = signal.clone();

                    spawn(move || { worker.work_with_signal(notifier, signal); })
                })
                .collect();

            for handle in handles {
                handle.join().expect("Failed to join thread");
            }
        }

        Ok(())
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

