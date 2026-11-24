#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

CRATE="special_menu_rust"
CRATE_DIR="$SCRIPT_DIR/$CRATE"
OUT_ROOT="$CRATE_DIR/android_output"
JNI_DEST="$SCRIPT_DIR/app/src/main/jniLibs"
KOTLIN_DEST="$SCRIPT_DIR/app/src/main/java"
BINDINGS_PKG_PATH="z/special_menu/rust"

echo "Building $CRATE (arm64-v8a, x86_64)..."
mkdir -p "$OUT_ROOT/jniLibs" "$OUT_ROOT/kotlin"
(
    cd "$CRATE_DIR"
    cargo ndk -o "$OUT_ROOT/jniLibs" -t arm64-v8a -t x86_64 build --release
)

echo "Generating Kotlin bindings..."
(
    cd "$CRATE_DIR"
    cargo run --bin uniffi-bindgen -- generate \
        --library "$OUT_ROOT/jniLibs/arm64-v8a/lib$CRATE.so" \
        --language kotlin \
        --out-dir "$OUT_ROOT/kotlin" \
        --config "$CRATE_DIR/uniffi.toml" \
        --no-format
)

echo "Copying .so files into app/src/main/jniLibs/..."
mkdir -p "$JNI_DEST/arm64-v8a" "$JNI_DEST/x86_64"
cp "$OUT_ROOT/jniLibs/arm64-v8a/lib$CRATE.so" "$JNI_DEST/arm64-v8a/"
cp "$OUT_ROOT/jniLibs/x86_64/lib$CRATE.so" "$JNI_DEST/x86_64/"

echo "Copying Kotlin bindings into app/src/main/java/$BINDINGS_PKG_PATH/..."
mkdir -p "$KOTLIN_DEST/$BINDINGS_PKG_PATH"
cp "$OUT_ROOT/kotlin/$BINDINGS_PKG_PATH/"*.kt "$KOTLIN_DEST/$BINDINGS_PKG_PATH/"

echo "Done. Now run: ./gradlew :app:assembleDebug"
