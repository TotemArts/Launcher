#!/bin/sh
export PATH=$PATH:$PWD/../sciter-js-sdk/bin/linux/x64/

cargo update --manifest-path backend/Cargo.toml && \
cargo run --manifest-path backend/Cargo.toml
