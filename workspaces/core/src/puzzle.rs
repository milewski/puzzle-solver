use core::panic;
use std::ops::{Add, AddAssign, Rem, Sub};
use std::str::FromStr;
use std::sync::Arc;

use anyhow::anyhow;
use k256::{ProjectivePoint, PublicKey, SecretKey};
use k256::elliptic_curve::group::GroupEncoding;
use num_bigint::BigUint;
use num_traits::{Num, One};
use serde::{Deserialize, Serialize};

trait PuzzleNumber {
    fn is_valid(&self) -> bool;
}

impl PuzzleNumber for u8 {
    fn is_valid(&self) -> bool {
        *self >= 1 && *self <= 120
    }
}

pub struct PuzzleManager<N: PuzzleNumber, T> {
    puzzle: WorkingPuzzle<N>,
    randomizer: Arc<GenericRandomizer<T>>,
    events: Vec<Box<dyn Fn(Event) + 'static>>
}

pub enum Event {
    SolutionFound,
    ReportHashRate
}

impl<N: PuzzleNumber, T: RandomizerTrait> PuzzleManager<N, T> {
    pub fn new(number: N, range: PuzzleRange, randomizer: T) -> Self {
        Self {
            puzzle: WorkingPuzzle {
                number,
                range
            },
            randomizer: Arc::new(GenericRandomizer::new(randomizer)),
            events: vec![],
        }
    }

    pub fn start(&mut self) {
        let randomizer = self.randomizer.clone();
        let mut worker = Worker {
            range: String::from_str("4000:7fff").unwrap(),
            ripemd160_address: [0; 20],
            randomizer
        };

        worker.random_mode();
    }

    pub fn on_event(&mut self, callable: impl Fn(Event) + 'static) {
        self.events.push(Box::new(callable))
    }
}

struct Worker<T> {
    range: String,
    ripemd160_address: [u8; 20],
    randomizer: Arc<GenericRandomizer<T>>,
}

impl<T> Worker<T> where T: RandomizerTrait {
    fn get_public_key(&self, private_key: &BigUint) -> anyhow::Result<PublicKey> {
        let mut private_key_bytes = private_key.to_bytes_le();
        private_key_bytes.resize(32, 0);
        private_key_bytes.reverse();

        let secret = SecretKey::from_slice(&private_key_bytes)?;

        Ok(secret.public_key())
    }

    fn random_in_range(&self, min: &BigUint, max: &BigUint) -> BigUint {
        if min > max {
            panic!("Invalid range: min should be less than or equal to max");
        }

        let bits = (max.bits() / 8) as usize;
        let random = BigUint::from_bytes_le(&self.randomizer.randomizer.random_buffer::<21>()[0..=bits]);

        min.add(random.rem(max.sub(min).add(1u8)))
    }

    fn range(&self) -> (BigUint, BigUint) {
        let range: Vec<BigUint> = self.range
            .split(':')
            .map(|value| BigUint::from_str_radix(value, 16).unwrap())
            .collect();

        (range[0].clone(), range[1].clone())
    }

    fn random_mode(&mut self) -> anyhow::Result<BigUint> {
        let (low, high) = self.range();
        let increments = BigUint::from(u16::MAX);

        loop {
            let random = self.random_in_range(&low, &high);

            let mut min = random;
            let mut max = min.clone().add(&increments).min(high.clone());

            if let Ok(private_key) = self.compute(&min, &max) {
                return Ok(private_key);
            }
        }
    }

    fn compute(&mut self, min: &BigUint, max: &BigUint) -> anyhow::Result<BigUint> {
        let mut counter = min.clone();
        let mut public_key = self.get_public_key(&counter)?.to_projective();

        while counter <= *max {
            let sha256: [u8; 32] = self.randomizer.randomizer.sha256(&public_key.to_bytes());
            let ripemd160: [u8; 20] = self.randomizer.randomizer.ripemd160(&sha256);

            if self.ripemd160_address == ripemd160 {
                return Ok(counter);
            }

            public_key.add_assign(ProjectivePoint::GENERATOR);

            counter = counter.add(BigUint::one());
        }

        Err(anyhow!("Solution not found..."))
    }
}

pub struct PuzzleRange {
    pub min: BigUint,
    pub max: BigUint
}

pub struct WorkingPuzzle<N: PuzzleNumber> {
    number: N,
    range: PuzzleRange,
}

// impl<N: PuzzleNumber> WorkingPuzzle<N> {
//     fn new() -> Self {}
// }

pub struct Puzzle<T: RandomizerTrait> {
    pub number: u8,
    ripemd160_address: [u8; 20],
    address: String,
    range: String,
    solution: Option<String>,
    randomizer: GenericRandomizer<T>
}

pub struct GenericRandomizer<T> {
    randomizer: T
}

impl<T> GenericRandomizer<T> {
    pub fn new(randomizer: T) -> Self {
        Self { randomizer }
    }
}

pub trait RandomizerTrait {
    fn random_buffer<const N: usize>(&self) -> [u8; N];
    fn sha256(&self, bytes: &[u8]) -> [u8; 32];
    fn ripemd160(&self, bytes: &[u8]) -> [u8; 20];
}

#[derive(Deserialize, Serialize, Debug)]
pub struct PuzzleJson {
    number: u8,
    address: String,
    range: String,
    private: Option<String>,
}

// impl<T: RandomizerTrait> Puzzle<T> {
//     pub fn new(
//         randomizer: GenericRandomizer<T>,
//         number: u8,
//         address: String,
//         range: String,
//         solution: Option<String>
//     ) -> Puzzle<T> {
//         let decoded = bs58::decode(address.clone()).into_vec().unwrap();
//         let ripemd160_address: [u8; 20] = decoded[1..=20].try_into().unwrap();
//
//         Puzzle {
//             randomizer,
//             number,
//             ripemd160_address,
//             address,
//             range,
//             solution,
//         }
//     }
//
//     // pub fn number(data: usize) -> Puzzle {
//     //     let puzzles = include_bytes!("./puzzles.json");
//     //     let puzzles: Vec<PuzzleJson> = serde_json::from_slice(puzzles).unwrap();
//     //     let data = puzzles.get(data - 1).unwrap();
//     //
//     //     Puzzle::from_json(data)
//     // }
//
//     // pub fn from_json(data: &PuzzleJson) -> Puzzle {
//     //     Puzzle::new(
//     //         data.number.to_owned(),
//     //         data.address.to_owned(),
//     //         data.range.to_owned(),
//     //         data.private.to_owned(),
//     //     )
//     // }
//
//     pub fn start(&mut self) -> anyhow::Result<BigUint> {
//         println!("Starting puzzle #{} {:?}", self.number, self.address);
//
//         self.random_mode()
//     }
//
//     fn range(&self) -> (BigUint, BigUint) {
//         let range: Vec<BigUint> = self.range
//             .split(':')
//             .map(|value| BigUint::from_str_radix(value, 16).unwrap())
//             .collect();
//
//         (range[0].clone(), range[1].clone())
//     }
//
//     fn random_in_range(&self, min: &BigUint, max: &BigUint) -> BigUint {
//         if min > max {
//             panic!("Invalid range: min should be less than or equal to max");
//         }
//
//         let bits = (max.bits() / 8) as usize;
//         let random = BigUint::from_bytes_le(&self.randomizer.randomizer.random_buffer::<21>()[0..=bits]);
//
//         min.add(random.rem(max.sub(min).add(1u8)))
//     }
//
//     pub fn random_mode(&mut self) -> anyhow::Result<BigUint> {
//         let (low, high) = self.range();
//         let increments = BigUint::from(u16::MAX);
//
//         loop {
//             let random = self.random_in_range(&low, &high);
//
//             let mut min = random;
//             let mut max = min.clone().add(&increments).min(high.clone());
//
//             println!("min: {:?} low: {:?} high: {:?}, diff: {:?}", min, low, max, max.clone().sub(min.clone()));
//
//             if let Ok(private_key) = self.compute(&min, &max) {
//                 return Ok(private_key);
//             }
//         }
//     }
//
//     fn get_public_key(&self, private_key: &BigUint) -> anyhow::Result<PublicKey> {
//         let mut private_key_bytes = private_key.to_bytes_le();
//         private_key_bytes.resize(32, 0);
//         private_key_bytes.reverse();
//
//         let secret = SecretKey::from_slice(&private_key_bytes)?;
//
//         Ok(secret.public_key())
//     }
//
//     fn compute(&mut self, min: &BigUint, max: &BigUint) -> anyhow::Result<BigUint> {
//         let mut counter = min.clone();
//         let mut public_key = self.get_public_key(&counter)?.to_projective();
//
//         while counter <= *max {
//             let sha256: [u8; 32] = self.randomizer.randomizer.sha256(&public_key.to_bytes());
//             let ripemd160: [u8; 20] = self.randomizer.randomizer.ripemd160(&sha256);
//
//             if self.ripemd160_address == ripemd160 {
//                 println!("Found Solution: {:?}", counter.to_str_radix(16));
//                 return Ok(counter);
//             }
//
//             public_key.add_assign(ProjectivePoint::GENERATOR);
//
//             counter = counter.add(BigUint::one());
//
//             // FreeRtos::delay_ms(1);
//         }
//
//         Err(anyhow!("Solution not found..."))
//     }
// }
