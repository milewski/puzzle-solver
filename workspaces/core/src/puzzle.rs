use std::ops::{AddAssign, Deref};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::{available_parallelism, JoinHandle, spawn};

use anyhow::bail;
use k256::{ProjectivePoint, SecretKey};
use k256::elliptic_curve::group::GroupEncoding;
use num_bigint::BigUint;
use num_traits::ToBytes;

use crate::puzzles::{PuzzleJson, PuzzleRange, Puzzles};

pub enum Event {
    SolutionFound(Solution),
    SolutionNotFound,
}

pub struct PuzzleManager<T: Hasher + Sync + Send> {
    puzzles: Puzzles,
    randomizer: Arc<Utility<T>>,
}

impl<T: Hasher + Send + Sync + 'static> PuzzleManager<T> {
    pub fn new(randomizer: T) -> anyhow::Result<Self> {
        Ok(
            Self {
                puzzles: Puzzles::load()?,
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
            self.puzzles.get_puzzle(number),
            self.randomizer.clone(),
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

struct Worker<T: Hasher> {
    range: PuzzleRange,
    increments: BigUint,
    target: [u8; 20],
    utility: Arc<Utility<T>>,
}

pub struct Solution(BigUint);

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
}

impl<T> Worker<T> where T: Hasher {
    fn from_puzzle(challenge: PuzzleJson, utility: Arc<Utility<T>>) -> anyhow::Result<Self> {
        Ok(
            Self {
                range: challenge.range()?,
                target: challenge.target()?,
                increments: BigUint::from(u16::MAX),
                utility,
            }
        )
    }

    fn get_curve_point(&self, key: &BigUint) -> anyhow::Result<ProjectivePoint> {
        let mut buffer = [0; 32];

        for (index, byte) in key.to_le_bytes().into_iter().enumerate() {
            buffer[32 - index - 1] = byte;
        }

        let point = SecretKey::from_slice(&buffer)?
            .public_key()
            .to_projective();

        Ok(point)
    }

    fn work_forever(&self, notifier: Arc<dyn Fn(Event) + Send + Sync + 'static>) {
        loop {
            if let Ok(solution) = self.compute() {
                break notifier(Event::SolutionFound(Solution(solution)));
            }
        }
    }

    fn work_with_signal(&self, notifier: Arc<dyn Fn(Event) + Send + Sync + 'static>, signal: Arc<AtomicBool>) {
        loop {
            if signal.load(Ordering::Relaxed) {
                break;
            }

            if let Ok(solution) = self.compute() {
                break notifier(Event::SolutionFound(Solution(solution)));
            }
        }
    }

    fn compute(&self) -> anyhow::Result<BigUint> {
        let (min, max) = self.range.random_between(&self.increments, self.utility.clone());

        let mut counter = min;
        let mut point = self.get_curve_point(&counter)?;

        while counter <= max {
            let sha256: [u8; 32] = self.utility.sha256(&point.to_bytes());
            let ripemd160: [u8; 20] = self.utility.ripemd160(&sha256);

            if self.target == ripemd160 {
                return Ok(counter);
            }

            point.add_assign(ProjectivePoint::GENERATOR);
            counter.add_assign(1u8);
        }

        bail!("Solution not found...")
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

