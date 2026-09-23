#!/usr/bin/env bash
set -e

SRC="$HOME/ProgStuff/z_db/db_wrapper/src/web_output"
DST="$HOME/ProgStuff/ZakkeN/cqrs_state_synchronization/web_interface/zdb_web_output"

[ -d "$SRC" ] || { echo "missing source: $SRC"; exit 1; }

mkdir -p "$DST"
rm -rf "$DST"/*
cp -r "$SRC"/. "$DST"/

echo "copied $SRC -> $DST"
ls -la "$DST"
