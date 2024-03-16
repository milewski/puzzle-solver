use std::ops::{Add, Rem, Sub};
use std::sync::Arc;

use anyhow::{bail, Context};
use num_bigint::BigUint;
use num_traits::Num;
use serde::{Deserialize, Deserializer};

use crate::puzzle::{Utility, Hasher};

#[derive(Debug)]
pub struct PuzzleRange {
    pub min: BigUint,
    pub max: BigUint,
}

impl PuzzleRange {
    fn from_str(range: &str) -> anyhow::Result<Self> {
        let mut values = range
            .split(':')
            .map(|value| BigUint::from_str_radix(value, 16));

        if let (Some(Ok(min)), Some(Ok(max))) = (values.next(), values.next()) {
            if min > max {
                return bail!("Invalid range: min should be less than or equal to max");
            }

            Ok(Self { min, max })
        } else {
            bail!("Invalid input format")
        }
    }

    fn random<R: Hasher>(&self, randomizer: Arc<Utility<R>>) -> BigUint {
        let bits = (self.max.bits() / 8) as usize;
        let random = &randomizer.random_bytes(bits + 1);
        let random = BigUint::from_bytes_le(&random);

        let min = &self.min;
        let max = &self.max;

        min.add(random.rem(max.sub(min).add(1u8)))
    }

    pub fn random_between<R: Hasher>(&self, increments: &BigUint, randomizer: Arc<Utility<R>>) -> (BigUint, BigUint) {
        let mut min = self.random(randomizer);
        let mut max = (&min).add(increments).min(self.max.clone());

        (min, max)
    }
}

#[derive(Deserialize, Debug, Default, Copy, Clone)]
pub struct PuzzleJson {
    number: u8,
    address: &'static str,
    range: &'static str,
    solution: Option<&'static str>,
}

impl PuzzleJson {
    pub fn range(&self) -> anyhow::Result<PuzzleRange> {
        PuzzleRange::from_str(self.range)
    }

    pub fn target(&self) -> anyhow::Result<[u8; 20]> {
        let mut target = [0u8; 20];

        let decoded = bs58::decode(self.address).into_vec()?;

        for (index, data) in decoded[1..=20].into_iter().enumerate() {
            target[index] = *data
        }

        Ok(target)
    }
}

pub struct Puzzles {
    puzzles: [PuzzleJson; 160],
}

impl Puzzles {
    pub fn load() -> anyhow::Result<Self> {
        let puzzles = include_bytes!("./puzzles.json");
        let parsed: Vec<PuzzleJson> = serde_json::from_slice(puzzles).context("unable to parse puzzles!")?;
        let mut puzzles = [PuzzleJson::default(); 160];

        for (index, puzzle) in parsed.into_iter().enumerate() {
            puzzles[index] = puzzle
        }

        Ok(Self { puzzles })
    }

    pub fn get_puzzle(&self, puzzle: u8) -> PuzzleJson {
        self.puzzles[(puzzle as usize) - 1]
    }
}
