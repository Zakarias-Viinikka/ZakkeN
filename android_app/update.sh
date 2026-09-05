#!/bin/bash

# URLs
Z_DB_URL="https://raw.githubusercontent.com/Zakarias-Viinikka/z_db/main/db_wrapper/android_output"
BLUEPRINTS_URL="https://raw.githubusercontent.com/Zakarias-Viinikka/ZakkeN/main/client_table_blueprints/android_output"
YRS_URL="https://raw.githubusercontent.com/Zakarias-Viinikka/ZakkeN/main/yrs/android_output_folder"

JNI_DEST="app/src/main/jniLibs/arm64-v8a"
KOTLIN_DEST="app/src/main/java"

echo "Updating native libraries and bindings for Zndroid..."

# 1. Clear old versions to avoid conflicts
rm -rf "$KOTLIN_DEST/com/z_db/android_mascot"
rm -rf "$KOTLIN_DEST/rustlib/client_table_blueprints"
rm -rf "$KOTLIN_DEST/rustlib/my_yrs_lib"
rm -rf "$KOTLIN_DEST/uniffi/protocol"

# 2. Re-create folders
mkdir -p "$JNI_DEST"
mkdir -p "$KOTLIN_DEST/com/z_db/android_mascot"
mkdir -p "$KOTLIN_DEST/rustlib/client_table_blueprints"
mkdir -p "$KOTLIN_DEST/rustlib/my_yrs_lib"
mkdir -p "$KOTLIN_DEST/uniffi/protocol"

# 3. Download z_db (LiveForever)
echo "Fetching z_db..."
curl -L "$Z_DB_URL/jniLibs/arm64-v8a/libdb_wrapper.so" -o "$JNI_DEST/libdb_wrapper.so"
curl -L "$Z_DB_URL/jniLibs/arm64-v8a/libprotocol.so" -o "$JNI_DEST/libprotocol.so"
curl -L "$Z_DB_URL/kotlin/com/z_db/android_mascot/db_wrapper.kt" -o "$KOTLIN_DEST/com/z_db/android_mascot/db_wrapper.kt"

# 4. Download client_table_blueprints
echo "Fetching client_table_blueprints..."
curl -L "$BLUEPRINTS_URL/jniLibs/arm64-v8a/libclient_table_blueprints.so" -o "$JNI_DEST/libclient_table_blueprints.so"
curl -L "$BLUEPRINTS_URL/jniLibs/arm64-v8a/libmy_yrs_lib-bced72b5f489fa65.so" -o "$JNI_DEST/libmy_yrs_lib-bced72b5f489fa65.so"
curl -L "$BLUEPRINTS_URL/kotlin/rustlib/client_table_blueprints/client_table_blueprints.kt" -o "$KOTLIN_DEST/rustlib/client_table_blueprints/client_table_blueprints.kt"

# 5. Download shared protocol
echo "Fetching shared protocol..."
curl -L "$Z_DB_URL/kotlin/uniffi/protocol/protocol.kt" -o "$KOTLIN_DEST/uniffi/protocol/protocol.kt"

# 6. Download yrs
echo "Fetching yrs..."
curl -L "$YRS_URL/jniLibs/arm64-v8a/libmy_yrs_lib.so" -o "$JNI_DEST/libmy_yrs_lib.so"
curl -L "$YRS_URL/kotlin/rustlib/my_yrs_lib/my_yrs_lib.kt" -o "$KOTLIN_DEST/rustlib/my_yrs_lib/my_yrs_lib.kt"

echo "Done! Native infrastructure synchronized for Zndroid."
