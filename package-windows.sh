#!/bin/sh
cargo update --manifest-path="windows-target/Cargo.toml" && \
cross build --manifest-path="windows-target/Cargo.toml" --target=i686-pc-windows-gnu --release && \
cp ./target/i686-pc-windows-gnu/release/RenegadeX-Launcher.exe "./windows-target/Renegade X Launcher.exe" && \
(rm windows-target/RenX-Launcher.zip || true) && \
zip -j9 windows-target/RenX-Launcher "windows-target/Renegade X Launcher.exe" "windows-target/RenegadeX-folder-permissions.exe" "windows-target/sciter.dll" "windows-target/SelfUpdateExecutor.exe" && \
zip -9 windows-target/RenX-Launcher -r "dom"