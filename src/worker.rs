use std::ops::{Add, AddAssign, Sub};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Sender};

use anyhow::bail;
use k256::{ProjectivePoint, SecretKey};
use k256::elliptic_curve::group::GroupEncoding;
use num_bigint::BigUint;
use num_traits::{ToBytes, ToPrimitive};
use rayon::iter::{IndexedParallelIterator, IntoParallelIterator, ParallelIterator};
use rayon::spawn;

use crate::puzzle::{Event, Hasher, Solution, Utility};
use crate::puzzles::{PuzzleDescriptor, PuzzleRange};
use crate::reporter::Reporter;

enum Error {
    FailedToReportHashRate
}

pub struct Worker<T: Hasher> {
    range: PuzzleRange,
    // reporter: Arc<Mutex<Reporter>>,
    reporter: Sender<u64>,
    // receiver: Arc<Mutex<Receiver<u64>>>,
    increments: BigUint,
    target: [u8; 20],
    utility: Arc<Utility<T>>,
}

impl<T> Worker<T>
where
    T: Hasher + Send + Sync
{
    pub fn from_puzzle(challenge: &PuzzleDescriptor, utility: Arc<Utility<T>>) -> anyhow::Result<Self> {
        let (sender, receiver) = channel::<u64>();

        spawn(move || {
            let mut reporer = Reporter::clean();

            loop {
                if let Ok(value) = receiver.recv() {
                    reporer.update(value);
                }
            }
        });

        Ok(
            Self {
                range: challenge.range().unwrap(),
                target: challenge.target().unwrap(),
                increments: BigUint::from(u16::MAX),
                reporter: sender,
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

    pub fn work_forever(&self, notifier: Arc<dyn Fn(Event) + Send + Sync + 'static>) {
        loop {
            if let Ok(solution) = self.compute_parallel() {
                break notifier(Event::SolutionFound(Solution(solution)));
            }
        }
    }

    pub fn work_with_signal(&self, notifier: Arc<dyn Fn(Event) + Send + Sync + 'static>, signal: Arc<AtomicBool>) {
        loop {
            if signal.load(Ordering::Relaxed) {
                break;
            }

            if let Ok(solution) = self.compute_parallel() {
                break notifier(Event::SolutionFound(Solution(solution)));
            }
        }
    }

    pub fn work(&self, increments: &BigUint) -> Option<Solution> {
        self.compute(increments)
    }

    fn compute_parallel(&self) -> anyhow::Result<BigUint> {
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
                let sha256: [u8; 32] = self.utility.sha256(&point);
                let ripemd160: [u8; 20] = self.utility.ripemd160(&sha256);

                if self.target == ripemd160 {
                    Some((&min).add(index + 1))
                } else {
                    None
                }
            })
            .find_map_first(|solution| solution)
            .take();

        // if let (Ok(mut logger), Some(count)) = (self.reporter.lock(), max.sub(min).to_u64()) {
        //     logger.update(count)
        // }

        if let Some(solution) = solution {
            return Ok(solution)
        }

        bail!("Solution not found...")
    }

    fn compute(&self, increments: &BigUint) -> Option<Solution> {
        let (min, max) = self.range.random_between(increments, self.utility.clone());

        let mut counter = min.clone();
        let mut point = self.get_curve_point(&counter).expect("could not get curve point");
        let difference = (&max).sub(min).to_u64().unwrap();

        while counter <= max {
            let sha256: [u8; 32] = self.utility.sha256(&point.to_bytes());
            let ripemd160: [u8; 20] = self.utility.ripemd160(&sha256);

            if self.target == ripemd160 {
                return Some(Solution(counter));
            }

            point.add_assign(ProjectivePoint::GENERATOR);
            counter.add_assign(1u8);
        }

        if let Err(error) = self.reporter.send(difference) {
            println!("Failed to report hash rate: {:?}", error)
        }

        None
    }
}

#[cfg(test)]
mod test {
    use std::sync::Arc;

    use num_bigint::BigUint;

    use crate::puzzle::Utility;
    use crate::puzzles::Puzzles;
    use crate::randomizer::Randomizer;
    use crate::worker::Worker;

    #[test]
    fn test_worker() {
        let randomizer = Arc::new(Utility::new(Randomizer {}));
        let puzzle = Puzzles::new();
        let puzzle = puzzle.get(10).unwrap();

        let worker = Worker::from_puzzle(puzzle, randomizer).unwrap();
        let increments = BigUint::from(1000u32);

        loop {
            if let Some(solution) = worker.work(&increments) {
                break assert_eq!(
                    solution.to_private_key(), [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 2]
                );
            }
        }
    }
}