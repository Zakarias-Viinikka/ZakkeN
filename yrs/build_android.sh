#!/usr/bin/env bash
set -e
cd "$(dirname "$0")"

cargo ndk -o ./android_output_folder/jniLibs -t arm64-v8a build --release
cargo run --bin uniffi-bindgen -- generate \
    --library ./android_output_folder/jniLibs/arm64-v8a/libmy_yrs_lib.so \
    --language kotlin \
    --out-dir ./android_output_folder/kotlin \
    --config ./uniffi.toml \
    --no-format
