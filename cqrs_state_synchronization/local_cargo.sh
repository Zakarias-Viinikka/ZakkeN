#!/usr/bin/env bash
set -e

ROOT="$(cd "$(dirname "$0")" && pwd)"

if [ "$1" = "run" ]; then
    shift
    cd "$ROOT/web_interface"
    exec trunk serve "$@"
fi

exec cargo "$@"
