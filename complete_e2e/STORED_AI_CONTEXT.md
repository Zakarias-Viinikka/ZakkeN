# complete_e2e — stored AI context

Written 2026-11-01. Fresh AI: this file gives you the e2e picture. The human will hand you their workflow and style guides separately.

## What complete_e2e is

Three-layer harness for testing `cqrs_state_synchronization/web_interface` against `zakken_server`, plus the test material that verifies the harness is actually worth something.

The three layers:

- **Orchestrator** (Rust binary in `orchestrator/`). Builds the server, builds the web app with trunk, picks free ports, spawns the server with a fresh temp db, serves `web_interface/dist` over HTTP, opens the browser at `?server_port=N`, runs pytest, kills the server, exits with pytest's exit code. Never opens a browser directly, never asserts domain facts.
- **Tester** (`tester/`, Python + pytest + Playwright). Drives the browser. Owns all assertions. Doesn't spawn anything.
- **Web app + server** — the real `web_interface` and `zakken_server`, not mocks. Their inputs come from `zzpath` / `zzport` args (server) and the `?server_port=N` query string (web app).

`c r` from `complete_e2e/` runs everything. `local_cargo.sh` prepends `.venv/bin` to PATH so pytest finds the right Python. Do NOT `source .venv/bin/activate` — a copied venv's activate script points at the original folder's Python, which caused real flaking earlier (headed mode `TargetClosedError`, headless passed).

## What was changed to make it work

In `zakken_server`:

- `personal_db_wrapper.rs` reads `zzpath <path>` from args, falls back to `zakken.sqlite` (renamed from `test_db.sqlite`). Panics if the flag is present without a value.
- `server_core/main.rs` reads `zzport <port>` from args, falls back to 3000. Panics on missing value or non-numeric.
- `socket_testing_table` scaffolding deleted (unused after the real server became the target).

No test-mode flag. The guard against touching real data is that the orchestrator always passes `zzpath /tmp/zakken_e2e_world.sqlite` and wipes it before spawning. Same effect, less surface.

In `web_interface`:

- `build_ws_url` reads `server_port` from the URL query string, falls back to 3000.
- `expose_methods_for_testing/` — folder with `core.rs` (has `expose_methods_main!()` macro), `method_show_popup.rs` (`method_show_popup!()` macro). Called once from `App` component body. Each `method_*` sets up a `window` CustomEvent listener torn down by `on_cleanup`. Currently one method: `show_popup` fires a popup with the event's `detail`. Intended as the pattern for exposing more Rust methods to the tester.
- Popup swapped to CoolPopup from `connecting_to_server_template` (`cool_popup.rs`). `create_popup` renamed to `show_popup` via astgrep across the crate. `PopupContainer` replaced by `CoolPopup` in main.rs.

In `complete_e2e/orchestrator/` (all of this was written this session):

- `main_for_building_the_server_binary.rs` — `cargo build -p server_core` in `zakken_server/`.
- `main_for_building_the_web_app_build.rs` — `trunk build` in `web_interface/`, skipped if `dist/index.html` is newer than all inputs.
- `main_for_starting_the_server_process.rs` — spawns `zakken_server/target/debug/zakken_server` with `zzport N zzpath /tmp/zakken_e2e_world.sqlite`, waits until the port answers TCP.
- `main_for_serving_the_web_app_build.rs` — axum static serve of `web_interface/dist`, SPA fallback to `index.html`.
- `main.rs` — glues the above, picks free ports, builds the browser URL with `?server_port=N`.

`complete_e2e/Cargo.toml` is `members = ["orchestrator"]` only. The real server and web app are reached via path dependencies / shell commands, not workspace members.

Tester currently has two tests: `test_web_app_loads_without_js_error` (page loads, no JS errors) and `test_show_popup` (fires the `show_popup` event, checks `.pp-toast-title` gets the text). The `page` fixture is session-scoped — one window for the whole run, not one per test (per-test was opening a window each time and looked like duplicate launches). Tests each navigate themselves.

## Decisions made

Assertions live in pytest. The orchestrator never decides pass or fail on a domain fact. It builds, spawns, serves, runs pytest, tears down. Anything that says "this should be true" is in Python.

Server state comes from reading the db file, not a query endpoint. After pytest exits and the server is killed, open /tmp/zakken_e2e_world.sqlite and read whatever the test wants to check. Blobs need Rust to deserialize, so the shape is: Rust reads the db and returns flat JSON, Python asserts on the JSON. Plain columns can be read directly in Python. No new server endpoint needed. Not built yet.

Raw SQL in Python tests is fine. The query is a snapshot of what the test author assumed the schema was. If the schema drifts, the test fails loudly and you investigate. The one drift raw SQL won't catch is a column being added — old queries still work, new data is invisible. Accepted.

Known bugs become test material, not fixes. The three-pass audit list stays unfixed. When the harness is up, the first thing it runs against is the audit's priority shortlist. Tests that fail on known-bug behavior prove the harness can detect bugs. Tests that pass mean the harness is blind to that bug. Either result is a calibration signal. Non-testable entries (server pruning, mutex poisoning) get fixed later, on their own.

Two browsers, not a mock client. Drop zakken_server/mock_client/ from the plan entirely. Two independent Playwright-launched browsers are two independent clients — separate OPFS, separate user_id, separate socket, separate login. Same server. Cross-client tests use both page fixtures. One-client tests still use just one. The second page fixture isn't built yet; add it when the first cross-client test needs it.

## What's pending

In rough order of "unblocks the most":

1. **Second browser fixture.** Add to `tester/conftest.py`, probably session-scoped like `page`. Used by cross-client tests. Nothing exists yet.
2. **First e2e test against the audit findings.** Highest-priority candidate: `get_diff` losing text on length-changing selection replace (`text_diff` finding #1). Single-client, provable, would answer "is the harness good enough" in one test. Other candidates: first-run server link never starting, drag / stale `page.blocks`, `apply_blob` `changed=false` for non-open pages (this one needs two browsers).
3. **Db-reading step in the orchestrator.** After pytest exits and the server is killed, read the sqlite file. Two ways: Rust side with `rusqlite` for blob-shaped data (returns JSON), or Python `sqlite3` for plain columns. Not built yet.
4. **Block-id helpers for tests.** Web app blocks live in the drag black box, so DOM order isn't the visual order. First attempt: sort DOM elements with `data-block-id` by `bounding_box().y` in the test. If the black box virtualizes (some blocks not in the DOM), fall back to exposing a read method from the box.
5. **Data-testids on elements.** Tests currently reference `.pp-toast-title` and similar. Adding `data-testid` attributes to the elements that tests need makes selectors survive UI changes. Tests should target model things (block ids, page ids, "does this block exist") not presentation (button text, colour, layout position) — the UI will change and presentation-bound tests will all have to be rewritten.

## Related docs

- `2026_10_31_cqrs_and_server_audit_findings.md` at ZakkeN root — the test-material list.
- `todo.md` at ZakkeN root — "cqrs and server audit findings" section names the priority shortlist.
- `e2e_webapp_server_mock.md` at ZakkeN root — the larger plan for a production e2e harness. `complete_e2e` is a smaller proof-of-shape; the plan doc has an addendum noting the difference (fixtures vs orchestrator split) and the two-browser choice.
- `rust_practice/e2e_template_for_web_with_server/` — the template `complete_e2e` was copied from. Both share the same `local_cargo.sh` PATH-prepend trick. The template's `STORED_AI_CONTEXT.md` explains the venv trap.
- `STORED_AI_CONTEXT.md` at ZakkeN root — mentions both the plan and the proof-of-shape.