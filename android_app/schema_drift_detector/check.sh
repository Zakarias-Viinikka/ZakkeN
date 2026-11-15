#!/usr/bin/env bash
set -e
cd "$(dirname "$0")"
if ! command -v cargo >/dev/null 2>&1; then
    if [ -f "$HOME/.cargo/env" ]; then
        # shellcheck disable=SC1091
        . "$HOME/.cargo/env"
    fi
fi
if ! command -v cargo >/dev/null 2>&1; then
    echo "schema_drift_detector: cargo not found on PATH and ~/.cargo/env is missing" >&2
    exit 2
fi
exec cargo run --quiet --manifest-path "$(pwd)/Cargo.toml" -- "$@"
