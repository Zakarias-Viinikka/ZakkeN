#!/usr/bin/env bash
export RUSTFLAGS="-C link-arg=-fuse-ld=mold"

if [ "$1" = "run" ] && [ $# -eq 1 ]; then
    exec cargo run --bin cli_gui
fi

exec cargo "$@"
