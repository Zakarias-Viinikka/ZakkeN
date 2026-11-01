#!/usr/bin/env bash
set -euo pipefail
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
export PATH="$(dirname "$HERE")/.venv/bin:$PATH"
exec python -m pytest "$@"
