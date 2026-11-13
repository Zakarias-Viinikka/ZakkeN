# TODO

## status

2026-11-04: preplan came back and was amended with the human's answers. Lives at ZakkeN/combining_connecting_template_with_cqrs_and_server.md. Started implementing: new `drift_guard_and_ops/` workspace at the repo root, first member `client/`, holding `ConnectionStatus`. Decisions made during the port are in `connecting_to_server_template/decisions_made_when_implementing_in_real_project.md`.

## immediate


### yrs: edit_text_block counts chars() but Doc uses byte offsets

`TextEdit.Delete` and `TextEdit.Replace` measure the delete range with
`text.chars().count()`, but `Doc::new()` defaults to byte offsets.
Non-ASCII text (å/ä/ö/emoji) inside a Delete or Replace range can
misplace the edit.

Reproduced by `ops.unicode_edit` in
`android_app/app/src/main/java/z/zndroid/Tests/OpsChecks.kt`. Removed
from `DevCheckRegistry.checks` so it doesn't block the Android e2e
suite. Re-add it once this is fixed.


## audit code and refactor

### cqrs and server audit findings (2026-10-31)

Three-pass audit by a separate AI, covering `web_interface/src/`, the drag black box, `yrs`, the builder and executor crates, `zakken_server`, `text_diff`, `smart_frac_index`, `love_letter`, and `client_table_blueprints`. Full findings in `2026_10_31_cqrs_and_server_audit_findings.md`.

Some fixes are required. The audit's suggested shortlist, in priority order:

1. `get_diff` loses text when a selection is replaced by text of a different length (third pass — not in any existing docs)
2. `Delete` combine position semantics are wrong (third pass — rare trigger, corrupting when it hits)
3. Text offsets: `Insert.position` is affected too, not just `Delete`/`Replace` (confirmed worse than the todo said)
4. First-run server link never starting
5. Drag / stale `page.blocks`
6. `apply_blob` returning `changed = false` for non-open pages, so menu titles don't refresh
7. A failed transaction commit leaving the transaction open
8. A rejected letter permanently blocking the send queue

The list in the doc is longer. Nothing acted on yet, and deliberately: the plan is to leave these bugs in place and write e2e tests for the user-visible entries, so that once the harness is set up, its first run failing on this list is proof the harness works. See the addendum in `e2e_webapp_server_mock.md` for which entries are testable that way. Non-testable entries get fixed later, on their own.

## server: edits that arrive for a block that's already deleted

If a client sends a love letter for a block the server has already seen
deleted, the edit currently gets dropped. Instead: hold it, and later
surface it to the user as "we synced this edit but its block was gone,
want to look?". Way down the line. Not started.

## planned

### Automated e2e testing from the template

The template (`connecting_to_server_template`) has a full state × action grid now — every sync mode, connection state, and verb combination has a defined behaviour. That grid should become automated tests, not just a manual hand-walk.

Shape: run the server, run the web app, click "Start tests" in the app, and let the tests drive the scenarios. Each row of the state table is a test case. Some are asserts against the visible state (border color, status line, panel readouts), some are assert-on-event (a popup appeared, a mode changed, an error is shown).

Then a shell script (or something similar) on the computer that runs the whole thing headlessly — start trunk serve, start the server, launch Playwright or similar, trigger the in-app test runner, wait, read the result, print pass/fail plus what went wrong for each failure.

Not started. Questions to work out later: where the test data lives, how tests reset between cases (reload? a reset endpoint on the fake server?), what "run the same tests on the real web_interface + zakken_server" looks like — presumably the same tests with different drivers, which is why the template has to have well-defined states in the first place.

### E2E web harness for web_interface + zakken_server

The Playwright-based plan in `e2e_webapp_server_mock.md`. Spawns a fresh test-mode server per test, drives the real Leptos client in Chromium with a websocket guard. Two independent browser processes act as the two clients — the mock client is dropped (see the plan's addendum). Covers the small production changes to `zakken_server` and `web_interface` needed to make it testable. Once running, the harness is meant to fail on the user-visible entries from `2026_10_31_cqrs_and_server_audit_findings.md`, which is how we'll know the harness itself is working. Not started.

A smaller proof-of-shape exists at `rust_practice/e2e_template_for_web_with_server/` (2026-10-31). Three layers instead of two: a Rust `orchestrator` owns process lifecycle and spawns `pytest`; the `tester` (pytest + Playwright) only drives the browser; the `web_app` (Leptos) + `server` (axum) are under test. Working, with two passing tests. See its `STORED_AI_CONTEXT.md` and the addendum in `e2e_webapp_server_mock.md`.

### Android port of feel_labs

The plan in `android_app/docs/android_port_of_feel_labs.md`. A Kotlin + Compose port of `messing_around/feel_labs`, plus a `cascade_physics` Rust crate that owns the cascade stepper and layout for all three modes and drag-reorder. UniFFI, JNI-free, Build glue in Gradle. Not started.

Visual effect labs: see writing_effects_lab.md and new_page_popup.md.

Cache and preload: see cache_and_preload.md.

In the visual testing folder (browser_ui_testing or whatever I named it):

I want to make a project that uses my zap egui project as a reference for how to do what I want.

Basically I want to create a demo for: "create a menu system for reversing a yrs action and displaying the change in the same way zapgui does"

## diff viewer component (web_diff_viewer)

A Leptos CSR port of zap's egui diff viewer, living at `~/ProgStuff/ZakkeN/web_diff_viewer/`. Standalone project, not part of ZakkeN; worked on as a reusable component. Eventually the reconcile UI in the web app could use it.

Built. Workspace with five crates: `block_text_diff` (pure logic), `block_diff_viewer_leptos` (the `DiffViewer` component + pure helpers; owns id-carrying props and frac assignment), `example_app` (trunk demo with four sample pairs), `example_2` (prototype of the reconcile UI shell), `generate_love_letters` (pure: turn a server page + a user page into `LoveLetterSketch`s keyed by block_id). The sketch-to-letter half lives in `drift_guard_and_ops/client::reconcile_planner`. Native tests via `c t` at workspace root. `c r` serves the demo.

See the "External: web_diff_viewer" section in `STORED_AI_CONTEXT.md` for the caller contract, what the viewer does, and the deliberate deviations from the original plan.

## export / import / wipe / adopt / reconcile

User-triggered sync recovery. Plan in
plan_for_general_data_management_ops.md, superseded where they
disagree by combining_connecting_template_with_cqrs_and_server.md.
Covers: hard-copy JSON for pages and backlinks, dataset generation
id in the server's Login reply, new request variants for wipe and
adopt, and a per-page reconcile flow. Web first; Android deferred
but kept compatible.

Partly done. The wipe request exists and works server-side: it
unregisters everyone, mints a new drift version with the caller's
drift_solution, and tells every other connected socket before
closing it. Reconcile is built end to end (2026-11-11): four new
requests, a fingerprint helper, a client-side planner, and a /reconcile
route that lets a lonewolf client merge local and server data page by
page before wiping itself and adopting the server's dataset. See
nov11.md for the reconcile_version and fingerprint decisions.

Still not built: export, import, adopt. Automatic wipe on
`drift_solution = WipeSelfAndAdoptServer` at login. The picker for
when drift_solution is absent. Adopt as a second bump source. An e2e
test for the diff-viewer path (both sides have the same page id with
different content).

## yrs: hard copy primitive

BossOfYrs::from_hard_copy was the original name. It landed as
`freeze_doc(input: FreezeDocInput) -> BossOfYrs` in
`yrs/src/yrs_wrapper/freeze_doc_main/`, with
`freeze_doc_builder(boss) -> FreezeDocInput` for the read-back.
File order is the order; fracs come from the rebuilt doc. Tests in
`yrs/tests/freeze_doc_tests.rs`. Both client and server will use it
for the export / import / adopt / reconcile flow.

## mock client: deliberate editing

The mock client makes random destructive edits, so it can only test so
much before it corrupts the state it's testing against. It should
become a way to edit documents from the terminal deliberately instead.
Future work, not started.

## E2E android testing - done, ongoing

Built. See `e2e_android_tests/` and the section in `STORED_AI_CONTEXT.md`
(section "E2E Android testing - 2026-10-17"). Android AI runs the suite
itself, reads the terminal output, and fixes the app. Tests that fail
because the test is wrong get reported to the human - the AI doesn't
edit the test tree.

Two things still open in the harness:
- Screenshot/XML artifacts were removed for speed. If diagnosing a
  visual-only regression becomes hard, the artifact-writing code is
  still present in `support/runner.py` - just re-add the calls.
- The `CHECK_SYNC` backend assertion only fires in tests that call it.
  Consider adding it to more behavioral tests as they're written.

The original wish ("visual test system", "AI writes the tests") is what
exists now. Playwright-the-library is web-only; this is the Android
equivalent built from Appium + adb, not a port of Playwright itself.

## android: double space opens the special menu

On Android the special menu opens on two spaces in a row, and pressing space again closes it. (Web uses z+space / space+z and esc or space again to close.) Not started.

## server: test mode (roll-back-everything)

For e2e testing a client against a real server without touching real
server state. Idea: the server can be started in, or switched into, a
mode where it holds one long transaction open (or wraps the whole
session in one), accepts all the normal requests, and rolls the
transaction back at teardown. Same shape as the app's
`DbManager.beginExternalTransaction()` hook — every write the server
makes during the run ends up discarded at the end.

Not started. Details open: whether it's a startup flag, a request on
the wire, or both; how the rollback is triggered (a request, a signal,
a timeout, connection close); what happens to connected-but-untracked
clients while test mode is on.

Depends on: nothing yet, but the shape of "server-side transaction I
can hold open" is the same primitive the export/import/reconcile flow
needs, so worth designing both together when either lands.

- **db needs one owner.** Right now every caller reaches into the global `db_helper` functions directly. Nothing owns the db. Fix it with the owner+handle pattern from `coding_style.md`: a task owns it, callers get a handle.
- If I wipe server, then reset the web app. The server shouldn't give error "registered name already taken".
- If I have an empty web app, and another client pushes changes. It shouldn't give a "start over fresh" error.
- Planned e2e tests are in `complete_e2e/tests_to_write.md`.
- Update `connecting_to_server_template` to match the current vision. It should get rid of `ConnState` (or whatever the connection-status enum was called there).
- Update the py tester in `rust_practice/`. Probably move `py_lab` there too.

## adopt racing with concurrent edits

Adopt replaces the server's dataset with the caller's. Other clients
may be sending edits at the same moment. The queue is serialized, so
that's not the problem — the problem is that a queued edit can be
acked and then wiped by an adopt that runs after it.

Concrete case: client A sends an edit, the server accepts it and
acks it, A deletes its local pending row. Then client B adopts.
Adopt wipes every table and installs B's data. A's edit is gone from
the server, but A thinks it's synced. A reconnects, sees the drift
version changed, goes to lonewolf. The only lonewolf exit today is
wipe_local_and_pull_from_server, which throws away A's local data,
so A's edit is lost.

The other ordering is fine. If the adopt runs first, A's letter hits
the drift check and is refused with DriftVersionMismatch, so it never
gets acked. A keeps the edit in its queue.

The fix is A's side, not the server's. Once reconcile exists, A
reconnects and reconciles its pages against the new dataset, keeping
the edit. Until then, the first ordering is data loss.

Not urgent until adopt is built. Written down so the adopt plan
accounts for it.

## OfflineDeliberately — parked

New `ConnState` variant for "offline on purpose". Same as Offline from
the UI's point of view but the reconnect task skips it. Three pieces:

- Add the variant to `ConnectionStatus`.
- Add a guard in `should_i_reconnect` to treat it as "don't poll".
- Add a "go online" exit — either a matching method or a reload.

Won't persist across reload; a reload always re-enables reconnect.
Accepted because a real user going offline on purpose is rare enough
that a reload as the exit is fine. Test scenarios that need "offline"
should kill the server instead.

Not started. See `nov11.md` for the design note.

### Research known bugs from similar apps

Adjacent apps share the same fundamental problems — CRDT sync, offline editing, block-based editing, drag reorder, search, backlinks, import/export. Notion, Obsidian, Logseq, AnyType, Roam, and the various open-source CRDT projects all have public bug trackers, GitHub issues, and engineering post-mortems. Research the bugs and complaints that are architecturally relevant to this app, then write tests that reproduce the user action. If the test passes, the app handles it. If it fails, a real bug was found before a real user could hit it.

The goal isn't to fix every bug or to achieve coverage. It's to widen the categories of bug I know to look for. The audit list in `2026_10_31_cqrs_and_server_audit_findings.md` is a list of bugs I already know about; this is a strategy for finding the ones I don't.

Not started. Filter (relevance to my architecture) is the hard part, and the answer to "what's relevant" is a function of how well I understand my own code.
