use std::ops::{Add, AddAssign};
use std::sync::Arc;
use std::thread::sleep;
use std::time::Duration;
use bitcoin::hashes::Hash;
use cudarc::driver::{CudaSlice, DeviceRepr, LaunchAsync, LaunchConfig};
use k256::elliptic_curve::group::{Curve, GroupEncoding};
use k256::elliptic_curve::point::AffineCoordinates;
use k256::{ProjectivePoint, SecretKey};
use k256::elliptic_curve::sec1::ToEncodedPoint;
use num_bigint::BigUint;
use num_traits::{FromPrimitive, ToBytes, Zero};
use rand::RngCore;

use crate::puzzle::{PuzzleManager, Utility};
use crate::puzzles::Puzzles;
use crate::randomizer::Randomizer;

mod puzzles;
mod reporter;
mod puzzle;
mod worker;
mod randomizer;

fn sample(key: &BigUint) -> Vec<u8> {
    let mut buffer = [0; 32];

    for (index, byte) in key.to_le_bytes().into_iter().enumerate() {
        buffer[32 - index - 1] = byte;
    }

    let mut point = SecretKey::from_slice(&buffer).unwrap()
        .public_key()
        .to_projective();

    let encoded = point.to_encoded_point(false);

    let mut points = vec![];

    for x in encoded.x().unwrap().0 {
        points.push(x)
    }

    for y in encoded.y().unwrap().0 {
        points.push(y)
    }

    // println!("{:02x?}", encoded.x().unwrap().0);
    // println!("{:02x?}", encoded.y().unwrap().0);

    return points;
}

#[repr(C)]
#[derive(Debug)]
struct Solution {
    thread: u32,
    index: u32,
}

impl Solution {
    fn is_found(&self) -> bool {
        self.thread != 0 || self.index != 0
    }

    fn print(&self, keys: &Vec<BigUint>) {
        let key = keys.get(self.thread as usize).unwrap();

        println!("Solution: {:?}", key.add(self.index + 1).to_str_radix(16));
        println!("{:?}", self);
    }
}

unsafe impl DeviceRepr for Solution {}

fn main() -> anyhow::Result<()> {
    // let mut puzzle = PuzzleManager::new(Randomizer {})?;
    // let worker = puzzle.get_worker_for_puzzle(66)?;
    // let increments = BigUint::from(1_000_000u32);
    //
    // loop {
    //     if let Some(solution) = worker.work(&increments) {
    //         break println!("{:?}", solution.to_hex());
    //     }
    // }
    let threads = 1;
    let blocks = 1;
    let randomizer = Arc::new(Utility::new(Randomizer {}));
    let puzzle = Puzzles::new();
    let puzzle = puzzle.get(15).unwrap();
    let range = puzzle.range().unwrap();
    let target = puzzle.target().unwrap();

    'outer: loop {
        let mut batches = vec![];
        let increments = BigUint::from(0x10000u32);

        for thread in 0..(threads * blocks) {
            let (key, _) = range.random_between(&increments, randomizer.clone());
            batches.push(key);
            // batches.push(BigUint::from(thread))
        }

        let mut flatten = vec![];

        for batch in &batches {
            for x_y in sample(batch) {
                flatten.push(x_y)
            }
        }

        // println!("TARGET: {:02x?}", target);
        // let key = BigUint::from(0x68f0 as u32);

        let dev = cudarc::driver::CudaDevice::new(0)?;

        dev.load_ptx(desktop::SECP256K1.into(), "sha256", &["demo"])?;

        // println!("{:02x?}", points);

        let points: CudaSlice<_> = dev.htod_sync_copy(&flatten)?;
        // let x: CudaSlice<_> = dev.htod_sync_copy(&x)?;
        // let y: CudaSlice<_> = dev.htod_sync_copy(&y)?;
        let target: CudaSlice<_> = dev.htod_sync_copy(&target)?;
        let mut output: CudaSlice<_> = dev.alloc_zeros::<u8>(32)?;

        let solution = Solution { index: 0, thread: 0 };

        let mut counter: CudaSlice<_> = dev.htod_sync_copy::<Solution>(&[solution])?;

        let hash_function = dev.get_func("sha256", "demo").unwrap();

        // let cfg: LaunchConfig = LaunchConfig {
        //     grid_dim: (blocks, 1, 1),
        //     block_dim: (threads, 1, 1),
        //     shared_mem_bytes: 128,
        // };

        let cfg: LaunchConfig = LaunchConfig {
            grid_dim: (blocks, 1, 1),
            block_dim: (threads, 1, 1),
            shared_mem_bytes: 0,
        };


        unsafe { hash_function.launch(cfg, (&points, &target, &mut counter, &mut output,)) }?;

        let result = dev.dtoh_sync_copy(&counter)?;
        let output = dev.dtoh_sync_copy(&output)?;

        println!("{:02x?}", output);

        for solution in &result {
            if solution.is_found() {
                break 'outer solution.print(&batches);
            }
        }

        // let result = BigUint::from_slice(out_host.as_slice());
        //
        // if result.is_zero() == false {
        //     break println!("Answer: {:?}", (&key).add(result).to_str_radix(16));
        // }

        // break;
    }

    Ok(())
}
