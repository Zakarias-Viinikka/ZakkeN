#!/usr/bin/env bash
# Mirrors web_diff_viewer/local_cargo.sh. `t`/`test` forces the native
# host target because the global cargo config defaults to wasm32.
# Everything else goes straight to cargo.

export RUSTFLAGS="-C link-arg=-fuse-ld=mold"
cd "$(dirname "$0")" || exit 1

case "$1" in
    t|test)
        HOST=$(rustc -vV | sed -n 's/^host: //p')
        shift
        exec cargo test --workspace --target "$HOST" "$@"
        ;;
esac
exec cargo "$@"
