use std::ops::{Add, Rem, Sub};
use std::sync::Arc;

use anyhow::Context;
use num_bigint::BigUint;
use num_traits::Num;
use serde::{Deserialize, Deserializer};

use crate::puzzle::{Hasher, Utility};

#[derive(Debug)]
pub enum Error {
    InvalidRange,
    InvalidInputFormat,
}

#[derive(Debug)]
pub struct PuzzleRange {
    pub min: BigUint,
    pub max: BigUint,
}

impl PuzzleRange {
    fn from_str(range: &str) -> Result<Self, Error> {
        let mut values = range
            .split(':')
            .map(|value| BigUint::from_str_radix(value, 16));

        if let (Some(Ok(min)), Some(Ok(max))) = (values.next(), values.next()) {
            if min > max {
                return Err(Error::InvalidRange);
            }

            Ok(Self { min, max })
        } else {
            return Err(Error::InvalidInputFormat);
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

#[derive(Deserialize, Debug, Default, PartialEq, Copy, Clone)]
pub struct PuzzleDescriptor {
    number: u8,
    address: &'static str,
    range: &'static str,
    solution: Option<&'static str>,
}

impl PuzzleDescriptor {
    pub fn range(&self) -> Result<PuzzleRange, Error> {
        PuzzleRange::from_str(self.range)
    }

    pub fn target(&self) -> Option<[u8; 20]> {
        let mut target = [0u8; 20];

        match bs58::decode(self.address).into_vec() {
            Err(_) => None,
            Ok(decoded) => {
                for (index, data) in decoded[1..=20].into_iter().enumerate() {
                    target[index] = *data
                }

                Some(target)
            }
        }
    }
}

pub struct Puzzles {
    puzzles: [PuzzleDescriptor; 160],
}

impl Puzzles {
    pub fn new() -> Self {
        let puzzles = include_bytes!("./puzzles.json");
        let parsed: Vec<PuzzleDescriptor> = serde_json::from_slice(puzzles).expect("unable to initialize puzzles!");
        let mut puzzles = [PuzzleDescriptor::default(); 160];

        for (index, puzzle) in parsed.into_iter().enumerate() {
            puzzles[index] = puzzle
        }

        Self { puzzles }
    }

    pub fn get(&self, puzzle: u8) -> Option<&PuzzleDescriptor> {
        match puzzle {
            0 | 161.. => None,
            _ => self.puzzles.get((puzzle as usize) - 1)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn can_fetch_all_puzzles() -> Result<(), Error> {
        let puzzle = Puzzles::new();

        for number in 1..=160 {
            assert_eq!(puzzle.get(number).unwrap().number, number);
        }

        Ok(())
    }

    #[test]
    pub fn error_when_puzzle_number_is_invalid() {
        let puzzle = Puzzles::new();

        assert_eq!(puzzle.get(0), None);
        assert_eq!(puzzle.get(161), None);
    }

    #[test]
    pub fn puzzle_target_is_correct() {
        let puzzle = Puzzles::new();
        let puzzle = puzzle.get(66).unwrap();

        assert_eq!(
            puzzle.target(), Some([
                0x20, 0xd4, 0x5a, 0x6a, 0x76, 0x25, 0x35, 0x70, 0x0c, 0xe9,
                0xe0, 0xb2, 0x16, 0xe3, 0x19, 0x94, 0x33, 0x5d, 0xb8, 0xa5
            ])
        );
    }
}