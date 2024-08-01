use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let builder = bindgen_cuda::Builder::default()
        .include_paths(vec![
            Path::new("./src/cuda/sha256.cu"),
            Path::new("src/cuda/secp256k1/ecc.cu"),
        ]);

    let bindings = builder.build_ptx().unwrap();
    bindings.write("src/lib.rs").unwrap();
}