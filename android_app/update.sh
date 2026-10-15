#!/bin/bash
set -e
cd "$(dirname "$0")"

RED='\033[0;31m'
NC='\033[0m'

ZAKKEN=/home/zakke/ProgStuff/ZakkeN
Z_DB=/home/zakke/ProgStuff/z_db

JNI_DEST="app/src/main/jniLibs/arm64-v8a"
KOTLIN_DEST="app/src/main/java"

CHANGED=0
declare -A OLD_HASHES
FILES_TO_CHECK=()

capture_hash() {
    local file="$1"
    FILES_TO_CHECK+=("$file")
    if [ -f "$file" ]; then
        OLD_HASHES["$file"]=$(md5sum "$file" | awk '{print $1}')
    else
        OLD_HASHES["$file"]=""
    fi
}

copy_file() {
    local src="$1"
    local dest="$2"
    if [ ! -f "$src" ]; then
        echo -e "${RED}Error: source file does not exist: $src${NC}"
        echo -e "${RED}Did the corresponding build_android.sh run?${NC}"
        exit 1
    fi
    cp "$src" "$dest"
}

echo "Updating native libraries and bindings from local builds..."

# -- List all destinations for change reporting --
capture_hash "$JNI_DEST/libdb_wrapper.so"
capture_hash "$JNI_DEST/libprotocol.so"
capture_hash "$KOTLIN_DEST/com/z_db/android_mascot/db_wrapper.kt"
capture_hash "$JNI_DEST/libclient_table_blueprints.so"
capture_hash "$KOTLIN_DEST/rustlib/client_table_blueprints/client_table_blueprints.kt"
capture_hash "$KOTLIN_DEST/uniffi/protocol/protocol.kt"
capture_hash "$JNI_DEST/libmy_yrs_lib.so"
capture_hash "$KOTLIN_DEST/rustlib/my_yrs_lib/my_yrs_lib.kt"
capture_hash "$JNI_DEST/liblove_letter.so"
capture_hash "$KOTLIN_DEST/rustlib/love_letter/love_letter.kt"
capture_hash "$JNI_DEST/libtext_diff.so"
capture_hash "$KOTLIN_DEST/rustlib/text_diff/text_diff.kt"
capture_hash "$JNI_DEST/libdata_builder_for_operations_that_need_to_be_correct.so"
capture_hash "$KOTLIN_DEST/rustlib/data_builder_for_operations_that_need_to_be_correct/data_builder_for_operations_that_need_to_be_correct.kt"

# -- Clear old versions --
rm -rf "$KOTLIN_DEST/com/z_db/android_mascot"
rm -rf "$KOTLIN_DEST/rustlib/client_table_blueprints"
rm -rf "$KOTLIN_DEST/rustlib/my_yrs_lib"
rm -rf "$KOTLIN_DEST/uniffi/protocol"
rm -rf "$KOTLIN_DEST/rustlib/love_letter"
rm -rf "$KOTLIN_DEST/rustlib/text_diff"
rm -rf "$KOTLIN_DEST/rustlib/data_builder_for_operations_that_need_to_be_correct"

# -- Hashed my_yrs_lib deps in jniLibs: remove old ones, we'll re-copy below --
rm -f "$JNI_DEST"/libmy_yrs_lib-*.so

# -- Recreate destination folders --
mkdir -p "$JNI_DEST"
mkdir -p "$KOTLIN_DEST/com/z_db/android_mascot"
mkdir -p "$KOTLIN_DEST/rustlib/client_table_blueprints"
mkdir -p "$KOTLIN_DEST/rustlib/my_yrs_lib"
mkdir -p "$KOTLIN_DEST/uniffi/protocol"
mkdir -p "$KOTLIN_DEST/rustlib/love_letter"
mkdir -p "$KOTLIN_DEST/rustlib/text_diff"
mkdir -p "$KOTLIN_DEST/rustlib/data_builder_for_operations_that_need_to_be_correct"

# -- z_db --
echo "Copying z_db..."
copy_file "$Z_DB/db_wrapper/android_output/jniLibs/arm64-v8a/libdb_wrapper.so" \
          "$JNI_DEST/libdb_wrapper.so"
copy_file "$Z_DB/protocol/output_for_android/jniLibs/arm64-v8a/libprotocol.so" \
          "$JNI_DEST/libprotocol.so"
copy_file "$Z_DB/db_wrapper/android_output/kotlin/com/z_db/android_mascot/db_wrapper.kt" \
          "$KOTLIN_DEST/com/z_db/android_mascot/db_wrapper.kt"
copy_file "$Z_DB/db_wrapper/android_output/kotlin/uniffi/protocol/protocol.kt" \
          "$KOTLIN_DEST/uniffi/protocol/protocol.kt"

# -- client_table_blueprints --
echo "Copying client_table_blueprints..."
copy_file "$ZAKKEN/client_table_blueprints/android_output/jniLibs/arm64-v8a/libclient_table_blueprints.so" \
          "$JNI_DEST/libclient_table_blueprints.so"
copy_file "$ZAKKEN/client_table_blueprints/android_output/kotlin/rustlib/client_table_blueprints/client_table_blueprints.kt" \
          "$KOTLIN_DEST/rustlib/client_table_blueprints/client_table_blueprints.kt"

# copy whatever hashed my_yrs_lib came along with blueprints
for f in "$ZAKKEN"/client_table_blueprints/android_output/jniLibs/arm64-v8a/libmy_yrs_lib-*.so; do
    [ -f "$f" ] && cp "$f" "$JNI_DEST/"
done

# patch package mismatch
sed -i 's/uniffi\.my_yrs_lib/rustlib.my_yrs_lib/g' \
    "$KOTLIN_DEST/rustlib/client_table_blueprints/client_table_blueprints.kt"

# -- yrs --
echo "Copying yrs..."
copy_file "$ZAKKEN/yrs/android_output_folder/jniLibs/arm64-v8a/libmy_yrs_lib.so" \
          "$JNI_DEST/libmy_yrs_lib.so"
copy_file "$ZAKKEN/yrs/android_output_folder/kotlin/rustlib/my_yrs_lib/my_yrs_lib.kt" \
          "$KOTLIN_DEST/rustlib/my_yrs_lib/my_yrs_lib.kt"

# -- love_letter --
echo "Copying love_letter..."
copy_file "$ZAKKEN/love_letter/android_output_folder/jniLibs/arm64-v8a/liblove_letter.so" \
          "$JNI_DEST/liblove_letter.so"
copy_file "$ZAKKEN/love_letter/android_output_folder/kotlin/rustlib/love_letter/love_letter.kt" \
          "$KOTLIN_DEST/rustlib/love_letter/love_letter.kt"

sed -i 's/uniffi\.my_yrs_lib/rustlib.my_yrs_lib/g' \
    "$KOTLIN_DEST/rustlib/love_letter/love_letter.kt"

# -- text_diff --
echo "Copying text_diff..."
copy_file "$ZAKKEN/text_diff/android_output_folder/jniLibs/arm64-v8a/libtext_diff.so" \
          "$JNI_DEST/libtext_diff.so"
copy_file "$ZAKKEN/text_diff/android_output_folder/kotlin/rustlib/text_diff/text_diff.kt" \
          "$KOTLIN_DEST/rustlib/text_diff/text_diff.kt"

sed -i 's/uniffi\.text_diff/rustlib.text_diff/g' \
    "$KOTLIN_DEST/rustlib/text_diff/text_diff.kt"
sed -i 's/uniffi\.my_yrs_lib/rustlib.my_yrs_lib/g' \
    "$KOTLIN_DEST/rustlib/text_diff/text_diff.kt"

# -- data_builder --
echo "Copying data_builder..."
copy_file "$ZAKKEN/cqrs_state_synchronization/data_builder_for_operations_that_need_to_be_correct/android_output/jniLibs/arm64-v8a/libdata_builder_for_operations_that_need_to_be_correct.so" \
          "$JNI_DEST/libdata_builder_for_operations_that_need_to_be_correct.so"
copy_file "$ZAKKEN/cqrs_state_synchronization/data_builder_for_operations_that_need_to_be_correct/android_output/kotlin/rustlib/data_builder_for_operations_that_need_to_be_correct/data_builder_for_operations_that_need_to_be_correct.kt" \
          "$KOTLIN_DEST/rustlib/data_builder_for_operations_that_need_to_be_correct/data_builder_for_operations_that_need_to_be_correct.kt"

# the builder's Kotlin may reference the same package names as the others
sed -i 's/uniffi\.my_yrs_lib/rustlib.my_yrs_lib/g' \
    "$KOTLIN_DEST/rustlib/data_builder_for_operations_that_need_to_be_correct/data_builder_for_operations_that_need_to_be_correct.kt"
sed -i 's/uniffi\.text_diff/rustlib.text_diff/g' \
    "$KOTLIN_DEST/rustlib/data_builder_for_operations_that_need_to_be_correct/data_builder_for_operations_that_need_to_be_correct.kt"

# -- Change detection --
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