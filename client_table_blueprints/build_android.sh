#!/usr/bin/env bash
set -e
cd "$(dirname "$0")"

cargo ndk -o ./android_output/jniLibs -t arm64-v8a build --release
cargo run --bin uniffi-bindgen -- generate \
    --library ./android_output/jniLibs/arm64-v8a/libclient_table_blueprints.so \
    --language kotlin \
    --out-dir ./android_output/kotlin \
    --config ./src/uniffi.toml \
    --no-format
rm -f ./android_output/jniLibs/arm64-v8a/libprotocol-*.so
ls -1t ./android_output/jniLibs/arm64-v8a/libmy_yrs_lib-*.so 2>/dev/null | tail -n +2 | xargs -r rm -f
