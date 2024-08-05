# Bitcoin Puzzle Solver Project

## What is this project about?

https://privatekeys.pw/puzzles/bitcoin-puzzle-tx

> In 2015, in order to show the hugeness of the private key space (or maybe just for fun), someone created a "puzzle"
> where he chose keys in a certain smaller space and sent increasing amounts to each of those keys like this [...]

Several puzzles remain unsolved, amounting to a total of 956.5 BTC!

The goal of this project is to tackle these puzzles. Currently, it only works on CPU and Nvidia GPUs specifically via CUDA.

# Download

You can find the binaries on the [releases](https://github.com/milewski/puzzle-solver/releases) page. 

- Windows: [puzzle-solver (CPU + GPU)](https://github.com/milewski/puzzle-solver/releases/download/0.1.1/x86_64-windows_puzzle-solver.exe)
- Linux: [puzzle-solver (CPU + GPU)](https://github.com/milewski/puzzle-solver/releases/download/0.1.1/x86_64-linux_puzzle-solver)
- Mac (Intel): [puzzle-solver (CPU)](https://github.com/milewski/puzzle-solver/releases/download/0.1.1/x86_64-apple_puzzle-solver)
- Mac (ARM): [puzzle-solver (CPU)](https://github.com/milewski/puzzle-solver/releases/download/0.1.1/aarch64-apple_puzzle-solver)

# How to Run the Solver

## Run on GPU (Nvidia)

```shell
./puzzle-solver.exe --puzzle 66 gpu
```

#### Options:

```
./puzzle-solver.exe --puzzle 66 gpu \ 
    --threads 1024 \
    --blocks 1024
```

## Run on CPU

```shell
./puzzle-solver.exe --puzzle 66 cpu
```

#### Options:

```
./puzzle-solver.exe --puzzle 66 cpu \ 
    --threads 8
```

## Run from Source

```
cargo run --release --features cuda --puzzle 66 gpu
```

## Donation

If you're feeling generous, please consider a donation. Every bit helps.

```
 BTC: bc1qkrcpyq9ep20nkkkh60jev7mpf0ytgjhev04aaz
 ETH: 0xcc13f793a3842fD3fE192f5358249612Fa3D173F
USDT: 0xcc13f793a3842fD3fE192f5358249612Fa3D173F
```
