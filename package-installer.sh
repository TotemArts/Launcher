#!/bin/sh
cd installer && \
cross +nightly update && \
cross +nightly build --target=i686-pc-windows-gnu --release && \
cd .. && \
cp ./installer/target/i686-pc-windows-gnu/release/Launcher-Installer.exe "./Launcher-Installer.exe"