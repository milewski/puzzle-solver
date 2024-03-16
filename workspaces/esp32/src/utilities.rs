use esp_idf_sys::{esp_fill_random, mbedtls_ripemd160, mbedtls_sha256};
use shared::puzzle::Hasher;

pub struct Randomizer {}

impl Hasher for Randomizer {
    fn random_bytes(&self, bits: usize) -> Vec<u8> {
        let mut buffer = vec![0u8; bits];
        let mut buffer: [u8; 21] = [0; 21];

        unsafe {
            esp_fill_random(buffer.as_mut().as_mut_ptr() as *mut std::ffi::c_void, bits);
            buffer.to_vec()
        }
    }

    fn sha256(&self, bytes: &[u8]) -> [u8; 32] {
        unsafe {
            let length = bytes.len();
            let mut output = [0; 32];

            mbedtls_sha256(bytes.as_ptr(), length, output.as_mut_ptr(), 0);
            output
        }
    }

    fn ripemd160(&self, bytes: &[u8]) -> [u8; 20] {
        unsafe {
            let length = bytes.len();
            let mut output = [0; 20];
            mbedtls_ripemd160(bytes.as_ptr(), length, output.as_mut_ptr());
            output
        }
    }
}
