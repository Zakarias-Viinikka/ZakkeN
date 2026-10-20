# TODO

## audit code and refactor

## server: edits that arrive for a block that's already deleted

If a client sends a love letter for a block the server has already seen
deleted, the edit currently gets dropped. Instead: hold it, and later
surface it to the user as "we synced this edit but its block was gone,
want to look?". Way down the line. Not started.

## server: edits that arrive for a block that's already deleted

If a client sends a love letter for a block the server has already seen
deleted, the edit currently gets dropped. Instead: hold it, and later
surface it to the user as "we synced this edit but its block was gone,
want to look?". Way down the line. Not started.

## planned

Visual effect labs: see writing_effects_lab.md and new_page_popup.md.

Cache and preload: see cache_and_preload.md.

In the visual testing folder (browser_ui_testing or whatever I named it):

I want to make a project that uses my zap egui project as a reference for how to do what I want.

Basically I want to create a demo for: "create a menu system for reversing a yrs action and displaying the change in the same way zapgui does"

## mock client: deliberate editing

The mock client makes random destructive edits, so it can only test so
much before it corrupts the state it's testing against. It should
become a way to edit documents from the terminal deliberately instead.
Future work, not started.

## E2E android testing — done, ongoing

Built. See `e2e_android_tests/` and the section in `STORED_AI_CONTEXT.md`
(section "E2E Android testing — 2026-10-17"). Android AI runs the suite
itself, reads the terminal output, and fixes the app. Tests that fail
because the test is wrong get reported to the human — the AI doesn't
edit the test tree.

Two things still open in the harness:
- Screenshot/XML artifacts were removed for speed. If diagnosing a
  visual-only regression becomes hard, the artifact-writing code is
  still present in `support/runner.py` — just re-add the calls.
- The `CHECK_SYNC` backend assertion only fires in tests that call it.
  Consider adding it to more behavioral tests as they're written.

The original wish ("visual test system", "AI writes the tests") is what
exists now. Playwright-the-library is web-only; this is the Android
equivalent built from Appium + adb, not a port of Playwright itself.