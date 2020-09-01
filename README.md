# RenegadeX Launcher
Some build-requirements:
```bash
sudo tee -a /etc/pacman.conf > /dev/null <<EOT
[ownstuff]
SigLevel = PackageOptional
Server = https://ftp.f3l.de/~martchus/$repo/os/$arch
Server = https://martchus.no-ip.biz/repo/arch/$repo/os/$arch
EOT
sudo pacman -Syu cmake gcc pkgconf wget openssl mingw-w64-gcc
```

Build commands for linux:
```bash
cargo update
cargo build --release
```
The first time you might get an issue such as:
```bash
warning: spurious network error (2 tries remaining): invalid Content-Type: application/octet-stream; class=Net (12)
warning: spurious network error (1 tries remaining): invalid Content-Type: application/octet-stream; class=Net (12)
error: failed to load source for a dependency on `xdelta`

Caused by:
  Unable to update https://github.com/SonnyX/xdelta-decoder-rust.git

Caused by:
  failed to update submodule `lzma-sys/xz`

Caused by:
  failed to fetch submodule `lzma-sys/xz` from https://git.tukaani.org/xz.git

Caused by:
  invalid Content-Type: application/octet-stream; class=Net (12)
```
If this happens, then run the following:
```bash
./fix-build-issue.sh
```

Run commands for Linux:
Download Sciter from `https://github.com/c-smile/sciter-sdk/` and add the path to $SCITER_PATH
```
export PATH=$PATH:$SCITER_PATH/bin.gtk/x64
cargo run --release
```

Build commands for windows from linux:
```bash
./package-windows.sh
```
