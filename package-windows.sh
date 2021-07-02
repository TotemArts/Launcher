#!/bin/sh
cargo update --manifest-path="backend/Cargo.toml" && \
cross build --manifest-path="backend/Cargo.toml" --target=i686-pc-windows-gnu --release && \
cp ./backend/target/i686-pc-windows-gnu/release/RenegadeX-Launcher.exe "./Renegade X Launcher.exe" && \
(rm RenX-Launcher.zip || true) && \
zip -j9 RenX-Launcher "Renegade X Launcher.exe" "RenegadeX-folder-permissions.exe" "sciter.dll" "SelfUpdateExecutor.exe" && \
zip -9 RenX-Launcher -r "dom"