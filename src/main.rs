use std::thread::sleep;
use std::time::Duration;
use bitcoin::hashes::Hash;
use cudarc::driver::{LaunchAsync, LaunchConfig};
use k256::elliptic_curve::group::GroupEncoding;
use k256::SecretKey;
use num_bigint::BigUint;
use num_traits::{FromPrimitive, ToBytes};
use rand::RngCore;

use crate::puzzle::PuzzleManager;
use crate::randomizer::Randomizer;

mod puzzles;
mod reporter;
mod puzzle;
mod worker;
mod randomizer;

fn sample() {

    let key = BigUint::from(1u8);
    let mut buffer = [0; 32];

    for (index, byte) in key.to_le_bytes().into_iter().enumerate() {
        buffer[32 - index - 1] = byte;
    }

    let point = SecretKey::from_slice(&buffer).unwrap()
        .public_key()
        .to_projective();

    println!("{:?}", point.to_bytes().0)

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

    sample();

    let phrase = "hello_world2";
    // let sha256 =   bitcoin::hashes::sha256::Hash::hash(phrase.as_bytes()).to_byte_array();

    // println!("{:?}", sha256);

    let dev = cudarc::driver::CudaDevice::new(0)?;

    dev.load_ptx(desktop::SHA256.into(), "sha256", &["kernel_sha256_hash"])?;

    let mut out = dev.alloc_zeros::<u8>(32)?;
    let mut indata = dev.htod_copy(phrase.as_bytes().to_vec())?;
    let mut indata_length = phrase.len();

    let hash_function = dev.get_func("sha256", "kernel_sha256_hash").unwrap();

    let cfg: LaunchConfig = LaunchConfig {
        grid_dim: (1, 1, 1),
        block_dim: (1, 1, 1),
        shared_mem_bytes: 0,
    };

    unsafe { hash_function.launch(cfg, (&indata, indata_length, &mut out,)) }?;

    let out_host = dev.dtoh_sync_copy(&out)?;

    println!("{:?}", out_host);


    // let out_host: Vec<f32> = dev.dtoh_sync_copy(&out)?;
    // assert_eq!(out_host, [1.0; 100].map(f32::sin));

    // // allocate buffers
    // let inp = dev.htod_copy(vec![1.0f32; 100])?;
    // let mut out = dev.alloc_zeros::<f32>(100)?;

    Ok(())
}
