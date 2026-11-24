#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

CRATE_DIR="$SCRIPT_DIR/rapier2d_wrapper"
OUT_ROOT="$CRATE_DIR/android_output"
JNI_DEST="$SCRIPT_DIR/app/src/main/jniLibs"
KOTLIN_DEST="$SCRIPT_DIR/app/src/main/java"

echo "Building rapier2d_wrapper (arm64-v8a, x86_64)..."
mkdir -p "$OUT_ROOT/jniLibs" "$OUT_ROOT/kotlin"
(
    cd "$CRATE_DIR"
    cargo ndk -o "$OUT_ROOT/jniLibs" -t arm64-v8a -t x86_64 build --release
)

echo "Generating Kotlin bindings..."
(
    cd "$CRATE_DIR"
    cargo run --bin uniffi-bindgen -- generate \
        --library "$OUT_ROOT/jniLibs/arm64-v8a/librapier2d_wrapper.so" \
        --language kotlin \
        --out-dir "$OUT_ROOT/kotlin" \
        --config "$CRATE_DIR/uniffi.toml" \
        --no-format
)

echo "Copying .so files into app/src/main/jniLibs/..."
mkdir -p "$JNI_DEST/arm64-v8a" "$JNI_DEST/x86_64"
cp "$OUT_ROOT/jniLibs/arm64-v8a/librapier2d_wrapper.so" "$JNI_DEST/arm64-v8a/"
cp "$OUT_ROOT/jniLibs/x86_64/librapier2d_wrapper.so" "$JNI_DEST/x86_64/"
rm -f "$JNI_DEST/arm64-v8a/libcascade_physics.so" "$JNI_DEST/x86_64/libcascade_physics.so"

echo "Copying Kotlin bindings into app/src/main/java/cascade/..."
mkdir -p "$KOTLIN_DEST/cascade"
cp "$OUT_ROOT/kotlin/cascade/rapier2d_wrapper.kt" "$KOTLIN_DEST/cascade/"
rm -f "$KOTLIN_DEST/cascade/cascade_physics.kt"

echo "Done. Now run: ./gradlew :app:assembleDebug"