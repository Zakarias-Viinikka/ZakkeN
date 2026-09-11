#!/bin/bash

RED='\033[0;31m'
NC='\033[0m'

CHANGED=0
declare -A OLD_HASHES
FILES_TO_CHECK=()

check_404() {
    local file="$1"
    if grep -q "404: Not Found" "$file" 2>/dev/null; then
        echo -e "${RED}Error: $file appears to be a 404 Not Found page${NC}"
        return 1
    fi
    return 0
}

capture_hash() {
    local file="$1"
    FILES_TO_CHECK+=("$file")
    if [ -f "$file" ]; then
        OLD_HASHES["$file"]=$(md5sum "$file" | awk '{print $1}')
    else
        OLD_HASHES["$file"]=""
    fi
}

download_file() {
    local url="$1"
    local dest="$2"
    curl -L "$url" -o "$dest" || exit 1
    check_404 "$dest" || exit 1
}

# URLs
Z_DB_URL="https://raw.githubusercontent.com/Zakarias-Viinikka/z_db/main/db_wrapper/android_output"
BLUEPRINTS_URL="https://raw.githubusercontent.com/Zakarias-Viinikka/ZakkeN/main/client_table_blueprints/android_output"
YRS_URL="https://raw.githubusercontent.com/Zakarias-Viinikka/ZakkeN/main/yrs/android_output_folder"
LOVE_LETTER_URL="https://raw.githubusercontent.com/Zakarias-Viinikka/ZakkeN/main/love_letter/android_output_folder"
TEXT_DIFF_URL="https://raw.githubusercontent.com/Zakarias-Viinikka/ZakkeN/main/text_diff/android_output_folder"

JNI_DEST="app/src/main/jniLibs/arm64-v8a"
KOTLIN_DEST="app/src/main/java"

echo "Updating native libraries and bindings for Zndroid..."

# List all files that will be downloaded
capture_hash "$JNI_DEST/libdb_wrapper.so"
capture_hash "$JNI_DEST/libprotocol.so"
capture_hash "$KOTLIN_DEST/com/z_db/android_mascot/db_wrapper.kt"
capture_hash "$JNI_DEST/libclient_table_blueprints.so"
capture_hash "$JNI_DEST/libmy_yrs_lib-bced72b5f489fa65.so"
capture_hash "$KOTLIN_DEST/rustlib/client_table_blueprints/client_table_blueprints.kt"
capture_hash "$KOTLIN_DEST/uniffi/protocol/protocol.kt"
capture_hash "$JNI_DEST/libmy_yrs_lib.so"
capture_hash "$KOTLIN_DEST/rustlib/my_yrs_lib/my_yrs_lib.kt"
capture_hash "$JNI_DEST/liblove_letter.so"
capture_hash "$KOTLIN_DEST/rustlib/love_letter/love_letter.kt"
capture_hash "$JNI_DEST/libtext_diff.so"
capture_hash "$KOTLIN_DEST/rustlib/text_diff/text_diff.kt"

# Clear old versions
rm -rf "$KOTLIN_DEST/com/z_db/android_mascot"
rm -rf "$KOTLIN_DEST/rustlib/client_table_blueprints"
rm -rf "$KOTLIN_DEST/rustlib/my_yrs_lib"
rm -rf "$KOTLIN_DEST/uniffi/protocol"
rm -rf "$KOTLIN_DEST/rustlib/love_letter"
rm -rf "$KOTLIN_DEST/rustlib/text_diff"

mkdir -p "$JNI_DEST"
mkdir -p "$KOTLIN_DEST/com/z_db/android_mascot"
mkdir -p "$KOTLIN_DEST/rustlib/client_table_blueprints"
mkdir -p "$KOTLIN_DEST/rustlib/my_yrs_lib"
mkdir -p "$KOTLIN_DEST/uniffi/protocol"
mkdir -p "$KOTLIN_DEST/rustlib/love_letter"
mkdir -p "$KOTLIN_DEST/rustlib/text_diff"

# Download z_db
echo "Fetching z_db..."
download_file "$Z_DB_URL/jniLibs/arm64-v8a/libdb_wrapper.so" "$JNI_DEST/libdb_wrapper.so"
download_file "$Z_DB_URL/jniLibs/arm64-v8a/libprotocol.so" "$JNI_DEST/libprotocol.so"
download_file "$Z_DB_URL/kotlin/com/z_db/android_mascot/db_wrapper.kt" "$KOTLIN_DEST/com/z_db/android_mascot/db_wrapper.kt"

# Download client_table_blueprints
echo "Fetching client_table_blueprints..."
download_file "$BLUEPRINTS_URL/jniLibs/arm64-v8a/libclient_table_blueprints.so" "$JNI_DEST/libclient_table_blueprints.so"
download_file "$BLUEPRINTS_URL/jniLibs/arm64-v8a/libmy_yrs_lib-bced72b5f489fa65.so" "$JNI_DEST/libmy_yrs_lib-bced72b5f489fa65.so"
download_file "$BLUEPRINTS_URL/kotlin/rustlib/client_table_blueprints/client_table_blueprints.kt" "$KOTLIN_DEST/rustlib/client_table_blueprints/client_table_blueprints.kt"

# Patch package mismatch for my_yrs_lib
sed -i 's/uniffi\.my_yrs_lib/rustlib.my_yrs_lib/g' "$KOTLIN_DEST/rustlib/client_table_blueprints/client_table_blueprints.kt"

# Download shared protocol
echo "Fetching shared protocol..."
download_file "$Z_DB_URL/kotlin/uniffi/protocol/protocol.kt" "$KOTLIN_DEST/uniffi/protocol/protocol.kt"

# Download yrs
echo "Fetching yrs..."
download_file "$YRS_URL/jniLibs/arm64-v8a/libmy_yrs_lib.so" "$JNI_DEST/libmy_yrs_lib.so"
download_file "$YRS_URL/kotlin/rustlib/my_yrs_lib/my_yrs_lib.kt" "$KOTLIN_DEST/rustlib/my_yrs_lib/my_yrs_lib.kt"

# Download love_letter
echo "Fetching love_letter..."
download_file "$LOVE_LETTER_URL/jniLibs/arm64-v8a/liblove_letter.so" "$JNI_DEST/liblove_letter.so"
download_file "$LOVE_LETTER_URL/kotlin/rustlib/love_letter/love_letter.kt" "$KOTLIN_DEST/rustlib/love_letter/love_letter.kt"

# Patch package mismatch for love_letter
sed -i 's/uniffi\.my_yrs_lib/rustlib.my_yrs_lib/g' "$KOTLIN_DEST/rustlib/love_letter/love_letter.kt"

# Download text_diff
echo "Fetching text_diff..."
download_file "$TEXT_DIFF_URL/jniLibs/arm64-v8a/libtext_diff.so" "$JNI_DEST/libtext_diff.so"
download_file "$TEXT_DIFF_URL/kotlin/rustlib/text_diff/text_diff.kt" "$KOTLIN_DEST/rustlib/text_diff/text_diff.kt"

# Patch package mismatch for text_diff
sed -i 's/uniffi\.text_diff/rustlib.text_diff/g' "$KOTLIN_DEST/rustlib/text_diff/text_diff.kt"
sed -i 's/uniffi\.my_yrs_lib/rustlib.my_yrs_lib/g' "$KOTLIN_DEST/rustlib/text_diff/text_diff.kt"

# Compare hashes after all modifications
for file in "${FILES_TO_CHECK[@]}"; do
    if [ -f "$file" ]; then
        new_hash=$(md5sum "$file" | awk '{print $1}')
    else
        new_hash=""
    fi
    old_hash="${OLD_HASHES[$file]}"
    if [ "$old_hash" != "$new_hash" ]; then
        CHANGED=1
        break
    fi
done

echo "Done! Native infrastructure synchronized for Zndroid."

if [ $CHANGED -eq 1 ]; then
    echo "a change was made"
else
    echo "nothing changed"
fi
