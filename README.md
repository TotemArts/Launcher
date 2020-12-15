# RenegadeX Launcher
To compile for windows (only compile from linux/wsl):
```bash
./package-windows.sh
```

Some build-requirements for Arch Linux:
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
