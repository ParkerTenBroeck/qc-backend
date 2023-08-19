#!/bin/bash

rm common.zip windows.zip linux.zip
zip common.zip -r ./db/diesel/migrations ./static ./template_scripts ./templates ./diesel.toml ./Rocket.toml ./config.json5

# cargo clean


cargo build --release --target x86_64-unknown-linux-gnu
cp common.zip linux.zip
zip linux.zip -j ./target/x86_64-unknown-linux-gnu/release/qc-backend

cargo build --release --target x86_64-pc-windows-gnu
cp common.zip windows.zip
zip windows.zip -j ./target/x86_64-pc-windows-gnu/release/qc-backend.exe

rm common.zip