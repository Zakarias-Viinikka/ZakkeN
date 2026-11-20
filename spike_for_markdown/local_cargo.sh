#!/usr/bin/env bash
set -e

ROOT="$(cd "$(dirname "$0")" && pwd)"

if [ "$1" = "run" ]; then
    shift
    cd "$ROOT"
    exec trunk serve "$@"
fi

if { [ "$1" = "t" ] || [ "$1" = "test" ]; } && [ "$PWD" = "$ROOT" ]; then
    shift
    npm --prefix "$ROOT/js" run build
    cargo test "$@"
    node "$ROOT/js/test_links.mjs"
    exec cargo check --target wasm32-unknown-unknown
fi

if [ "$1" = "check" ] && [ "$PWD" = "$ROOT" ]; then
    has_target=0
    for arg in "$@"; do
        case "$arg" in
            --target|--target=*) has_target=1 ;;
        esac
    done
    if [ "$has_target" = 0 ]; then
        npm --prefix "$ROOT/js" run build
        cargo "$@"
        shift
        exec cargo check --target wasm32-unknown-unknown "$@"
    fi
fi

exec cargo "$@"