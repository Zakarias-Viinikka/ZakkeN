#!/usr/bin/env bash
set -e

SRC="$HOME/ProgStuff/z_db/db_wrapper/src/web_output"
DST="$HOME/ProgStuff/ZakkeN/browser_ui_testing/drag_reorder/zdb_web_output"

[ -d "$SRC" ] || { echo "missing source: $SRC"; exit 1; }

mkdir -p "$DST"
rm -rf "$DST"/*
cp -r "$SRC"/. "$DST"/

python3 - "$DST/worker_wrapper.js" <<'PY'
import sys

path = sys.argv[1]
old = "'leptos_db'"
new = "'drag_reorder_test'"

with open(path) as f:
    content = f.read()

if old not in content:
    sys.exit(f"assert failed: {old!r} not found in {path}")

with open(path, "w") as f:
    f.write(content.replace(old, new, 1))

print(f"renamed {old} -> {new} in {path}")
PY

echo "copied $SRC -> $DST"
