#!/bin/sh
cd backend && \
cargo update && \
cross build --target=i686-pc-windows-gnu --release && \
cd ..
cp ./backend/target/i686-pc-windows-gnu/release/RenegadeX-Launcher.exe "./Renegade X Launcher.exe" && \
(rm RenX-Launcher.zip || true) && \
zip -j9 RenX-Launcher "Renegade X Launcher.exe" "RenegadeX-folder-permissions.exe" "sciter.dll" "SelfUpdateExecutor.exe" && \
zip -9 RenX-Launcher -r "dom"