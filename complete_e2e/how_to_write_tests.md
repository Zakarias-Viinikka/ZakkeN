# How to write tests

## No unnecessary waiting

Do not pad timeouts "just in case". Do not multiply timeouts by a
safety factor. If a wait is only there because someone was unsure, cut
it. The test should not sit idle when it could be checking.

## Verify what's on disk before reasoning about it

Before debugging a problem, check what the code actually says. `rg` for
the symbol, the file, the string. If the thing you believe exists
doesn't exist, the answer is "it wasn't written", not "figure out why
it's broken". Reasoning about a file you haven't read burns turns.

Same for test failures: if a test asserts X, `rg` where X is set. If
the setter doesn't exist in `src/`, that's the bug. If it exists in
`src/` but not in `dist/`, the build is stale. Don't guess between
those — check.

## Expose a read method before inferring

When a test needs to know "what state is the app in", add a window
event that returns that state as JSON. Do not infer it from UI text,
DOM shape, or the sequence of actions you took.

The pattern already exists:
- `read_titles` → the local titles
- `read_client_state` → url, user_id, server_name, drift_version, port, page_count
- `read_client_db` style events per table, as needed

A debug read is ten lines. A guess that sends the investigation down
the wrong path costs ten turns.

## Timeouts

Every wait has a single number behind it: `DEFAULT_SECS` in
`support/timeouts.py`. Call sites use `ms()` (for Playwright's
`timeout=`) or `secs()` (for Python-side `timeout_secs=`), never a
literal. If a specific wait legitimately needs longer, define a named
constant next to that call site with a one-line comment saying why —
do not silently bump the default.

The default is 2 seconds because that is enough for a healthy app on
localhost to reach most states. When a test fails on a timeout, first
check whether the app is actually slow there or the test is waiting
for the wrong signal. Bumping the default is almost never the right
answer.
