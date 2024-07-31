use k256::elliptic_curve::rand_core::RngCore;
use bitcoin::hashes::Hash;
use crate::puzzle::Hasher;

pub struct Randomizer {}

impl Hasher for Randomizer {
    fn random_bytes(&self, bits: usize) -> Vec<u8> {
        let mut buffer = vec![0u8; bits];
        rand::thread_rng().fill_bytes(&mut buffer);
        buffer
    }

    fn sha256(&self, bytes: &[u8]) -> [u8; 32] {
        bitcoin::hashes::sha256::Hash::hash(&bytes).to_byte_array()
    }

    fn ripemd160(&self, bytes: &[u8]) -> [u8; 20] {
        bitcoin::hashes::ripemd160::Hash::hash(bytes).to_byte_array()
    }
}