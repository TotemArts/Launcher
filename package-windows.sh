#!/bin/sh
cargo build --target=x86_64-pc-windows-gnu --release
wine ResourceHacker.exe -save ./RenegadeX-Launcher.exe -open ./target/x86_64-pc-windows-gnu/release/RenegadeX-Launcher.exe -resource RenegadeX-Launcher.res -action add
rm RenX-Launcher.zip
zip -j9 RenX-Launcher RenegadeX-Launcher.exe sciter.dll SelfUpdateExecutor.exe
zip -9 RenX-Launcher -r dom
