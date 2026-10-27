#!/usr/bin/env bash
export RUSTFLAGS="-C link-arg=-fuse-ld=mold"

if [ "$1" = "run" ] && [ $# -eq 1 ]; then
    exec cargo run --bin zakken_server
fi

if [ "$1" = "check" ]; then
    shift
    exec cargo check --workspace --all-targets "$@"
fi

exec cargo "$@"