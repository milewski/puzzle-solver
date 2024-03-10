## Instructions

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

## On windows attach to using WSL

Use the following commands to share usb device between the host and wsl

- Install https://github.com/dorssel/usbipd-win
- usbipd wsl list
- usbipd attach --wsl --busid 2-2
