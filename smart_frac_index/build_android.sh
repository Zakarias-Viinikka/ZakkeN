#!/usr/bin/env bash
set -e
cd "$(dirname "$0")"

cargo ndk -o ./android_output_folder/jniLibs -t arm64-v8a build --release
cargo run --bin uniffi-bindgen -- generate \
    --library ./android_output_folder/jniLibs/arm64-v8a/libsmart_frac_index.so \
    --language kotlin \
    --out-dir ./android_output_folder/kotlin \
    --config ./uniffi.toml \
    --no-format
# cargo ndk may leave a hashed duplicate behind; keep only the canonical name.
ls -1t ./android_output_folder/jniLibs/arm64-v8a/libsmart_frac_index-*.so 2>/dev/null | tail -n +2 | xargs -r rm -f
