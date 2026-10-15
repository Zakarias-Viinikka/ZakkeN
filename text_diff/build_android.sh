#!/usr/bin/env bash
set -e
cd "$(dirname "$0")"

rm -rf ./android_output_folder

cargo ndk -o ./android_output_folder/jniLibs -t arm64-v8a build --release --lib
cargo run --target "$(rustc -vV | sed -n 's/host: //p')" --bin uniffi-bindgen -- generate \
    --library ./android_output_folder/jniLibs/arm64-v8a/libtext_diff.so \
    --language kotlin \
    --out-dir ./android_output_folder/kotlin \
    --config ./uniffi.toml \
    --no-format
