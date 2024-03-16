## What is this project about?

https://privatekeys.pw/puzzles/bitcoin-puzzle-tx

> In 2015, in order to show the hugeness of the private key space (or maybe just for fun), someone created a "puzzle"
> where he chose keys in a certain smaller space and sent increasing amounts to each of those keys like this [...]

Several puzzles remain unsolved, amounting to a total of 956.5 BTC!

The goal of this project is to tackle these puzzles. It operates on a low-powered ESP32 board,
generating random private keys within the challenge's key space and checking if you are the lucky winner. The likelihood
of success is zero to none, given that the device is millions of times slower than the slowest PC you've ever used.
But who knows? Maybe you're feeling lucky today and want to give it a shot.

## Desktop 

```shell
cargo run -p desktop --release --target x86_64-unknown-linux-gnu
cargo run -p esp32 --release --target xtensa-esp32s3-espidf
```

## Development Setup Guide

Install `esp32` [toolchain](https://github.com/esp-rs/rust-build):

```bash
cargo install espup
cargo install ldproxy
cargo install espflash
espup install
```

## On Mac

To be able to flash via USB you need to install libuv:

```
brew install libuv
```

Manual: https://docs.espressif.com/projects/esp-idf/en/latest/esp32c3/get-started/index.html#installation

## If you are using WSL on windows

Use the following commands to share usb device between the host and wsl

- Install https://github.com/dorssel/usbipd-win
- usbipd list
- usbipd attach --wsl --busid 2-2
