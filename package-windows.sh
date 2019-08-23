#!/bin/sh
cargo build --target=x86_64-pc-windows-gnu --release
cp ./target/x86_64-pc-windows-gnu/release/RenegadeX-Launcher.exe ./RenegadeX-Launcher.exe
rm RenX-Launcher.zip
zip -j9 RenX-Launcher RenegadeX-Launcher.exe sciter.dll SelfUpdateExecutor.exe
zip -9 RenX-Launcher -r dom
