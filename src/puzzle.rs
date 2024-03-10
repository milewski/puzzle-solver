use core::panic;
use std::{include_bytes, println};
use std::ops::{Add, AddAssign, Rem, Sub};

use anyhow::anyhow;
use esp_idf_hal::delay::FreeRtos;
use k256::{ProjectivePoint, PublicKey, SecretKey};
use k256::elliptic_curve::group::GroupEncoding;
use num_bigint::BigUint;
use num_traits::{Num, One};
use serde::{Deserialize, Serialize};

use crate::utilities::{random_buffer, ripemd160, sha256};

pub struct Puzzle {
    pub number: u8,
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

    pub fn start(&mut self) -> anyhow::Result<BigUint> {
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

        let bits = (max.bits() / 8) as usize;
        let random = BigUint::from_bytes_le(&random_buffer::<21>()[0..=bits]);

        min.add(random.rem(max.sub(min).add(1u8)))
    }

    pub fn random_mode(&mut self) -> anyhow::Result<BigUint> {
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

        Ok(secret.public_key())
    }

    fn compute(&mut self, min: &BigUint, max: &BigUint) -> anyhow::Result<BigUint> {
        let mut counter = min.clone();
        let mut public_key = self.get_public_key(&counter)?.to_projective();

        while counter <= *max {
            let sha256: [u8; 32] = sha256(&public_key.to_bytes());
            let ripemd160: [u8; 20] = ripemd160(&sha256);

            if self.ripemd160_address == ripemd160 {
                println!("Found Solution: {:?}", counter.to_str_radix(16));
                return Ok(counter);
            }

            public_key.add_assign(ProjectivePoint::GENERATOR);

            counter = counter.add(BigUint::one());

            FreeRtos::delay_ms(1);
        }

        Err(anyhow!("Solution not found..."))
    }
}
