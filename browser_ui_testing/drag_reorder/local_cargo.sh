#!/usr/bin/env bash
if [ "$1" = "run" ]; then
    cd "$(dirname "$0")/rapier_tests_combined" || exit 1
    exec ./local_cargo.sh "$@"
fi
exec cargo "$@"