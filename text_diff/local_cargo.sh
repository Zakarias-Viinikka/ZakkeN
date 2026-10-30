#!/usr/bin/env bash
set -e

cd "$(dirname "$0")" || exit 1

# This crate's .cargo/config.toml defaults the build target to
# wasm32-unknown-unknown, because the web client consumes it that way.
# But test and run need the host target — a wasm binary can't execute
# on this machine. Both get the target forced here.
# `c t` and `c r` arrive as `t` and `r`, and the `nt` alias is already
# this exact thing; matching every form is cheaper than arguing.
case "$1" in
    t|test)
        shift
        exec cargo test --target x86_64-unknown-linux-gnu "$@"
        ;;
    r|run)
        shift
        exec cargo run --target x86_64-unknown-linux-gnu "$@"
        ;;
esac

exec cargo "$@"