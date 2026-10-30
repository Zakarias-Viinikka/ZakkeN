# Web E2E harness — implementation plan

> Plan by the expensive AI, 2026-10-30. Path references updated for the cqrs naming pass of the same day (see `ZakkeN/coding_style.md`): `server_link/` → `owner_of_interacting_with_server/`, `receive.rs`/`merge.rs`/`send.rs`/`identity.rs`/`two_tab.rs` → new names, `db/kick_send.rs` → `db/main_send_unsent_love_letters.rs`, `kick_send()` → `main_send_unsent_love_letters()`, `Batcher::new()` → `BatcherHandle::spawn_task_and_get_handle()`. Path references below are updated. New code should follow `ZakkeN/coding_style.md`.

The eight sections are in the order asked for.

Four calls to check before you hand it over:

I dropped ResetTestData and __e2e_wipe_local in favour of one fresh server process per test, each with its own temp DB and port. That removes every protocol change and every stale-state problem. The test server and mock client become function-scoped as a result.
cli_gui gets a --plain mode. Its redraw-the-screen output can’t be parsed reliably, and it only reads its socket while a request is in flight.
The web client must pin ?test_mode and server_port at startup and stash them in sessionStorage. The setup redirect, and probably a reload after setup, drops the query string. The client would then silently fall back to port 3000, which is the real server.
The ping is a plain HTTP route pinged by the Python harness, not a websocket request, so Python never has to speak the Rust protocol.

Implementer prompt: web E2E harness for ZakkeN

You are the implementer. A stronger model has already done the design thinking, and this document is the plan. The repo root is ZakkeN/. The relevant directories are:

zakken_server/ (the axum server, mock_client/, and the shared crates)
cqrs_state_synchronization/web_interface/ (the Leptos CSR client)
e2e_android_tests/ (read it for shape and conventions, not content)

The deliverable is e2e_web_tests/ plus the small production changes listed below, ending in one passing happy-path test.

Rules of engagement

The plan is settled. If you think part of it is wrong, say which part and why before deviating. Never quietly implement around it.
Where the plan says “find out” or names a fallback, do that and report which branch you took.
Scope is exactly what is listed. No extra tests, no dataset generation, no yrs hard-copy primitive, no template-test work, no Android changes, and nothing in connecting_to_server_template. If one of those blocks you, say so instead of building it.
Get each layer working before starting the next (order and checkpoints are in Section 7).
Match the codebase’s commenting habit: plain prose that explains why.
Add no dependencies beyond the ones named.
1. Decisions I’m changing, and why

No B decision changes. Changes to C:

C#5 (ResetTestData) is dropped. The isolation unit is one fresh world per test: a new server process, a new temp DB file, and a new port.
A reset request needs a new variant in the shared Request enum, so every crate that matches on it must compile again, Android’s included.
It also needs an exemption from the “not logged in” gate in do_request, and a table-wipe primitive that isn’t visible in the files I was given.
Even then it leaves stale in-memory state. The server’s telephone keeps its socket→user map. The mock client keeps its in-memory pages, and the wipe deletes its registered-user row, so the server would stop fanning letters out to it.
A fresh process has none of these problems and costs a few hundred ms per test.
C#6 (window.__e2e_wipe_local()) is dropped. C#10 already makes the browser context function-scoped, and a new context starts with empty OPFS and localStorage. The function would be dead production code. See Risk 1 for the caveat.
C#10 scopes change as a consequence. Test server and mock client are function-scoped. Browser and a static web-app server stay session-scoped. Context and page stay function-scoped, as in C.
C#8 stands (subprocess, stdin/stdout, no library mode) with one addition: cli_gui gets --plain alongside the server-URL argument. Normal output clears the screen and redraws a header after every command, which can’t be parsed reliably. The client also reads its socket only while a request is in flight, so the harness needs a clean line protocol and a “look at the world now” command (sync).
C#7 is sharpened, not reversed. The query string is read once at startup and persisted in sessionStorage, and server_port is honoured only when test_mode=1.
SetupGuard client-navigates to /setup, which drops the query string, and the setup flow very likely reloads the page afterwards.
A lazily-read server_port would silently fall back to 3000, the real server.
C#2 stands with stricter validation (Section 2). Test mode requires an explicit, non-default --db-path and --port. --ping-timeout is legal only in test mode, with a default of 10 s.
B’s open “where does the ping live” is resolved as a plain HTTP route on the test server, pinged by the Python harness. This is a resolution, not an overturn. Python then never has to speak the Rust protocol (payload encoding, login).

Accepted as written:

C#1, C#3, and C#9 (keep the script name run-ui-tests.sh for parity).
C#4: the reinterpretation is right, and per-test worlds make it stronger.
C#11 and C#12: nothing out of scope blocks this work.

One Android convention can’t be mirrored: its README says “Do not modify production app code”. This task needs small production changes (Sections 2–4). List them in the new README.

2. Server-side plan

A world is a process. The server only needs to learn four things: which DB file, which port, whether it is a test world, and to die if its owner stops pinging. There is no protocol change and no new request variant. Nothing in socket.rs, allowed_requests/, the queue, telephone, or server_protocol is touched.

Files touched (paths under zakken_server/server_side/):

db/src/personal_db_wrapper.rs:
Make the default path a public constant, value unchanged (test_db.sqlite).
Add a variant of new_liver that takes a path.
Keep new_liver() working by delegating to it, since other callers may exist (grep).
server_core/src/setup.rs: setup_db takes the db path and passes it through. Grep for other callers and fix them.
server_core/src/config.rs (new, in the lib so it is unit-testable): argument parsing and validation.
server_core/src/test_mode.rs (new, in the lib): ping clock, ping route, watchdog.
server_core/src/lib.rs: declare the two new modules.
server_core/src/main.rs: wiring.
server_core/Cargo.toml: only if something is missing, for example tokio’s time feature for the watchdog’s sleep. Don’t add clap unless the zakken_server workspace already uses it. Four flags are easy to hand-parse.

Arguments. They are parsed in config.rs, called first thing in main, before the DB is opened. The flags are --db-path, --port, --ping-timeout (integer seconds), and --test-mode. Defaults reproduce today’s behaviour exactly: test_db.sqlite relative to cwd, port 3000, bind 0.0.0.0. Rules:

An unknown flag, a missing value, or an unparsable number prints the problem and a usage line to stderr and exits with code 2.
--ping-timeout without --test-mode exits 2. It must be at least 1, and defaults to 10 when test mode is on.
--test-mode exits 2 unless --db-path and --port were both given explicitly. It also exits 2 if the port is 3000 or the db path’s file name is test_db.sqlite. This makes it structurally impossible for a harness bug to point a test world at real data. The harness therefore names its file world.sqlite.
Parsing returns a plain config value, with nothing global.
Add unit tests in the same file for defaults, each error, and the test-mode guards.

Test-mode behaviour (test_mode.rs, wired from main):

Bind 127.0.0.1:<port> instead of 0.0.0.0, which keeps throwaway worlds off the LAN.
Keep a shared “last ping” clock (an Arc<Mutex<Instant>> is fine), initialised at server start.
The ping route is GET /__test/ping. It answers 200 and stamps the clock.
Build it as its own small router whose state is the clock, and merge it into the main router only in test mode. Outside test mode the route doesn’t exist (404). That is the “refused outside test mode” guarantee.
The watchdog is a spawned tokio task that wakes every second. If now - last_ping > timeout, it writes one line to stderr and calls std::process::exit(42).
Write directly to stderr, because log() goes through an async queue that may not flush.
There is deliberately no graceful shutdown.
Because the clock starts at launch, a harness that dies right after spawning leaves no orphan.
Spawn it after the listener is bound and before axum::serve.
Print one banner line to stderr at startup: TEST MODE, db path, port, timeout.

Also make the “listening on” log line print the actual bound address. Everything else in main.rs is unchanged.

Your questions, answered directly:

Argument parsing lives in config.rs.
The ping check lives in test_mode.rs, as a watchdog task started from main.
The reset handler doesn’t exist.
The new request variant doesn’t exist (see Section 1).

Check what server_core::logging::log does with its lines. If they don’t also reach stdout or stderr, don’t change log. The harness will read the logs table on failure instead (Section 5, support/server.py).

3. Mock client plan

All changes are in zakken_server/mock_client/cli_gui/src/main.rs, plus serde_json in that crate’s Cargo.toml. It is already in the workspace via server_protocol. mock_client_core is untouched.

Arguments (hand-parsed):

--server-url <ws url> replaces the hard-coded constant, which stays as the default. The header line should print the runtime URL.
--plain switches on the line protocol below.

Without --plain, behaviour must be unchanged apart from using the runtime URL.

Plain-mode contract. The harness and cli_gui must agree on this exactly:

After a successful connect, login/register, and page load, print one line: READY. A connect failure behaves as today (stderr message, exit 1).
For each stdin line, execute the same commands as today. Print the result lines with no indentation, no screen clear, no header, and no > prompt. Then print one sentinel line, <<END>>, and flush.
pages prints exactly one line of JSON. Its shape is an object with a pages array, where each page has page_id, disabled, and a blocks array of {index, block_id, text}. Use raw text, with no (empty) substitution. Build it with serde_json::json! from the existing pages_info() data.
An unknown command prints today’s message lines, then the sentinel.
quit and stdin EOF exit 0 with no sentinel. Process exit is the signal.

Behaviour the harness must understand. The client only reads its websocket while a request is in flight. A letter pushed by the server while the process sits idle is applied the next time a command runs. So sync is the mock client’s “look at the world now” command: send sync, then pages. Duplicate delivery (push plus pull) is harmless. Merges are idempotent, and acks for already-acked letters get replies the client skips.

4. Web client plan

All changes are in cqrs_state_synchronization/web_interface/src/.

test_config.rs (new, declared in lib.rs). It computes a small config once: enabled and an optional server_port. The rules:

Inputs: window.location.search (test_mode, server_port) and sessionStorage (keys zakken_e2e_test_mode and zakken_e2e_server_port).
Precedence:
test_mode=1 in the URL enables it and writes both keys.
test_mode=0 clears both keys and disables it.
With neither, the stored keys are used.
Port validation: server_port is parsed as a u16. If test mode is enabled but the port is missing or invalid, treat the whole config as disabled and console.warn. The harness’s websocket guard catches that case.
Storage failures: tolerate them and just use the URL.
Compute once: use a OnceLock. main() calls the initialiser as its very first line, before mount_to_body. This pins the value before SetupGuard’s navigation can change the URL.
Browser API access: use js_sys::Reflect, the same way build_ws_url does, or enable the needed web-sys features, whichever touches Cargo.toml less.

owner_of_interacting_with_server/core.rs::build_ws_url(). Use the config’s port when enabled, otherwise 3000. Scheme and hostname logic is unchanged. Grep cqrs_state_synchronization/ and zakken_server/client_internals/ for other hard-coded 3000 or ws:// connection URLs, and route any that you find through the same config.

Indicator. Add a small component in main.rs (like SetupGuard). Place it in App’s view outside the tables_ready gate, so it shows from first paint during “Loading...”. It renders nothing when the config is disabled.

Visible part: fixed to the bottom-left (the popup container owns the top-right) with a loud colour and a high z-index. The text is roughly “TEST WORLD · server :PORT · <conn state>“.
Machine-readable part: a stable element id="e2e-test-world" with data-server-port and data-conn. The latter is the lowercased Debug name of CONN_STATE, kept reactive.

data-conn gives the harness a deterministic “socket is up” signal. Nothing else changes behaviour in the web client.

5. Harness layout

All of this goes in e2e_web_tests/ at the repo root.

requirements.txt has pytest>=7.0.0 and playwright>=1.48. Version 1.48 added WebSocket routing, which the guard needs. Do not install pytest-playwright. Its own page and browser fixtures and CLI flags would fight ours.
pytest.ini is identical to Android’s.
.gitignore covers .venv/, .build/, and artifacts/.
README.md covers:
prerequisites: Python 3.10+, venv, pip install -r requirements.txt, playwright install chromium, the Rust toolchain with the wasm32 target, and trunk
how to run, including headless and -k
the env var table
how to read results and add a test
the list of production-code changes made for the harness
conftest.py, support/, tests/ and run-ui-tests.sh are described below.

run-ui-tests.sh. Mirror Android’s style: RUN_TS/RUN_DIR, a log() helper, colours, a PASS/FAIL banner, an artifacts/latest link, and exit with pytest’s status. Drop all adb and Appium parts.

Add ~/.cargo/bin to PATH if missing. AI shells don’t source profile files, which is the same reason Android has its nvm block.
Run preflight checks with clear messages and distinct exit codes: cargo, trunk, the wasm32-unknown-unknown target, python3. Unless E2E_HEADLESS is set, a display (DISPLAY or WAYLAND_DISPLAY) must exist. Fail fast with a hint rather than silently going headless.
Build unless SKIP_BUILD=1:
Build the server binary and cli_gui with cargo, logging to $RUN_DIR/build_server.log.
Build the web app with trunk build, run from the web_interface directory, with its output dir set to e2e_web_tests/.build/web_dist. Log to $RUN_DIR/build_web.log. The fixed location keeps each run from copying tens of MB of wasm into artifacts/.
On failure, print the log tail and exit non-zero.
Resolve binary paths by asking cargo metadata for the target directory and reading bin names from the Cargo.toml files. Don’t guess. Export ZAKKEN_SERVER_BIN, MOCK_CLIENT_BIN, WEB_DIST_DIR, and RUN_DIR. Pre-set values win.
Activate .venv if present. Run pytest tests/ --color=yes "$@" through tee into $RUN_DIR/pytest.log. The "$@" is how -k works, and Android’s script lacks it. Strip ANSI afterwards.

Env vars:

E2E_HEADLESS: unset or 0 means headed.
E2E_FAIL_HOLD_SECONDS: default 5, ignored when headless.
E2E_PING_TIMEOUT_SECONDS: default 10. Raise it when debugging.
SKIP_BUILD
RUN_DIR
ZAKKEN_SERVER_BIN, MOCK_CLIENT_BIN, WEB_DIST_DIR: if missing when pytest runs directly, fail with a message to use the script.

Fixtures (conftest.py):

Session scope:
run: the recorder, rooted at RUN_DIR (default ./artifacts/_manual_run), finalised in a finally.
webapp: the static server over WEB_DIST_DIR.
browser: sync_playwright() plus Chromium, headed unless E2E_HEADLESS.
Function scope, strictly chained so teardown runs page → mock client → server: test_server, then mock_client (depends on test_server), then page (depends on mock_client, browser, webapp, run), then app, a bundle of page, server, mock, and base URL. app registers itself on run so the recorder can collect diagnostics.
An autouse _begin_test marks test boundaries in the report.
A pytest_runtest_makereport hook stores outcomes on the item, so page teardown knows whether the test failed.

Per-file purpose:

support/server.py owns one world. start():
picks a free port (bind 127.0.0.1:0, read it, close)
makes a temp dir
spawns the server with --test-mode --db-path <tmp>/world.sqlite --port <p> --ping-timeout <n>
sets cwd to the temp dir, as a second belt against stray relative paths
sends stdout and stderr to RUN_DIR/server_<test>.log
Immediately start a daemon pinger thread that does GET /__test/ping every 2 s with a 1 s timeout, ignoring errors. Readiness is the first successful ping within 20 s. If the process exits first, raise with the log tail.
stop() halts the pinger, then terminate, wait 5 s, kill, and delete the temp dir. Tolerate an already-dead process.
Expose port, ws_url (ws://127.0.0.1:<port>/ws), is_alive(), and a log-tail helper.
If the server’s log() doesn’t reach stdout or stderr, then on failure only, after the process is stopped and before the temp dir is deleted, open the db read-only with stdlib sqlite3 and dump the last ~100 rows of the logs table to RUN_DIR/server_db_logs_<test>.txt.
All harness HTTP calls must bypass env proxies (an opener with an empty proxy map).
support/mock_client.py wraps the cli_gui subprocess:
It spawns cli_gui --plain --server-url <ws_url> with piped stdin and stdout, and stderr to RUN_DIR/mock_<test>.log.
A reader thread pushes stdout lines into a queue and appends them to that same log (prefix <, with sent commands prefixed >). Never block on readline in the main thread.
Wait for READY for up to 15 s, otherwise raise with the log tail.
command(text, timeout=10) writes a line and collects lines until <<END>>, raising on timeout or process death.
sync() and pages() (parse the JSON line) are the building blocks.
wait_for_page(title, timeout=20) loops sync then pages every ~0.5 s until some page’s first block text equals the title. On timeout it raises AssertionError listing the titles it did see and the last sync output.
stop() sends quit, closes stdin, waits 3 s, then kills.
support/webapp.py is a threaded static server on a free 127.0.0.1 port over the dist dir:
Unknown paths without a file extension fall back to index.html. The router has client routes like /setup. Paths with an extension that don’t exist get a real 404.
Force application/wasm for .wasm.
Send whatever extra headers the project’s normal dev flow sends. Check Trunk.toml for [serve] headers, and see Risk 1 on COOP/COEP.
Keep request logging quiet.
support/browser.py handles launching and per-test page creation:
Create a context with a fixed viewport (1280×800).
Attach collectors for console and pageerror.
Set default timeouts (10 s for actions, 30 s for first boot).
Install the websocket guard. Use context.route_web_socket with a predicate that matches every WebSocket URL except exactly ws://127.0.0.1:<test port>/ws.
The handler records the offending URL and closes the socket without ever calling connect_to_server. A routed socket that isn’t connected is mocked, so the real server never sees it.
The allowed socket isn’t intercepted at all.
Provide a “hold if failed” helper: if headed and the test failed, sleep E2E_FAIL_HOLD_SECONDS before closing. The pinger thread keeps the server alive during the hold.
support/runner.py is the UiTestRun equivalent. Copy Android’s structure: a step list, report.json and report.md rewritten after every step, and step() re-raising. Replace the adb/logcat diagnostics with web ones:
step(description, expected, action) snapshots the console, pageerror, and ws-violation counts, then runs the action.
On failure it records the URL, the first ~500 chars of body text, console errors, page errors, ws violations, and the tails of the server log and mock transcript. Then it rewrites the report and re-raises.
On success it still fails the step if, during the step, a page error occurred, a console error containing panicked at appeared, or a ws violation was recorded. This mirrors Android failing on GlobalPopup.
Group steps under per-test headings.
No screenshots, videos, or traces anywhere.
support/ui.py holds low-level primitives (locator helpers, the timeout constants) and nothing app-specific.
support/flows.py holds app flows and PREFIX = "__AI_AUTOTEST__". Flows are open_app, complete_setup, create_page, type_page_title, and wait_connected. The app was not visible to me. Read leptos_components/setup_page.rs, menu.rs, main_content.rs, and the block components to learn the real labels.
Prefer role, text, and placeholder locators.
For typing into blocks use real key presses (press_sequentially), not fill. The app’s diff batcher is built around real input events.
Add a data-testid only as a last resort, and list every one in your report.
tests/test_web_page_reaches_mock_client.py is the single test.
6. The happy-path test

Fixtures have already provided a fresh test server and a mock client logged in as mock_client. They have also provided the static web app and a fresh page with the guard installed. The test takes run and app, and every step is wrapped in run.step(description, expected, callable).

Open the app in the test world.
Go to <webapp>/?test_mode=1&server_port=<port>, always via 127.0.0.1.
Expected: boot finishes. The “Loading...” placeholder disappears and the app lands on /setup, since a fresh profile has no user.
Verify the world.
Expected: the #e2e-test-world indicator is visible and its data-server-port equals the test server’s port.
Complete first-run setup.
Enter a user name (web_e2e) and confirm.
Expected: the app reaches the page list.
If setup reloads the page, the indicator must still show the same port. That is the sessionStorage check.
Wait for the server link.
Expected: the indicator’s data-conn becomes connected within 20 s.
Create a page in the web UI and type the title __AI_AUTOTEST__<timestamp> into the title block.
Expected: the title is visible in the page view.
Mock client sees it.
Call mock.wait_for_page(title, timeout=20).
Expected: exactly one page whose first block equals the title.
Implicit end-of-step checks. The runner enforces these: no websocket other than the test world’s was attempted, no page error or panic, and the test server is still alive.

Use Playwright’s expect for web state and the mock polling loop for the cross-client assertion. Use no fixed sleeps anywhere except the failure hold.

7. Anything else the implementer needs

Build order and checkpoints. Verify each layer before starting the next.

Server.
Run it by hand with --test-mode --db-path /tmp/x/world.sqlite --port 3101.
Hit /__test/ping, then stop pinging. It must exit with code 42 about 10 s later.
Missing or default args must exit 2.
With no args it must behave as before: 0.0.0.0:3000, test_db.sqlite, and 404 on the ping route.
cargo test passes for config.rs.
cli_gui.
Pipe sync, pages, and quit into --plain against the hand-run server.
Confirm READY, the JSON line, and a sentinel after each command.
Web client.
Build with trunk build and serve the dist with anything that does SPA fallback.
Open it in a normal browser with ?test_mode=1&server_port=3101.
The banner should show the port and go connected after setup. The test server log should show the connection, and a real server on 3000, if running, should show nothing. A reload after setup must keep the port.
Spike (throwaway, not committed): headed Chromium via Playwright with a non-persistent context loads the built app and reaches /setup. This decides the Risk 1 branch before you write fixtures.
Harness proper and the test.

Rules of the road:

Always use 127.0.0.1, never localhost. The server binds IPv4 only, and OPFS needs a secure context, which 127.0.0.1 is.
Nothing in the harness may touch port 3000 or test_db.sqlite, and no test may assume a server exists on 3000.
Keep Android’s -x and -k behaviour, and the reserved __AI_AUTOTEST__ title prefix.

Debug map:

Symptom	Where to look
Page stuck on “Loading...”	OPFS or COOP/COEP (Risk 1)
data-conn never reaches connected	server_<test>.log; check the page’s actual ws URL against the port
Mock never sees the page	The mock_<test>.log transcript; look for “not logged in” in the server log (Risk 3)
Server dies mid-test	Exit code 42 means ping timeout, so the pinger thread stalled or the timeout is too low for the debugging you’re doing
ws-violation failures	Config persistence in test_config

Definition of done:

./run-ui-tests.sh passes headed, and E2E_HEADLESS=1 passes.
-k filtering works.
Temporarily breaking the final assertion produces a failing run. The browser stays about 5 s, then closes, and report.md shows the failing step with console, server, and mock tails.
kill -9 on pytest mid-test leaves no server_core or cli_gui process after about 15 s.
The real world is untouched. test_db.sqlite’s mtime is unchanged and nothing connected to 3000.
Both cargo workspaces and the web build still compile.

Report back, briefly:

The files you created or changed, grouped by area.
Any deviation from this plan, with the reason.
Which Risk 1 branch you took.
Anything flaky you saw.
The exact commands to run it.
8. Risks and unknowns
OPFS in a non-persistent Playwright context, and COOP/COEP.
I couldn’t confirm that OPFS, including createSyncAccessHandle, works in an off-the-record context. Chromium has an in-memory file-system path for incognito, but I couldn’t verify the sync-access-handle case.
If the app uses sqlite-wasm’s “opfs” VFS, it also needs a worker and SharedArrayBuffer, so the static server must send the same COOP/COEP headers the project’s normal dev flow sends.
The spike in Section 7 decides this. If the app won’t boot in a plain new_context(), switch to launch_persistent_context with a fresh temp user-data-dir per test, and make the browser function-scoped. Nothing else in the plan changes.
SetupPage is not visible to me.
Its labels and flow are unknown, and a reload after setup is likely. That is exactly why config persistence matters.
If setup behaves unexpectedly, such as asking for something beyond a name, report it instead of improvising.
Login race after connected.
data-conn proves the socket is open, not that login and initial sync finished. If the first letter is lost because it was sent before login, don’t paper over it with sleeps.
Inspect owner_of_interacting_with_server’s spawn_sender and the relogin-on-connect sequence. If a marker is needed, add one more data- attribute to the indicator and report it.
LiveForever::new(path) semantics.
I haven’t seen it. Confirm it accepts an absolute path and creates the file, and watch for WAL side files (all go in the temp dir).
Server log() destination. Unknown. Handled by the DB-table fallback in Section 5 if it doesn’t reach stdout or stderr.
Build specifics.
Binary and package names, the workspace target directory, where index.html and Trunk.toml live, and first-build tool downloads (trunk fetches wasm-bindgen) are all unverified. cargo metadata and the actual files are the source of truth.
The first build can be slow.
Editing semantics in blocks. I don’t know whether blocks are contenteditable or inputs, or how long the batcher debounces. Typing real keys and polling on the mock side covers both.
Console noise. If the “page error, panic, or ws violation fails the step” rule trips on benign startup output, narrow the rule rather than dropping it.
Playwright version. WebSocket routing needs 1.48 or later, which the requirements pin enforces.
Headed mode needs a display. The script checks and fails fast.

## Addendum: style adjustments (2026-10-30, after the coding_style.md pass)

Read `ZakkeN/coding_style.md` before writing anything. Where it conflicts with a file or type name used earlier in this document, the style doc wins. One exception: files that have a direct Android analog keep Android's names, because the plan mirrors `e2e_android_tests/`. That means `conftest.py`, `pytest.ini`, `requirements.txt`, `run-ui-tests.sh`, `README.md`, `support/runner.py`, `support/flows.py`, `support/ui.py`, and `support/browser.py` are unchanged. Everything new gets the long, descriptive naming. No `config`, `utils`, `helpers`, `manager`, `handler`, or `service`. If you have a better name in the same style, use it, but not a shorter one. Do not refactor existing code to match the style (for example, `main.rs` already creates channels itself). The style applies to what you add.

### 1. Server: `config.rs` and `test_mode.rs` are replaced

New files in `server_core/src/`:

- `main_for_reading_launch_arguments.rs` — syntax only. Returns a plain settings value. Unknown flag, missing value, or unparsable number exits 2.
- `test_mode_guard.rs` — `TestModeGuard` sits in front of "start as a test world". Holds every combination rule from Section 2:
  - `--ping-timeout` is legal only with `--test-mode`, and must be at least 1.
  - `--db-path` and `--port` must both be explicit.
  - Port can't be 3000; db file name can't be `test_db.sqlite`.
  - On any rule failing, refuses with a message and exit 2.

The name says what it guards. The conditions live in the code. `lib.rs` stays plumbing only.

### 2. Server: the watchdog becomes owner + handle with a doorbell

No shared clock, no polling. New folder `server_core/src/owner_of_test_world_lifetime/`:

- `mod.rs` — plumbing only.
- `core.rs` — holds `spawn_task_and_get_handles(ping_timeout)`. Creates the channel inside, spawns the owner task, returns one handle (`test_world_lifetime_handle`). Main never touches the channel.
- `route_that_answers_the_test_harness_pings.rs` — small router whose state is the handle. Handler rings the doorbell and answers 200. Merged into the main router only in test mode.

The handle has one caller-pov method, named along the lines of `tell_the_test_world_the_harness_is_still_alive()`. Internally it rings a doorbell: sends `()` down an unbounded channel. Name the channel ends in the rxtx style. Suggestions:

- `tx_tell_the_watchdog_the_harness_just_pinged`
- `rx_hear_the_harness_just_pinged_so_the_watchdog_stays_quiet`

The owner task is the watchdog. Its loop waits on the receiver with a timeout of `--ping-timeout`. A ring resets the wait. Silence for the full timeout writes one line to stderr and calls `std::process::exit(42)`. A closed channel counts as silence too, so a dropped handle can never quietly disable the dead-man protection. The wait starts when the task is spawned, so a harness that dies right after launch still leaves no orphan.

In `main.rs`, the new code reads as a table of contents: read arguments, run the guard, then in test mode spawn the owner and keep only the handle.

### 3. Mock client: plain mode gets its own file

Put the plain line-protocol loop in `cli_gui/src/main_for_plain_line_mode.rs`, so the interactive screen loop stays byte-for-byte unchanged. The only refactor allowed in `main.rs` is extracting the command match into one function that returns result lines, so both modes share it. Plain mode overrides only the `pages` output (the JSON line).

### 4. Web client

- `test_config.rs` becomes `main_for_reading_and_remembering_test_world_settings.rs`. Its header comment is the map of who touches it: `main()` writes it once at startup; `build_ws_url` and the banner read it. If `web_interface/src/shared_owner/` exists, put it there. If it doesn't, don't create the folder, but keep the map in the header comment.
- Put the banner in `leptos_components/small_components/banner_that_says_this_is_a_test_world.rs`. The component is `TestWorldBanner`.
- If `CONN_STATE` already has a shared-owner map, add the banner as a reader.
- `server_link/` and `identity.rs` are slated for renames in `coding_style.md` (`owner_of_interacting_with_server/`, `main_for_login_and_registration.rs`). Look at what's on disk and use those paths. Do not do the renames yourself.

### 5. Harness: new files in the style

- `support/server.py` → `support/test_world_server_wrapper.py`, class `TestWorldServerWrapper`.
- `support/mock_client.py` → `support/mock_client_process_wrapper.py`.
- `support/webapp.py` → `support/static_web_app_server_wrapper.py`.
- The websocket guard moves out of `browser.py` into `support/test_world_websocket_guard.py`. It sits in front of every websocket the page tries to open and refuses any not headed to this test world.
- The `app` fixture is renamed `world`, a bundle of handles. Tests read `def test_...(run, world)`.
- The test file and function names describe the whole scenario, for example `test_page_created_in_web_client_reaches_mock_client_through_test_server`.

### 6. Comments and docs

Every new comment is written in the author's voice: plain prose, conversational, explaining why. Tell the story of the surprising part, like the Samsung-keyboard note in `ui.py` or "Created at birth, handed to the telephone at birth. Never moves." in `socket.rs`. Don't narrate what a line does. Every new file with a caller gets a short header with two labelled parts: caller pov (what a user of this file needs to know) and internals pov (only if it helps someone changing it).

### 7. Report back

Add one item to the final report: any place where `coding_style.md` and this plan disagreed, and which you followed.
