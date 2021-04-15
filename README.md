# RenegadeX Launcher

![screenshot](screenshot.png)

### Makes use of the following dependencies of the same developer (SonnyX):

[RenegadeX-patcher-lib](https://github.com/SonnyX/RenegadeX-patcher-lib)

[xDelta-decoder](https://github.com/SonnyX/xdelta-decoder-rust)

[Download-Async](https://github.com/SonnyX/download-async)

[UnZip](https://github.com/SonnyX/unzip-rs)

[RunAs](https://github.com/SonnyX/rust-runas)

## Compilation Instructions
Cross-Compiling relies on Docker, make sure to have it installed!
```bash
cargo install cross
```

To compile for windows (only compile from linux/wsl):
```bash
./package-windows.sh
```

Some build-requirements for Arch Linux:
```bash
sudo pacman -Syu cmake gcc pkgconf wget openssl
```

Build commands for linux:
```bash
cargo update
cargo build --release
```

Run commands for Linux:
Download Sciter from `https://github.com/c-smile/sciter-sdk/` and add the path to $SCITER_PATH
```
export PATH=$PATH:$SCITER_PATH/bin.gtk/x64
cargo run --release
```