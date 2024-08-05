run puzzle device:
    cargo run --release -- --puzzle {{ puzzle }} {{ device }}

compile target input output:
    docker compose run --rm builder cargo build --release --target {{ target }} --target-dir output --features cuda
    just cleanup {{ target }} {{ input }} {{ output }}

compile-mac target input output:
    docker compose run --rm builder cargo zigbuild --release --target {{ target }} --target-dir output
    just cleanup {{ target }} {{ input }} {{ output }}

cleanup target input output:
    mv output/{{ target }}/release/{{ input }} output/{{ output }}
    rm -rf \
        output/{{ target }} \
        output/release \
        output/CACHEDIR.TAG \
        output/.rustc_info.json

build:
    rm -rf output
    just compile      x86_64-pc-windows-gnu     puzzle-solver.exe  x86_64-windows_puzzle-solver.exe
    just compile      x86_64-unknown-linux-gnu  puzzle-solver      x86_64-linux_puzzle-solver
    just compile-mac  x86_64-apple-darwin       puzzle-solver      x86_64-apple_puzzle-solver
    just compile-mac  aarch64-apple-darwin      puzzle-solver      aarch64-apple_puzzle-solver
