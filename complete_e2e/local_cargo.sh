#!/usr/bin/env bash
set -e

HERE="$(cd "$(dirname "$0")" && pwd)"
if [ -d "$HERE/.venv/bin" ]; then
    export PATH="$HERE/.venv/bin:$PATH"
fi

if [ "$1" = "run" ] || [ "$1" = "r" ]; then
    shift
    if [ "$1" = "r" ] || [ "$1" = "release" ] || [ "$1" = "--release" ]; then
        case "$1" in
            r|release) shift ;;
            --release) shift ;;
        esac
        exec cargo run --release -p orchestrator "$@"
    fi
    exec cargo run -p orchestrator "$@"
fi

exec cargo "$@"