#!/usr/bin/env bash
export RUSTFLAGS="-C link-arg=-fuse-ld=mold"

if [ "$1" = "run" ] && [ $# -eq 1 ]; then
    exec cargo run --bin zakken_server
fi

if [ "$1" = "check" ]; then
    shift
    cargo check --workspace --all-targets "$@" || exit
    ( cd client_internals/web && cargo check --all-targets --target wasm32-unknown-unknown "$@" ) || exit
    ( cd client_internals/android && cargo check --all-targets "$@" ) || exit
    exit 0
fi

exec cargo "$@"