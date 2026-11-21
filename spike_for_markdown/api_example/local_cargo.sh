#!/usr/bin/env bash
if [ "$1" = "run" ]; then
    exec trunk serve "${@:2}"
fi
exec cargo "$@"
