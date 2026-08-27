#!/bin/sh
set -e

REPO="Zakarias-Viinikka/z_db"
SUBPATH="db_wrapper/src/web_output"
DEST="z_db"
HASH_FILE=".last_web_output_hash"

LATEST=$(curl -s "https://api.github.com/repos/$REPO/commits?path=$SUBPATH&per_page=1" | jq -r '.[0].sha')

OLD=$(cat "$HASH_FILE" 2>/dev/null || echo "")

if [ "$LATEST" = "$OLD" ]; then
    echo "web_output unchanged, skipping copy"
else
    TMP_DIR=$(mktemp -d)
    git clone --depth 1 --filter=blob:none --sparse "https://github.com/$REPO.git" "$TMP_DIR" > /dev/null 2>&1
    cd "$TMP_DIR"
    git sparse-checkout set "$SUBPATH"
    cd - > /dev/null
    cp -r "$TMP_DIR/$SUBPATH/." "$DEST"
    echo "$LATEST" > "$HASH_FILE"
    rm -rf "$TMP_DIR"
    echo "web_output updated"
fi
