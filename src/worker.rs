use std::ops::{Add, AddAssign, Sub};
use std::sync::Arc;
use std::sync::mpsc::{channel, Sender};
use std::thread;
use std::thread::{available_parallelism, JoinHandle, spawn};

use clap::Subcommand;
#[cfg(feature = "cuda")]
use cudarc::driver::{CudaSlice, DeviceRepr, LaunchAsync, LaunchConfig};
use k256::{ProjectivePoint, SecretKey};
use k256::elliptic_curve::group::GroupEncoding;
use k256::elliptic_curve::sec1::ToEncodedPoint;
use num_bigint::BigUint;
use num_traits::{ToBytes, ToPrimitive};

use crate::puzzle::{Hasher, Solution, Utility};
use crate::puzzles::{PuzzleDescriptor, PuzzleRange};
use crate::reporter::Reporter;

#[cfg(feature = "cuda")]
pub const SECP256K1: &str = include_str!(concat!(env!("OUT_DIR"), "/secp256k1.ptx"));

#[cfg(feature = "cuda")]
#[repr(C)]
#[derive(Debug, Default)]
struct ThreadSolution {
    thread: u32,
    index: u32,
}

#[cfg(feature = "cuda")]
impl ThreadSolution {
    fn is_found(&self) -> bool {
        self.thread != 0 || self.index != 0
    }

    fn to_solution(&self, keys: &Vec<BigUint>) -> Solution {
        let key = keys.get(self.thread as usize).unwrap();

        Solution(key.add(self.index + 1))
    }
}

#[cfg(feature = "cuda")]
unsafe impl DeviceRepr for ThreadSolution {}

#[derive(Debug, Clone, Subcommand)]
pub enum Device {
    CPU {
        #[arg(short, long, default_value_t = 0)]
        threads: u8,

        #[arg(short, long, default_value_t = 100000)]
        increments: u32,
    },

    #[cfg(feature = "cuda")]
    GPU {
        #[arg(short, long, default_value_t = 256)]
        threads: u32,
        #[arg(short, long, default_value_t = 256)]
        blocks: u32,
        #[arg(short, long, default_value_t = 50000)]
        increments: u32,
    },
}

enum Error {
    FailedToReportHashRate
}

pub struct Worker<T: Hasher + 'static> {
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
    T: Hasher + Send + Sync,
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

    pub fn work(&self, device: Device) -> Option<Solution> {
        match device {
            Device::CPU { threads, increments } => self.compute_parallel(threads, increments),
            #[cfg(feature = "cuda")]
            Device::GPU { blocks, threads, increments } => self.compute_gpu(blocks, threads, increments)
        }
    }

    fn compute_parallel(&self, threads: u8, increments: u32) -> Option<Solution> {
        let increments = BigUint::from(increments);
        let mut handlers = vec![];

        loop {
            let threads = match threads {
                0 => available_parallelism().unwrap().get(),
                _ => threads as usize
            };

            for _ in 0..=threads {
                let (min, max) = self.range.random_between(&increments, self.utility.clone());
                let utility = self.utility.clone();
                let target = self.target;
                let reporter = self.reporter.clone();
                let mut counter = min.clone();
                let mut point = self.get_curve_point(&counter).expect("could not get curve point");
                let difference = (&max).sub(min).to_u64().unwrap();

                let handle: JoinHandle<Option<Solution>> = thread::spawn(move || {
                    while counter <= max {
                        let sha256: [u8; 32] = utility.sha256(&point.to_bytes());
                        let ripemd160: [u8; 20] = utility.ripemd160(&sha256);

                        if target == ripemd160 {
                            return Some(Solution(counter));
                        }

                        point.add_assign(ProjectivePoint::GENERATOR);
                        counter.add_assign(1u8);
                    }

                    if let Err(error) = reporter.send(difference) {
                        println!("Failed to report hash rate: {:?}", error)
                    }

                    None
                });

                handlers.push(handle);
            }

            for handler in handlers.drain(..) {
                if let Ok(Some(solution)) = handler.join() {
                    return Some(solution);
                }
            }
        }
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

    #[cfg(feature = "cuda")]
    fn compute_gpu(&self, blocks: u32, threads: u32, increments: u32) -> Option<Solution> {
        let device = cudarc::driver::CudaDevice::new(0).unwrap();

        device.load_ptx(SECP256K1.into(), "secp256k1", &["start"]).unwrap();

        let target_param: CudaSlice<_> = device.htod_sync_copy(&self.target).unwrap();

        let cfg: LaunchConfig = LaunchConfig {
            grid_dim: (blocks, 1, 1),
            block_dim: (threads, 1, 1),
            shared_mem_bytes: 0,
        };

        let hashes = ((threads * blocks) * increments) as u64;
        let increments = BigUint::from(increments);

        loop {
            let mut batches = vec![];

            for _ in 0..(threads * blocks) {
                let (key, _) = self.range.random_between(&increments, self.utility.clone());

                batches.push(key);
            }

            let mut flatten: Vec<u8> = Vec::with_capacity((threads * blocks * 2 * 32) as usize);

            for key in &batches {
                if let Ok(point) = self.get_curve_point(key) {
                    let encoded = point.to_encoded_point(false);

                    if let (Some(x), Some(y)) = (encoded.x(), encoded.y()) {
                        flatten.extend(x.0);
                        flatten.extend(y.0);
                    }
                }
            }

            let increments_u32 = increments.to_u32().unwrap();
            let points_param: CudaSlice<_> = device.htod_sync_copy(&flatten).unwrap();
            let increment_param: CudaSlice<u32> = device.htod_sync_copy(&[increments_u32]).unwrap();
            let mut solution_param: CudaSlice<_> = device.htod_sync_copy::<ThreadSolution>(&[ThreadSolution::default()]).unwrap();

            let start_function = device.get_func("secp256k1", "start").unwrap();

            unsafe { start_function.launch(cfg, (&points_param, &target_param, &mut solution_param, &increment_param,)) }.unwrap();

            for solution in &device.dtoh_sync_copy(&solution_param).unwrap() {
                if solution.is_found() {
                    return Some(solution.to_solution(&batches));
                }
            }

            if let Err(error) = self.reporter.send(hashes) {
                println!("Failed to report hash rate: {:?}", error)
            }
        }
    }
}

#[cfg(test)]
mod test {
    use std::sync::Arc;

    use crate::puzzle::Utility;
    use crate::puzzles::Puzzles;
    use crate::randomizer::Randomizer;
    use crate::worker::{Device, Worker};

    #[test]
    fn test_worker() {
        let randomizer = Arc::new(Utility::new(Randomizer {}));
        let puzzle = Puzzles::new();
        let puzzle = puzzle.get(10).unwrap();

        let worker = Worker::from_puzzle(puzzle, randomizer).unwrap();

        loop {
            if let Some(solution) = worker.work(Device::CPU { threads: 1, increments: 100 }) {
                break assert_eq!(
                    solution.to_private_key(), [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 2]
                );
            }
        }
    }
}