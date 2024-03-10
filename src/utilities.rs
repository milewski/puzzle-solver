use esp_idf_sys::{esp_fill_random, mbedtls_ripemd160, mbedtls_sha256};

pub fn random_buffer<const N: usize>() -> [u8; N] {
    let mut buffer: [u8; N] = [0; N];

    unsafe {
        esp_fill_random(buffer.as_mut().as_mut_ptr() as *mut std::ffi::c_void, N);
        buffer
    }
}

pub fn sha256(bytes: &[u8]) -> [u8; 32] {
    unsafe {
        let length = bytes.len();
        let mut output = [0; 32];

        mbedtls_sha256(bytes.as_ptr(), length, output.as_mut_ptr(), 0);
        output
    }
}

pub fn ripemd160(bytes: &[u8]) -> [u8; 20] {
    unsafe {
        let length = bytes.len();
        let mut output = [0; 20];
        mbedtls_ripemd160(bytes.as_ptr(), length, output.as_mut_ptr());
        output
    }
}
