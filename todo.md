# TODO

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

The Playwright-based plan in `e2e_webapp_server_mock.md`. Spawns a fresh test-mode server per test, drives the real Leptos client in Chromium with a websocket guard, asserts through the mock client. Covers the small production changes to `zakken_server` and `web_interface` needed to make it testable. Not started.

### Android port of feel_labs

The plan in `android_app/docs/android_port_of_feel_labs.md`. A Kotlin + Compose port of `messing_around/feel_labs`, plus a `cascade_physics` Rust crate that owns the cascade stepper and layout for all three modes and drag-reorder. UniFFI, JNI-free, Build glue in Gradle. Not started.

Visual effect labs: see writing_effects_lab.md and new_page_popup.md.

Cache and preload: see cache_and_preload.md.

In the visual testing folder (browser_ui_testing or whatever I named it):

I want to make a project that uses my zap egui project as a reference for how to do what I want.

Basically I want to create a demo for: "create a menu system for reversing a yrs action and displaying the change in the same way zapgui does"

## export / import / wipe / adopt / reconcile

User-triggered sync recovery. Plan in
plan_for_general_data_management_ops.md.
Covers: hard-copy JSON for pages and backlinks, dataset generation
id in the server's Login reply, new request variants for wipe and
adopt, and a per-page reconcile flow. Web first; Android deferred
but kept compatible. Depends on the yrs hard copy primitive below.

Not started.

## yrs: hard copy primitive

BossOfYrs::from_hard_copy(blocks: Vec<HardCopyBlock>) -> Arc<BossOfYrs>.
Takes a plain block list (id, content, meta, frac_position) and
returns a fresh doc with those blocks, using
insert_new_block_with_preexisting_id so ids survive. No history,
no update log, fresh clocks. Both client and server need it for the
export / import / wipe / adopt flow. Needed fairly soon.

Not started.

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
