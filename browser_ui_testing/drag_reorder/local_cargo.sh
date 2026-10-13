#!/usr/bin/env bash
if [ "$1" = "run" ]; then
    cd "$(dirname "$0")/final_test" || exit 1
    exec ./local_cargo.sh "$@"
fi
exec cargo "$@"