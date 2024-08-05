fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    #[cfg(feature = "cuda")]
    bindgen_cuda::Builder::default()
        .include_paths(vec!["src/cuda/secp256k1.cu"])
        .build_ptx()
        .unwrap();
}