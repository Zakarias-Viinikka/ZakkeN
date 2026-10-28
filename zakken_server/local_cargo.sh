#!/usr/bin/env bash
export RUSTFLAGS="-C link-arg=-fuse-ld=mold"
cd "$(dirname "$0")" || exit 1

if [ "$1" = "run" ] && [ $# -eq 1 ]; then
    exec cargo run --bin zakken_server
fi

if [ "$1" = "check" ]; then
    shift
    cargo check --workspace --all-targets "$@" || exit
    exit 0
fi

exec cargo "$@"