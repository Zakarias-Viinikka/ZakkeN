#!/usr/bin/env bash
set -e

HERE="$(cd "$(dirname "$0")" && pwd)"
HOST="$(rustc -vV | awk '/^host:/ {print $2}')"

if [ "$1" = "run" ] || [ "$1" = "r" ]; then
    if [ ! -d "$HERE/.venv" ]; then
        echo "[local_cargo] first run, creating .venv"
        python3 -m venv "$HERE/.venv"
        "$HERE/.venv/bin/pip" install --quiet -r "$HERE/requirements.txt"
    fi
    export PATH="$HERE/.venv/bin:$PATH"
    if ! "$HERE/.venv/bin/playwright" install --dry-run chromium > /dev/null 2>&1; then
        echo "[local_cargo] installing chromium"
        "$HERE/.venv/bin/playwright" install chromium
    fi
    shift
    cd "$HERE/orchestrator"
    exec cargo run --target "$HOST" -- "$@"
fi

cd "$HERE/orchestrator"
exec cargo --target "$HOST" "$@"