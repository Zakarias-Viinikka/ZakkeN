#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

if [ "$#" -eq 0 ]; then
    exec ./gradlew :app:assembleDebug
else
    exec ./gradlew "$@"
fi
