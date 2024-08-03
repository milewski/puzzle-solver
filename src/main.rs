use std::ops::{Add, AddAssign};
use std::sync::Arc;
use std::thread::sleep;
use std::time::Duration;
use bitcoin::hashes::Hash;
use cudarc::driver::{CudaSlice, LaunchAsync, LaunchConfig};
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

fn sample(key: BigUint, count: u32) -> Vec<u8> {
    let mut buffer = [0; 32];

    for (index, byte) in key.to_le_bytes().into_iter().enumerate() {
        buffer[32 - index - 1] = byte;
    }

    let mut point = SecretKey::from_slice(&buffer).unwrap()
        .public_key()
        .to_projective();

    let mut points: Vec<u8> = vec![];

    for _ in 0..count {
        let encoded = point.to_encoded_point(false);

        for x in encoded.x().unwrap().0 {
            points.push(x);
        }

        for y in encoded.y().unwrap().0 {
            points.push(y);
        }

        // println!("{:02x?}", encoded.x().unwrap().0);
        // println!("{:02x?}", encoded.y().unwrap().0);

        // let encoded = point.to_encoded_point(true);
        //
        // let hashed = bitcoin::hashes::sha256::Hash::hash(&encoded.to_bytes()).to_byte_array();
        // let rimped = bitcoin::hashes::ripemd160::Hash::hash(&hashed).to_byte_array();
        //
        // println!("RESULT: {:02x?}", rimped);

        point.add_assign(ProjectivePoint::GENERATOR);
    }

    // println!("{:02x?}", points);
    // println!("{:02x?}", points[32..=64]);

    // println!("{:02x?}", points);

    points

    // loop {
    //
    //     let hashed = bitcoin::hashes::sha256::Hash::hash(&point2.to_bytes()).to_byte_array();
    //     let rimped = bitcoin::hashes::ripemd160::Hash::hash(&hashed).to_byte_array();
    //
    //     if rimped == [0x59, 0x6d, 0xeb, 0x19, 0x75, 0x9d, 0x0c, 0xe1, 0x07, 0x85, 0xde, 0x04, 0x97, 0x4f, 0x5d, 0xc7, 0x89, 0x48, 0x9e, 0x27]{
    //         println!("{:02x?}", rimped);
    //     }
    //
    //     point2.add_assign(ProjectivePoint::GENERATOR);
    //
    // }
    // point2.add_assign(ProjectivePoint::GENERATOR);
    //point2.add_assign(ProjectivePoint::GENERATOR);

    // println!("{:02x?}", point.to_affine().x().0);
    // println!("{:02x?}", point.to_bytes().0);

    // let hashed = bitcoin::hashes::sha256::Hash::hash(&point2.to_bytes()).to_byte_array();
    // let rimped = bitcoin::hashes::ripemd160::Hash::hash(&hashed).to_byte_array();
    //
    // // println!("{:02x?}", hashed);
    // println!("{:02x?}", rimped);
    //
    // let point = point.to_affine().to_encoded_point(false);
    // // let point = ProjectivePoint::GENERATOR.to_encoded_point(false);
    //
    // // println!("x: {:02x?}", point.to_affine().x().0);
    // return (
    //     point.x().unwrap().0,
    //     point.y().unwrap().0,
    // );
    // println!("y: {:02x?}", point.to_affine().y().0);
}

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
    let threads = 1024;
    let blocks = 1024;
    let randomizer = Arc::new(Utility::new(Randomizer {}));
    let puzzle = Puzzles::new();
    let puzzle = puzzle.get(20).unwrap();
    let range = puzzle.range().unwrap();
    let target = puzzle.target().unwrap();

    loop {
        let increments = BigUint::from(0x10000u32);
        let (key, _) = range.random_between(&increments, randomizer.clone());

        // println!("TARGET: {:02x?}", target);

        // let key = BigUint::from(0x68f2 as u32);

        let points = sample(key.clone(), threads * blocks);

        println!("{:?}", points.len());

        let dev = cudarc::driver::CudaDevice::new(0)?;

        dev.load_ptx(desktop::SECP256K1.into(), "sha256", &["demo"])?;

        // let target: [u8; 20] = [0xe8, 0xb5, 0xf0, 0xdc, 0x24, 0x24, 0x01, 0x70, 0xbb, 0xf5, 0x54, 0xf2, 0x24, 0x2e, 0x20, 0x25, 0xaa, 0xa6, 0x97, 0xf8];

        let points: CudaSlice<_> = dev.htod_sync_copy(&points)?;
        // let x: CudaSlice<_> = dev.htod_sync_copy(&x)?;
        // let y: CudaSlice<_> = dev.htod_sync_copy(&y)?;
        let target: CudaSlice<_> = dev.htod_sync_copy(&target)?;

        let mut counter: CudaSlice<u32> = dev.alloc_zeros::<u32>(4)?;

        let mut out = dev.alloc_zeros::<u8>(20)?;

        let hash_function = dev.get_func("sha256", "demo").unwrap();

        let cfg: LaunchConfig = LaunchConfig {
            grid_dim: (blocks, 1, 1),
            block_dim: (threads, 1, 1),
            shared_mem_bytes: 0,
        };

        unsafe { hash_function.launch(cfg, (&points, &target, &mut counter,)) }?;

        let out_host = dev.dtoh_sync_copy(&counter)?;
        let result = BigUint::from_slice(out_host.as_slice());

        if result.is_zero() == false {
            break println!("Answer: {:?}", (&key).add(result).to_str_radix(16));
        }
    }

    Ok(())
}
