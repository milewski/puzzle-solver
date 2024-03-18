use std::ops::{Add, AddAssign, Deref, Sub};
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::{available_parallelism, JoinHandle, spawn};

use anyhow::bail;
use embedded_hal::blocking::delay::DelayUs;
use k256::{ProjectivePoint, SecretKey};
use k256::elliptic_curve::group::GroupEncoding;
use num_bigint::BigUint;
use num_traits::{ToBytes, ToPrimitive};
use rayon::prelude::*;

use crate::puzzles::{PuzzleJson, PuzzleRange, Puzzles};
use crate::reporter::Reporter;

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
                puzzles: Puzzles::load()?,
                reporter: Reporter::new(),
                randomizer: Arc::new(Utility::new(randomizer)),
            }
        )
    }

    pub fn start_embedded(&self, number: u8, delay: impl DelayUs<u32> + 'static, notifier: impl Fn(Event) + Send + Sync + 'static) -> anyhow::Result<()> {
        let mut delay = Platform::Embedded(Box::new(delay));

        Ok(
            self.get_worker_for_puzzle(number)?.work_embedded(Arc::new(notifier), &mut delay)
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
            self.reporter.clone(),
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

enum Platform {
    Desktop,
    Embedded(Box<dyn DelayUs<u32>>),
}

struct Worker<T: Hasher> {
    range: PuzzleRange,
    reporter: Arc<Mutex<Reporter>>,
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

    pub fn to_private_key(&self) -> [u8; 32] {
        let mut buffer = [0; 32];

        for (index, byte) in self.to_le_bytes().into_iter().enumerate() {
            buffer[32 - index - 1] = byte;
        }

        buffer
    }
}

impl<T> Worker<T> where T: Hasher + Send + Sync {
    fn from_puzzle(challenge: PuzzleJson, utility: Arc<Utility<T>>, reporter: Arc<Mutex<Reporter>>) -> anyhow::Result<Self> {
        Ok(
            Self {
                range: challenge.range()?,
                target: challenge.target()?,
                increments: BigUint::from(u16::MAX),
                reporter,
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

    fn work_embedded(&self, notifier: Arc<dyn Fn(Event) + Send + Sync + 'static>, delay: &mut Platform) {
        loop {
            if let Ok(solution) = self.compute_parallel(delay) {
                break notifier(Event::SolutionFound(Solution(solution)));
            }
        }
    }

    fn work_forever(&self, notifier: Arc<dyn Fn(Event) + Send + Sync + 'static>) {
        loop {
            if let Ok(solution) = self.compute_parallel(&mut Platform::Desktop) {
                break notifier(Event::SolutionFound(Solution(solution)));
            }
        }
    }

    fn work_with_signal(&self, notifier: Arc<dyn Fn(Event) + Send + Sync + 'static>, signal: Arc<AtomicBool>) {
        loop {
            if signal.load(Ordering::Relaxed) {
                break;
            }

            if let Ok(solution) = self.compute_parallel(&mut Platform::Desktop) {
                break notifier(Event::SolutionFound(Solution(solution)));
            }
        }
    }

    fn compute_parallel(&self, platform: &mut Platform) -> anyhow::Result<BigUint> {
        let (min, max) = self.range.random_between(&self.increments, self.utility.clone());

        let mut counter = min.clone();
        let mut point = self.get_curve_point(&counter)?;

        let mut keys = Vec::with_capacity(self.increments.to_usize().unwrap());

        while counter <= max {
            point.add_assign(ProjectivePoint::GENERATOR);
            keys.push(point.to_bytes());
            counter.add_assign(1u8);
        }

        let solution: Option<BigUint> = keys
            .into_par_iter()
            .enumerate()
            .map(|(index, point)| {

                // match platform {
                //     Platform::Desktop => {}
                //     Platform::Embedded(delay) => delay.delay_us(1)
                // }

                let sha256: [u8; 32] = self.utility.sha256(&point);
                let ripemd160: [u8; 20] = self.utility.ripemd160(&sha256);

                if self.target == ripemd160 {
                    Some(min.clone().add(index + 1))
                } else {
                    None
                }
            })
            .find_map_first(|solution| solution)
            .take();

        if let (Ok(mut logger), Some(count)) = (self.reporter.lock(), max.sub(min).to_u64()) {
            logger.update(count)
        }

        if let Some(solution) = solution {
            return Ok(solution)
        }

        bail!("Solution not found...")
    }

    fn compute(&self, platform: &mut Platform) -> anyhow::Result<BigUint> {
        let (min, max) = self.range.random_between(&self.increments, self.utility.clone());

        let mut counter = min.clone();
        let mut point = self.get_curve_point(&counter)?;
        let difference = (&max).sub(min).to_u64();

        while counter <= max {
            match platform {
                Platform::Desktop => {}
                Platform::Embedded(delay) => delay.delay_us(1)
            }

            let sha256: [u8; 32] = self.utility.sha256(&point.to_bytes());
            let ripemd160: [u8; 20] = self.utility.ripemd160(&sha256);

            if self.target == ripemd160 {
                if let (Ok(mut reporter), Some(count)) = (self.reporter.lock(), difference) {
                    reporter.update(count);
                }

                return Ok(counter);
            }

            point.add_assign(ProjectivePoint::GENERATOR);
            counter.add_assign(1u8);
        }

        if let (Ok(mut reporter), Some(count)) = (self.reporter.lock(), difference) {
            reporter.update(count);
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

