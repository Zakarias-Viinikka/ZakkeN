#!/usr/bin/env bash
# Runs cargo from the workspace root. Two subcommands deviate from plain
# cargo:
# - `run`: cd into example_app and run trunk serve so trunk finds its
#   index.html and Cargo.toml.
# - `test` (and cargo's `t` alias): force the native host target. The
#   global cargo config in this environment defaults to wasm32, which
#   makes cargo produce a .wasm test binary and the test runner dies
#   with "Exec format error". A --target override on this invocation
#   wins over the config default.
# Everything else goes straight to cargo.

case "$1" in
    run)
        cd "$(dirname "$0")/example_app" || exit 1
        shift
        exec trunk serve "$@"
        ;;
    test|t)
        HOST=$(rustc -vV | sed -n 's/^host: //p')
        shift
        # Buffer each "Running …" block and drop the whole thing when
        # its "test result:" line reports 0 passed, so the empty
        # unit/integration/doc-test binaries don't spam the output.
        cargo test --target "$HOST" "$@" 2>&1 | awk '
            function flush_or_drop() {
                if (buf !~ / 0 passed;/) printf "%s", buf
                buf = ""
            }
            /^ *(Running|Doc-tests) / { buf = $0 "\n"; state = 1; last_was_result = 0; next }
            state == 1 {
                if ($0 == "" && last_was_result) {
                    flush_or_drop()
                    state = 0
                    last_was_result = 0
                    next
                }
                buf = buf $0 "\n"
                if (/^test result: /) last_was_result = 1
                next
            }
            { print }
        '
        exit ${PIPESTATUS[0]}
        ;;
esac
exec cargo "$@"