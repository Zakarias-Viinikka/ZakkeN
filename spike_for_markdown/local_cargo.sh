#!/usr/bin/env bash
set -e
ROOT="$(cd "$(dirname "$0")" && pwd)"

if [ "$1" = "run" ]; then
    shift
    cd "$ROOT"
    exec trunk serve "$@"
fi

if [ "$1" = "check" ]; then
    shift
    cd "$ROOT"
    exec cargo check --target wasm32-unknown-unknown "$@"
fi

exec cargo "$@"
