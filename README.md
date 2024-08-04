# Bitcoin Puzzle Solver Project

## What is this project about?

https://privatekeys.pw/puzzles/bitcoin-puzzle-tx

> In 2015, in order to show the hugeness of the private key space (or maybe just for fun), someone created a "puzzle"
> where he chose keys in a certain smaller space and sent increasing amounts to each of those keys like this [...]

Several puzzles remain unsolved, amounting to a total of 956.5 BTC!

The goal of this project is to tackle these puzzles. Currenly it only works on CPU and Nvidia GPUs expecificaly via CUDA.

# How to Run the Solver

## Run on GPU 

>> Note: This works only with Nvidia GPUs.

```shell
./puzzle-solver.exe --puzzle 66 gpu
```

### Options:

```
./puzzle-solver.exe --puzzle 66 gpu \ 
    --threads 1024 \
    --blocks 1024
```

## Run on CPU

```shell
./puzzle-solver.exe --puzzle 66 cpu
```

## Donation

If you're feeling generous, please consider a donation. Every bit helps.

```
BTC: bc1qkrcpyq9ep20nkkkh60jev7mpf0ytgjhev04aaz
USDT/ETH: 0xcc13f793a3842fD3fE192f5358249612Fa3D173F
```
