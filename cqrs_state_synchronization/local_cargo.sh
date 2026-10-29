#!/usr/bin/env bash
set -e

ROOT="$(cd "$(dirname "$0")" && pwd)"

if [ "$1" = "run" ]; then
    shift
    cd "$ROOT/web_interface"
    exec trunk serve "$@"
fi

# web_interface only builds for wasm32 (its own .cargo/config.toml sets the
# target when cargo runs from inside its directory). If we're at the workspace
# root and the user didn't pick a scope, exclude web_interface so the host
# build only touches the native crates.
if [ "$PWD" = "$ROOT" ]; then
    case "$1" in
        check|build|test|clippy)
            has_scope=0
            for arg in "$@"; do
                case "$arg" in
                    -p|--package|--workspace|-w|--exclude) has_scope=1 ;;
                esac
            done
            if [ "$has_scope" = 0 ]; then
                exec cargo "$@" --workspace --exclude web_interface
            fi
            ;;
    esac
fi

exec cargo "$@"