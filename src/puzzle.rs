use serde::{Deserialize, Serialize};
use std::{include_bytes, println};
use num_bigint::BigUint;
use core::panic;
use esp_idf_sys::{esp_fill_random, mbedtls_ripemd160, mbedtls_sha256};
use k256::{ProjectivePoint, PublicKey, SecretKey};
use esp_idf_hal::delay::FreeRtos;
use anyhow::anyhow;
use num_traits::{Num, One};
use std::ops::{Add, AddAssign, Rem, Sub};
use k256::elliptic_curve::group::GroupEncoding;

pub struct Puzzle {
    number: u8,
    ripemd160_address: [u8; 20],
    address: String,
    range: String,
    solution: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct PuzzleJson {
    number: u8,
    address: String,
    range: String,
    private: Option<String>,
}

impl Puzzle {
    pub fn new(number: u8, address: String, range: String, solution: Option<String>) -> Puzzle {
        let decoded = bs58::decode(address.clone()).into_vec().unwrap();
        let ripemd160_address: [u8; 20] = decoded[1..=20].try_into().unwrap();

        Puzzle {
            number,
            ripemd160_address,
            address,
            range,
            solution,
        }
    }

    pub fn number(data: usize) -> Puzzle {
        let puzzles = include_bytes!("./puzzles.json");
        let puzzles: Vec<PuzzleJson> = serde_json::from_slice(puzzles).unwrap();
        let data = puzzles.get(data - 1).unwrap();

        Puzzle::from_json(data)
    }

    pub fn from_json(data: &PuzzleJson) -> Puzzle {
        Puzzle::new(
            data.number.to_owned(),
            data.address.to_owned(),
            data.range.to_owned(),
            data.private.to_owned(),
        )
    }

    pub fn start(&mut self) -> anyhow::Result<String> {
        println!("Starting puzzle #{} {:?}", self.number, self.address);

        self.random_mode()
    }

    fn range(&self) -> (BigUint, BigUint) {
        let range: Vec<BigUint> = self.range
            .split(':')
            .map(|value| BigUint::from_str_radix(value, 16).unwrap())
            .collect();

        (range[0].clone(), range[1].clone())
    }

    fn random_in_range(&self, min: &BigUint, max: &BigUint) -> BigUint {
        if min > max {
            panic!("Invalid range: min should be less than or equal to max");
        }

        let random = unsafe {
            let mut result: [u8; 21] = [0; 21];
            let bits = (max.bits() / 8) as usize;

            esp_fill_random(
                &mut result as *mut _ as *mut std::ffi::c_void,
                result.len(),
            );

            BigUint::from_bytes_le(&result[0..=bits])
        };

        min.add(random.rem(max.sub(min).add(1u8)))
    }

    pub fn random_mode(&mut self) -> anyhow::Result<String> {
        let (low, high) = self.range();
        let increments = BigUint::from(u16::MAX);

        loop {
            let random = self.random_in_range(&low, &high);

            let mut min = random;
            let mut max = min.clone().add(&increments).min(high.clone());

            println!("min: {:?} low: {:?} high: {:?}, diff: {:?}", min, low, max, max.clone().sub(min.clone()));

            if let Ok(private_key) = self.compute(&min, &max) {
                return Ok(private_key);
            }
        }
    }

    fn get_public_key(&self, private_key: &BigUint) -> anyhow::Result<PublicKey> {
        let mut private_key_bytes = private_key.to_bytes_le();
        private_key_bytes.resize(32, 0);
        private_key_bytes.reverse();

        let secret = SecretKey::from_slice(&private_key_bytes)?;

        // println!("Public {:?}", secret.public_key().to_projective().to_bytes());

        Ok(secret.public_key())
    }

    fn compute(&mut self, min: &BigUint, max: &BigUint) -> anyhow::Result<String> {
        let mut counter = min.clone();
        let mut public_key = self.get_public_key(&counter)?.to_projective();

        while counter <= *max {
            let sha256: [u8; 32] = unsafe {
                let bytes = public_key.to_bytes();
                let length = bytes.len();
                let mut output = [0; 32];

                mbedtls_sha256(bytes.as_ptr(), length, output.as_mut_ptr(), 0);
                output
            };

            let ripemd160: [u8; 20] = unsafe {
                let length = sha256.len();
                let mut output = [0; 20];
                mbedtls_ripemd160(sha256.as_ptr(), length, output.as_mut_ptr());
                output
            };

            if self.ripemd160_address == ripemd160 {
                println!("found: {:?}", counter.to_str_radix(16));

                return Ok(counter.to_str_radix(16));
            }

            public_key.add_assign(ProjectivePoint::GENERATOR);

            counter = counter.add(BigUint::one());

            FreeRtos::delay_ms(1);
        }

        Err(anyhow!("Solution not found..."))
    }
}
