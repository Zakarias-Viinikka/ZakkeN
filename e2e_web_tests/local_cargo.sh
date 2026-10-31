#!/usr/bin/env bash
set -e

HERE="$(cd "$(dirname "$0")" && pwd)"
MANIFEST="$HERE/orchestrator/Cargo.toml"

if [ "$1" = "run" ]; then
    shift
    if [ "$1" = "r" ] || [ "$1" = "release" ]; then
        shift
        exec cargo run --release --manifest-path "$MANIFEST" "$@"
    fi
    exec cargo run --manifest-path "$MANIFEST" "$@"
fi

exec cargo --manifest-path "$MANIFEST" "$@"
