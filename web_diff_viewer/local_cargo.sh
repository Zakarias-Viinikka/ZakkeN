#!/usr/bin/env bash
# Runs cargo from the workspace root. Two subcommands deviate from plain
# cargo:
# - `run`: cd into example_app and run trunk serve so trunk finds its
#   index.html and Cargo.toml.
# - `test` (and cargo's `t` alias): force the native host target. The
#   global cargo config in this environment defaults to wasm32, which
#   makes cargo produce a .wasm test binary and the test runner dies
#   with "Exec format error". A --target override on this invocation
#   wins over the config default.
# Everything else goes straight to cargo.

case "$1" in
    run)
        cd "$(dirname "$0")/example_app" || exit 1
        shift
        exec trunk serve "$@"
        ;;
    test|t)
        HOST=$(rustc -vV | sed -n 's/^host: //p')
        shift
        exec cargo test --target "$HOST" "$@"
        ;;
esac
exec cargo "$@"