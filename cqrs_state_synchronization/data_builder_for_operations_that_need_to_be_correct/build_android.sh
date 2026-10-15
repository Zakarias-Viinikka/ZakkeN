#!/usr/bin/env bash
set -e
cd "$(dirname "$0")"

cargo ndk -o ./android_output/jniLibs -t arm64-v8a build --release
cargo run --bin uniffi-bindgen -- generate \
    --library ./android_output/jniLibs/arm64-v8a/libdata_builder_for_operations_that_need_to_be_correct.so \
    --language kotlin \
    --out-dir ./android_output/kotlin \
    --config ./uniffi.toml \
    --no-format
