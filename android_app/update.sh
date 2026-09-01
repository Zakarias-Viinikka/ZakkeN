#!/bin/bash

BASE_URL="https://raw.githubusercontent.com/Zakarias-Viinikka/z_db/main/db_wrapper/android_output"
JNI_DEST="app/src/main/jniLibs/arm64-v8a"
KOTLIN_DEST="app/src/main/java"

echo "Cleaning and updating database library for Zndroid..."

# 1. Clear old versions to avoid conflicts
rm -rf "$KOTLIN_DEST/com/z_db/android_mascot"
rm -rf "$KOTLIN_DEST/uniffi/protocol"

# 2. Re-create folders
mkdir -p "$JNI_DEST"
mkdir -p "$KOTLIN_DEST/com/z_db/android_mascot"
mkdir -p "$KOTLIN_DEST/uniffi/protocol"

# 3. Download the SO files
curl -L "$BASE_URL/jniLibs/arm64-v8a/libdb_wrapper.so" -o "$JNI_DEST/libdb_wrapper.so"
curl -L "$BASE_URL/jniLibs/arm64-v8a/libprotocol.so" -o "$JNI_DEST/libprotocol.so"

# 4. Download Kotlin Bindings
curl -L "$BASE_URL/kotlin/com/z_db/android_mascot/db_wrapper.kt" -o "$KOTLIN_DEST/com/z_db/android_mascot/db_wrapper.kt"
curl -L "$BASE_URL/kotlin/uniffi/protocol/protocol.kt" -o "$KOTLIN_DEST/uniffi/protocol/protocol.kt"

echo "Done! Your library is now clean and synchronized for Zndroid."
