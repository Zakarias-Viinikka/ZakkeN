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

## Update - 2026-11-05

Two browser fixture, drift gating, and the first cross-client test are in.
Everything below works. Test 003 passes end to end.

### Two browser stage

`support/two_browser_stage.py` — `TwoBrowserStage.launch(playwright, url)`.
Takes the shared Playwright instance (never starts its own; the sync API
forbids two). Launches two Chromium browsers (two OS windows, two OPFS,
two user_ids). Paints a coloured corner label on each ("BRAVE" blue,
"EDGE" red, top-right). Methods `make_brave_big`, `make_edge_big`,
`share_screen_with_edge_and_brave` reposition via CDP
(`Browser.setWindowBounds`). Test code uses `stage.brave` / `stage.edge`
as ordinary Playwright Pages.

Fixture in conftest: `two_browser_stage`. Registers both browsers via
`bring_up_registered` (below). Does NOT close on teardown, so the human
can inspect after the test.

### New helpers

`support/setup_page.py` — `bring_up_registered(page, port, name_prefix)`.
Handles three entry states: `/custom_port_setting_page` (fresh), `/setup`
(after reset_everything), `/Menu` (already registered). Waits for any
in-flight reload before deciding which state it is.

`support/client_db.py` — `read_titles(page)` (fires `read_titles` window
event, polls `window.__read_titles__`), `read_client_state(page)` (fires
`read_client_state` event, returns `{url, user_id, server_name,
drift_version, port, page_count}`).

`support/test_events.py` — `wipe_server`, `reset_everything`,
`insert_three_pages`. Fire the matching window events.

`support/popup.py` — `popup(page, msg)` (fires show_popup),
`assert_no_error_popups(page)` (reads `window.__popup_log__`, fails if
any entry has `kind == "error"`), `clear_popup_log(page)`.

`support/db.py` — adds `wait_until_server_has_pages(count)`.

`support/speed.py` + `speed_config.json` at `tester/` root — per-test
speed multiplier. Wait calls (`pause`, `speed.sleep`, `speed.budget`)
scale. Playwright timeout calls (`seconds()`) do NOT scale; timeouts are
failure detection, not pacing. See `how_to_write_tests.md`.

### New expose methods (web app)

In `expose_methods_for_testing/`:
- `wipe_server`
- `reset_everything`
- `insert_three_pages`
- `read_titles` → `window.__read_titles__`
- `read_client_state` → `window.__client_state__`

All registered in `expose_methods_main!()`.

`leptos_components/small_components/popup.rs` grew `show_error(msg)` and
a `log_popup_to_window` that appends `{kind, message}` to
`window.__popup_log__` on every popup. This is what
`assert_no_error_popups` reads.

### Bug fixes landing with test 003

- `dev_actions::insert_three_pages` now bumps `MENU_TICK` (previously
  went via reload to refresh the menu; the window-event path didn't).
- `handle_one` in the receiving worker bumps `MENU_TICK` on every
  successful apply (was gated on `outcome.changed`, which is false for
  pages that aren't currently open).
- `menu.rs` `For` key is `(db_id, title)` — was `db_id` alone, so a
  title going from `""` to `"Grocery list"` didn't re-render.
- `bring_up_registered` waits for the reload that `reset_everything`
  triggers before checking which entry state it's in.

### Orchestrator build check

`main_for_building_the_web_app_build` is now content-based, not
mtime-based. Hash of every tracked input lives in
`dist/.source_fingerprint`. If the hash matches, skip trunk. If not,
rebuild. mtime comparison was silently missing changes (atomic writes,
or a second `trunk serve` touching dist).

### How test 003 works

`tests/test_003_two_clients/test_two_clients_after_wipe.py`:
1. Brave wipes the server (both still on /Menu, both get kicked to
   /setup on next reload).
2. Brave resets, registers a new name. Edge resets, registers a new
   name. Both bootstrap from the empty server dataset.
3. Brave inserts three pages. Waits for the server to have 3.
4. Edge should receive them. Asserts `read_titles(edge)` returns
   `["Grocery list", "Work notes", "Ideas"]`.
5. No error popups fired on either side.
6. Clicks the "Grocery list" button on Edge, asserts the page opens
   with 6 `textarea.ps-content` blocks.

Popups are fired via `both(stage, msg)` at every numbered step
(`0/7` … `7/7`) so a watching human can follow along.

## Update - 2026-11-07

Audit-bug tests written, plus helpers, narration and speed changes.

### Tests and what they showed

- `tests/test_004_text_sync.py` has two tests. Both use the two-browser stage, even though only Brave edits, because the stage fixture closes the shared `page`.
  - Select `il` in "Milk", type `X`, reload, expect `MXk`. FAILS: the block comes back as `Mk`. Confirms audit #1 (`get_diff` loses text on a length-changing selection replace).
  - Type `åäö` at the end of "Milk", wait for sync, type `X` after it, reload, expect `MilkåäöX`. FAILS: the page dies with a yrs panic ("attempt to subtract with overflow", yrs-0.27.4 block.rs:1613) and the X never reaches the server. Confirms audit #3 (chars vs bytes vs UTF-16). Worse than lost text: the wasm instance is killed.
- `tests/test_005_remote_title_edit.py`: Edge stays on /Menu, Brave appends " v2" to the title block of "Grocery list", Edge's menu must show it without a reload. PASSES. Audit #6 is fixed. It is a regression test now.
- The two 004 tests are expected to keep failing until audit #1 and #3 are fixed. They are the bugs, not harness problems.
- Renaming from the menu isn't testable: the pen button has no click handler yet, so tests rename by typing in the title block of the open page.

### New helpers (tester/support)

- `fresh_world.py`: `fresh_world_with_three_pages(stage)` wipes the server, resets and re-registers both browsers, has Brave insert the three seed pages, waits for Edge to receive them, waits for the letter count to settle, then clears both popup logs. Also server-side waits that watch the db instead of sleeping: `count_letters`, `wait_for_new_letter(before, page, timeout_secs)` (table is `love_letters_from_clients`), `wait_until_letters_settle`, `wait_until_titles`, `count_users`, `wait_until_users_drop_below`.
- `blocks.py`: find blocks by text, never by position (`block_values`, `block_with_value`, `wait_for_value_in_blocks`), plus `open_page_from_menu`, `reload_and_wait_for_blocks`, `select_range`, `caret_to_end`. `watch_js_errors(page)` collects page errors and wasm panics for stage pages (the shared `page` fixture does this only for itself). `report_js_errors_on_failure` prefixes a plain sentence ("The app crashed (...)") to any failure when errors were seen.
- `narrate.py`: `say`, `say_both` (popup, then hold scaled by speed), `announce_result(stage)` (wrap the whole test; puts TEST PASSED or TEST FAILED: reason on both screens so a failure is never silent), `app_said(page)` (popups the app itself showed, appended to failure messages).
- Narration messages must start with a digit, "Setup", "Step" or "TEST". `app_said` uses that to tell them apart from app popups. Keep it when adding new narration.
- Popup text is plain language, no technical terms. The human asked for this. Test 003 still has its old popups and does not use `announce_result`.
- Stage tests call `stage.close()` as their last line, so browsers close on a pass and stay open on a failure for inspection.

### Speed

- `speed_config.json`: 000/001/002 at 0.1, 003 at 0.5, 004/005 at 1.5. The two-browser fixture's sleeps now go through `speed.sleep`. `test_000`'s db-poll deadline is a fixed 3 s: timeouts must not scale with pacing.

### App fix that landed with these tests

- `menu.rs` `create_local_pages` panicked when the Menu unmounted while a menu read was in flight (a disposed signal read after the async await). Now uses `try_get_untracked` and `try_set`. A disposed signal there means nobody is waiting for the result, so this isn't hiding errors. Other `spawn_local` tasks that touch component signals after an await may have the same issue; the JS-error check should flag them.

### Not done

- Drift-gating tests #10 and #11 were skipped: the web app doesn't use the lonewolf state yet. The deferred ones (4 to 8) still wait on reconcile.
- Next candidates: audit #2 (`Delete` combine), drag-then-edit snap-back (needs mouse-drag helpers), the rejected-letter queue block.

## Update - 2026-11-11

Reconcile e2e test, plus infrastructure cleanups.

### New test

`tests/test_006_reconcile/test_reconcile_client_only.py`. Walks the
full reconcile flow: fresh world → brave wipes the server → edge goes
lonewolf → brave resets and re-registers fresh → edge opens
`/reconcile` → edge clicks Finish → wait for Connected → navigate back
to `/Menu` → assert titles and server page count. Only client-only
pages are covered; the diff-viewer path (both sides have the same
page id with different content) isn't a test yet.

### Timeout variable

`support/timeouts.py` is new. `DEFAULT_SECS = 2.0` is the one number
every test uses. Two helpers: `ms()` for Playwright `timeout=`, `secs()`
for Python-side `timeout_secs=`. Every call site was migrated. If a
specific wait legitimately needs longer (e.g.
`_RECONCILE_FINISH_SECS` in `support/reconcile.py`,
`_SETUP_BUTTON_STABLE_MS` in `support/setup_page.py`), the constant is
defined beside the call site with a one-line comment saying why.

### New helpers

- `support/read_conn_state.py` — `read_conn_state(page)` and
  `wait_for_conn_state(page, wanted, timeout_secs)`. Fires
  `read_conn_state` window event, reads `window.__conn_state__`.
  Accepts one wanted value or a tuple. Catches "Execution context was
  destroyed" during reload and retries.
- `support/reconcile.py` — `open_reconcile` (fires the `navigate` event
  with `/reconcile`, because the dev-sidebar button is inside a
  slide-out panel and out-of-viewport clicks time out), `click_finish_reconcile`,
  `wait_for_reconcile_to_finish` (waits for CONN_STATE = Connected, not
  for the URL to change: the reload keeps the URL).
- `method_read_conn_state!()` on the web app, registered in
  `expose_methods_main!()`.

### Infrastructure fixes

- `orchestrator/src/main_for_running_the_tester.rs` forwards the
  caller's argv to pytest, so `c r -- -k test_xxx` works.
- `two_browser_stage.py` respects `HEADLESS=1`, and sets a default
  timeout on each page.
- `conftest.py` fixture now registers both browsers *before* firing
  the "Brave is signed in, now signing in Edge" popups. The old order
  had a popup fading over the setup button while Playwright tried to
  click it, which timed out at 2s.
- `read_titles`, `fresh_world`, `blocks`, `db`, `setup_page`,
  `reconcile` all use `timeouts.ms()` / `timeouts.secs()`.
- `speed_config.json` lowered: 003 from 0.5 to 0.3, 004/005 from 1.5
  to 1.0.
- `test_006` removed all `pause()` calls.

### Known test-organization issue

The `retry N` zap verb re-runs the *whole* chain even when only one
step failed. That means any subsequent step whose old-text already
matched from an earlier successful run will fail. The workaround is
to either send only the failed steps in a fresh chain, or re-order
so already-applied steps aren't in the retry set.

### Existing failures (unchanged, expected)

`test_004` still fails on the audit bugs (get_diff losing text, yrs
panicking on non-ASCII). Those are the bugs, not the harness.
