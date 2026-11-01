# complete_e2e — stored AI context

Written 2026-11-01. Fresh AI: this file gives you the e2e picture. The human will hand you their workflow and style guides separately.

## What complete_e2e is

Three-layer harness for testing `cqrs_state_synchronization/web_interface` against `zakken_server`, plus the test material that verifies the harness is worth something.

- **Orchestrator** (Rust, `orchestrator/`). Builds the server, builds the web app with trunk, picks free ports, wipes the temp db, spawns the server, serves `web_interface/dist` over HTTP, opens the browser at `/custom_port_setting_page`, runs pytest (passing `WEB_APP_URL` and `SERVER_PORT` as env vars), kills the server, exits with pytest's exit code. Never asserts domain facts.
- **Tester** (`tester/`, Python + pytest + Playwright). Drives the browser. Owns all assertions.
- **Web app + server** — the real ones, not mocks. The server reads `zzpath` / `zzport` from args; the web app learns the port from an exposed method (below).

`c r` runs everything. `local_cargo.sh` prepends `.venv/bin` to PATH so pytest finds the right Python. Do NOT `source .venv/bin/activate` — a copied venv's activate script points at the original folder's Python, which causes headed `TargetClosedError` flakes while headless passes.

## How the port flows

The orchestrator picks a free port so `c r` can run twice without "port in use". The port is not in the URL.

The app boots at `/custom_port_setting_page` — a marker route that doesn't auto-redirect. Py fires the exposed `set_port` event with the port number. That writes sessionStorage and flips the connect gate to `AllowedToConnect`. Py then fires the exposed `navigate` event to `/` (SPA route change, no reload). `SetupGuard` redirects to `/setup`. Registration runs. Once `USER_ID` exists, the connect effect fires.

The connect effect in `main.rs` waits for four things: `INIT_DONE`, `SESSION_ID`, `USER_ID`, and the gate being `AllowedToConnect`. All are read with `.get()`. An untracked read means the effect fires once and never re-runs — that was a real bug: on a fresh run `USER_ID` becomes `Some` *after* navigation, so the effect never re-fired, and the socket only started on the next full page load.

Why not the URL: the router drops `?server_port=N` on the first in-app navigation. Anything reading the port later sees only the default (3000). That also "worked" in tests only because every test reloaded with the query string intact — masking both the port bug and the connect-on-fresh-run bug above.

## What was changed to make it work

In `zakken_server`: `personal_db_wrapper.rs` reads `zzpath <path>`; `server_core/main.rs` reads `zzport <port>`. Both panic on malformed flags. No test-mode flag — the orchestrator always passes `zzpath /tmp/zakken_e2e_world.sqlite` and wipes it (plus `-wal` and `-shm`) before spawning. Server log lines are prefixed `[server]`.

In `web_interface`:

- `owner_of_interacting_with_server/server_port.rs` — `get_port() -> Option<u16>`, `set_port(n)` writing sessionStorage and flipping the gate.
- `owner_of_interacting_with_server/connecting_gate.rs` — `Blocked` / `AllowedToConnect`, starts blocked (or allowed if sessionStorage already had a port, so reloads don't reset).
- `owner_of_interacting_with_server/core.rs` — `connect_to_socket()` guards on the four conditions and `LINK_STARTED`; callable from the connect effect and from the exposed method.
- `leptos_components/custom_port_setting_page.rs` — the marker route.
- `leptos_components/setup_guard.rs` — redirect-to-setup component, with `PATHS_THAT_SKIP_SETUP`.
- `expose_methods_for_testing/` — four `method_*!()` macros: `show_popup`, `set_port`, `connect_to_socket`, `navigate`. Invoked once from `App`'s component body.
- `setup_page.rs` — `username_input` and `finish_setup_button` testids.

In `complete_e2e/orchestrator/`: build server, build web app (skipped if `dist/` up to date), pick free ports, wipe temp db, spawn server, axum static serve, run pytest with `WEB_APP_URL` + `SERVER_PORT`. `main.rs` wires it.

## Tester layout

    tester/
      conftest.py      # fixtures
      pytest.ini       # testpaths = tests
      run_tests.sh     # prepends venv to PATH, runs pytest
      tests/           # test_*.py, numbered test_000_, test_001_, ...
      support/         # helpers, no test_ prefix
        db.py
        popup.py
        seconds.py
        setup_page.py

Fixtures in `conftest.py`:

- `browser` — Chromium, headless if `HEADLESS=1`.
- `page` — session-scoped, one window for the run. Attaches `load` / `framenavigated` / `console` / `pageerror` listeners and accumulates `js_errors`.
- `web_app` — session-scoped. `goto(/custom_port_setting_page)`, fire `set_port`, fire `navigate` to `/`, fire a popup. Returns the page.
- `web_app_url` — from `WEB_APP_URL` env var.
- `pause` — no-op when `HEADLESS=1`.
- `_no_js_errors_per_test` — autouse. Fails the current test if new JS errors appeared during it. Without this, pytest hides stdout of passing tests, so a panic during a passing test is invisible until a later test fails on the accumulated list.

`support/db.py` reads `/tmp/zakken_e2e_world.sqlite` directly: `get_path`, `connect_readonly`, `list_tables`, `query`. Read-while-running works.

`support/seconds.py` — `seconds(n)` converts to Playwright's millisecond timeouts.

`support/setup_page.py` — `on_setup_page`, `setup_page_register(page, name)`.

`support/popup.py` — `popup(page, message)`.

## Existing tests

- `test_000_register_if_necessary.py` — if on `/setup`, registers a fresh random name and asserts the row in `registered_users`. Skips if already past setup.
- `test_001_web_app_connects_to_server.py` — `test_show_popup` fires the `show_popup` event and checks `.pp-toast-title` (last) gets the text.
- `test_002_read_db_while_server_running.py` — opens the db read-only mid-run, lists tables, fires a popup with the count.

## Decisions made

Assertions live in pytest. The orchestrator never decides pass or fail on a domain fact. It builds, spawns, serves, runs pytest, tears down. Anything that says "this should be true" is in Python.

Server state comes from reading the db file, not a query endpoint. After pytest exits and the server is killed, open /tmp/zakken_e2e_world.sqlite and read whatever the test wants to check. Blobs need Rust to deserialize, so the shape is: Rust reads the db and returns flat JSON, Python asserts on the JSON. Plain columns can be read directly in Python. No new server endpoint needed. Partly built: plain-column reads work via `support/db.py`. Blob reads not built yet.

Raw SQL in Python tests is fine. The query is a snapshot of what the test author assumed the schema was. If the schema drifts, the test fails loudly and you investigate. The one drift raw SQL won't catch is a column being added — old queries still work, new data is invisible. Accepted.

Known bugs become test material, not fixes. The three-pass audit list stays unfixed. When the harness is up, the first thing it runs against is the audit's priority shortlist. Tests that fail on known-bug behavior prove the harness can detect bugs. Tests that pass mean the harness is blind to that bug. Either result is a calibration signal. Non-testable entries (server pruning, mutex poisoning) get fixed later, on their own.

Two browsers, not a mock client. Drop zakken_server/mock_client/ from the plan entirely. Two independent Playwright-launched browsers are two independent clients — separate OPFS, separate user_id, separate socket, separate login. Same server. Cross-client tests use both page fixtures. One-client tests still use just one. The second page fixture isn't built yet; add it when the first cross-client test needs it.

## What's pending

1. **Second browser fixture.** Add to `tester/conftest.py`, session-scoped like `page`. Cross-client tests use it.
2. **First e2e test against the audit findings.** Candidate: `get_diff` losing text on length-changing selection replace (`text_diff` finding #1). Single-client, provable, would answer "is the harness good enough" in one test.
3. **Blob-shaped db read.** Rust side with `rusqlite` reading `blobbed_page` / `love_letter_sketch`, returning JSON for pytest. Plain-column reads already work.
4. **Block-id helpers.** Web app blocks live in the drag black box, so DOM order isn't visual order. Sort DOM elements with `data-block-id` by `bounding_box().y`, or expose a read method from the box.
5. **Data-testids on elements.** Tests reference `.pp-toast-title`. Adding testids to the elements tests need makes selectors survive UI changes. Test the model (block ids, page ids), not presentation (button text, colour).

## Related docs

- `/home/zakke/ProgStuff/ZakkeN/2026_10_31_cqrs_and_server_audit_findings.md` — the test-material list.
- `/home/zakke/ProgStuff/ZakkeN/todo.md` — "cqrs and server audit findings" section.
- `/home/zakke/ProgStuff/ZakkeN/e2e_webapp_server_mock.md` — larger plan for a production e2e harness.
- `/home/zakke/ProgStuff/rust_practice/e2e_template_for_web_with_server/` — the template. Its `GOTCHAS.md` lists the cost-time issues.