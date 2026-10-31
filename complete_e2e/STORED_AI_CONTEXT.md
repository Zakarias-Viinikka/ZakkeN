# e2e_template_for_web_with_server — stored AI context

Written for a fresh AI picking this up. If code and this file disagree, the code wins.

## What this is — 2026-10-31

A minimal Leptos + axum + pytest setup for automated e2e testing. Lives in `rust_practice/`, separate from any ZakkeN crate. Its purpose is to be a reference for the shape of an e2e harness — three layers, small enough to hold in the head — without the size of the real thing.

Three layers, stacked. Each layer only talks to the one below it.

- **orchestrator** (Rust binary). Builds, spawns, serves, runs the tester, tears down. Never opens a browser.
- **tester** (Python + pytest + Playwright). Opens a real Chromium browser, drives the web app, asserts. Does not start or stop anything.
- **web_app** (Leptos CSR) and **server** (axum). The things being tested. The web app connects to the server over a websocket.

The orchestrator never opens a browser; the tester never spawns a process. Both are deliberate.

## Run

    source .venv/bin/activate
    c r

`c r` is `local_cargo.sh` at the workspace root; it runs `cargo run -p orchestrator`. Install instructions in `requirements.md`.

## The web app

`web_app/src/app_main.rs` — the UI. One button ("Send hello") talks to the server over a websocket. Reply lands in `data-testid="reply"`. Connection state in `data-testid="connected"`. Tester-set message in `data-testid="tester-message"`.

`web_app/src/server_talker.rs` — the only file that touches `web_sys::WebSocket`. Owns the connection, the last reply, and the connected signal.

`web_app/src/methods_called_by_tester.rs` — macros that listen for `e2e_*` window events and write to Leptos signals. Same pattern as `javascript_take_the_wheel!` in the leptos_starter examples. Two macros (`tester_can_set_message!`, `tester_can_clear_message!`), invoked inline in the component body, torn down via `on_cleanup`. No `install()` wrapper — that was tried and removed; the user's preferred shape is inline setup in the component body.

From the tester:

    page.evaluate("window.dispatchEvent(new CustomEvent('e2e_set_message', { detail: 'hi' }))")

Same call shape as typing in the browser console.

The server port comes from the URL query string (`?server_port=N`). The orchestrator builds that URL, so the web app connects to the right server even though the port is picked at runtime.

## The server

`server/src/main.rs` — reads `--port` from args (default 3000), builds the router, serves.

`server/src/main_for_replying_to_websocket_messages.rs` — one websocket, receives a text message, replies with a fixed string, prints what it received.

No db, no protocol crate, no queue. The point is to see the wire work.

## The orchestrator

`orchestrator/src/main.rs` — steps in order:

1. `main_for_building_the_server_binary` — `cargo build -p server`. Fast when up-to-date.
2. `main_for_building_the_web_app_build` — `trunk build` only if `web_app/dist/index.html` is older than any tracked input (`src/`, `css/`, `index.html`, `Cargo.toml`, `Trunk.toml`, `rust-toolchain.toml`). Trunk re-runs wasm-bindgen every call, so the freshness check saves seconds.
3. `main_for_finding_a_free_port` — binds `127.0.0.1:0`, reads the port, drops.
4. `main_for_starting_the_server_process` — spawns the server, waits for TCP to answer.
5. `main_for_serving_the_web_app_build` — axum serving `web_app/dist` with SPA fallback to `index.html`.
6. `main_for_running_the_tester` — `python3 -m pytest -v` in `tester/`, `WEB_APP_URL` in the env. Waits for exit code.

Then kills the server, aborts the static serve, exits with pytest's code.

## The tester

**One window per session, not one per test.** The `page` fixture is session-scoped so a run opens one browser window and every test runs in it. Per-test `page` fixtures open a fresh window for each test, which reads as "opens, closes, opens again."

**The venv is prepended by `local_cargo.sh`, not activated.** `local_cargo.sh` does `export PATH="$HERE/.venv/bin:$PATH"` so `python3` resolves to `complete_e2e/.venv/bin/python3`. Do not `source .venv/bin/activate` in this folder. A copied venv still carries the original path in its `activate` script, so activating the copy points `python3` at the *source* venv (in this case `rust_practice/e2e_template_for_web_with_server/.venv/`). Symptoms: pytest runs fine, but headed tests flake with `TargetClosedError` partway through a test because two Python interpreters (test process and the pytest process inside `c r`) are talking to the same browser through different Playwright installs. Headless passing and headed failing is one of its shapes. The fix is to not activate; `c r` handles it.

`tester/conftest.py` fixtures:

- `browser` — Chromium via Playwright, maximized. Headless if `HEADLESS=1`.
- `page` — fresh context, `no_viewport=True` so the browser's own window size wins.
- `web_app_url` — from `WEB_APP_URL` env var, set by the orchestrator.
- `pause` — sleeps between test steps so a person can watch. No-op when `HEADLESS=1`.

`tester/test_web_app_connects_to_server.py` — two tests:

- `test_web_app_shows_connected` — page loads, `data-testid="connected"` reads "connected".
- `test_tester_can_call_rust_methods` — dispatches `e2e_set_message`, asserts the page updates; dispatches `e2e_clear_message`, asserts it clears.

To run invisible: `HEADLESS=1 c r`.

## Not there yet

Nothing below exists. Do not assume it works.

- Test reports beyond pytest stdout.
- Artifact collection on failure (screenshots, logs, traces).
- CI integration.
- A fresh test server per test. One server runs for the whole pytest session.
- Any state endpoint on the server. The tester can only see what the app shows; it can't ask the server what it holds.

## What to know before changing this

The three-layer split is deliberate. Do not "simplify" by merging the orchestrator into pytest or the tester into the orchestrator. If a future need pulls them together, note why in this file.

Macros in `methods_called_by_tester.rs` stay macros, invoked inline, torn down via `on_cleanup`. `install()` was tried and removed.

The orchestrator picks free ports so `c r` can run twice without a "port in use" error. If you see `server_port=3000` in the web app URL, the port override failed — check `orchestrator/src/main.rs`.

## Server assertions: read the db file after the run (2026-10-31)

The orchestrator spawns `zakken_server` with `zzpath /tmp/zakken_e2e_world.sqlite`. After pytest exits and the server is killed, the orchestrator can open that file with `rusqlite` and check whatever it wants — that the page row exists, that the love letter row exists, that a seen-letter id is present, and so on. The server's handle is closed, no lock contention, no second browser, no test-only endpoint.

This is not implemented yet. It's the agreed shape for the next time we need "did the server actually store X". The existing tests only assert on what the page shows; none of them read the db.

The "check server state through a second browser client" approach was considered and is still valid for a different question — "does client B see what client A did". That needs two browsers. "Did the server write it to disk" does not. Different tests, both useful.

Where to put the assertion code when we get to it: a new module in `orchestrator/src/`, called by `main.rs` after the server is killed. Failure there is reported separately from pytest, exit code still non-zero.
