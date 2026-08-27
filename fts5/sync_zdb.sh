#!/bin/sh
set -e

ZDB_PATH="${ZDB_PATH:-$HOME/ProgStuff/z_db}"

SUBPATH="db_wrapper/src/web_output"
DEST="z_db"
HASH_FILE=".last_web_output_hash"

# Calculate a hash of all file contents in the source directory (recursive)
# Ignores metadata, only content matters.
LATEST=$(find "$ZDB_PATH/$SUBPATH" -type f -print0 \
    | sort -z \
    | xargs -0 sha256sum \
    | sha256sum \
    | cut -d' ' -f1)

OLD=$(cat "$HASH_FILE" 2>/dev/null || echo "")

if [ "$LATEST" = "$OLD" ]; then
    echo "web_output unchanged, skipping copy"
else
    cp -r "$ZDB_PATH/$SUBPATH/." "$DEST"
    echo "$LATEST" > "$HASH_FILE"
    echo "web_output updated"
    # Uncomment the next line if you still want to update the protocol dependency
    # cargo update -p protocol
fi
