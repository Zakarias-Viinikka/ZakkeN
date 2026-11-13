# nov12claude.md: e2e test progress (written by Claude, 2026-11-12)

Purpose: a running log so a fresh AI can pick up e2e test work where the last one stopped.
Read first: complete_e2e/STORED_AI_CONTEXT.md, complete_e2e/how_to_write_tests.md, complete_e2e/tests_to_write.md.
Run tests: `c r` from complete_e2e (or `c r -- -k <name>` to run one test).
Rules that matter: no padded timeouts (support/timeouts.py), find blocks by text not position,
narration messages must start with a digit, "Setup", "Step" or "TEST", stage tests call stage.close() last.

## Key finding about the old plan

tests_to_write.md tests 4, 5, 6, 7, 8 were written when "leave lonewolf" meant "drain the queued letters".
Reconcile as built does NOT drain. It:
  1. fingerprints local pages vs server pages (block texts only),
  2. shows one card per page that differs (client-only pages get no card),
  3. sends ONE ReconcileLetters batch (diff letters made on the client),
  4. wipes the client locally and pulls the server dataset.
So queue-count and "each edit exactly once" assertions (tests 4, 6, 7) do not apply as written. They need
rewriting around reconcile: what the server and the other client end up with after Finish.
Test 5 (incoming while stopped) still makes sense: after Finish the lonewolf client should hold everything the
other client did, because it re-pulls the server dataset.

## Status

- [x] test_006_reconcile/test_reconcile_client_only.py (existed, passes per STORED_AI_CONTEXT)
- [x] WRITTEN, NOT YET RUN: test_006_reconcile/test_reconcile_mismatched_page.py
      - test_reconcile_use_mine: Edge lonewolf, edits title of "Grocery list" to "Grocery list v2", picks "use mine",
        Finish. Expect: Edge and Brave both end up with the new title, server still has 3 pages, no error popups.
      - test_reconcile_use_server: same setup, picks "use server's". Expect: original titles on both, no new letters at server.
- [ ] Merged path (open diff overlay, edit right side, Done). Needs a look at web_diff_viewer before writing.
- [ ] Mismatched + client-only in one reconcile.
- [ ] Test 11 (fresh client, no stored drift_version, login does not go lonewolf). Probably unblocked now, uses read_client_state + read_conn_state.
- [ ] Test 10 (popup-once flag). Earlier skipped because the app did not use lonewolf; it does now. Re-check.
- [ ] Test 2 (enter_lonewolf from every start status). The synthetic "offline" part needs to know what the app listens to.
- [ ] Tests 4, 6, 7 rewritten around reconcile (see key finding).
- [ ] Test 5 around reconcile. Test 8 (gate vs bootstrap), test 9 (storm).

## Files touched so far

- NEW  /home/zakke/ProgStuff/ZakkeN/nov12claude.md
- NEW  complete_e2e/tester/tests/test_006_reconcile/test_reconcile_mismatched_page.py
- EDIT complete_e2e/tester/support/reconcile.py (appended: only_card, pick_on_only_card, go_to_menu)
- EDIT complete_e2e/tester/support/test_events.py (appended: enter_lonewolf)

## Assumptions in the new tests (check these first if they fail)

- Reconcile cards have no testids. Tests use `.rp-card`, `.rp-card-badge`, and button names "use mine" / "use server's".
- Badge text is matched case-insensitively against "mine" / "server" (resolve_choice.rs not read).
- enter_lonewolf (window event) keeps the state Lonewolf. If it decays back to Connected, the "Setup 2" wait or the reconcile page will show it.
- read_titles reads titles from the local db, so waiting for "Grocery list v2" in it means the edit was flushed before /reconcile is opened.
- Only the title block of "Grocery list" differs, so exactly one card is expected.
- Not asserted: server letter table counts after reconcile, because it is unknown whether ReconcileLetters stores rows in love_letters_from_clients.
  The use_server test does assert the count is unchanged (it should be: nothing is sent).

## Wanted later: testids on reconcile page elements (mismatched_card.rs, ready_view.rs) so selectors survive UI changes.

## Update 2 (same day, batch 2)

- [x] FIXED typo: the Rust variant is `ConnState::LonewolfAndOffline`. Old tests said `LoneWolfAndOffline`
      (never matched; worked only because "Lonewolf" was also in the tuple). New shared constant `LONEWOLF`
      in support/drift.py. tests_to_write.md still has the old spelling.
- [x] WRITTEN, NOT RUN: tests/test_007_drift/test_fresh_client_no_lonewolf.py (test 11).
      Both browsers reset, re-register, must reach Connected (never lonewolf) and store the same drift_version.
      Does NOT compare to the server's own drift row (needs tbl_drift_version.rs for table and column names).
- [x] WRITTEN, NOT RUN: tests/test_007_drift/test_enter_lonewolf.py (test 2, partial).
      Covers: enter_lonewolf -> Lonewolf and stays; a second enter_lonewolf -> still Lonewolf; an edit while
      lonewolf sends nothing within DEFAULT_SECS and Brave doesn't see it.
      NOT covered: reload + synthetic offline event -> LonewolfAndOffline (need to know what the app listens to).
- [ ] Test 10 (popup-once flag): needs the key_value storage table name and the popup text in the app.
- [ ] Merged diff-overlay reconcile path. Needs web_diff_viewer's DiffViewer DOM (selectors) before writing.

### Files touched in batch 2
- NEW  complete_e2e/tester/support/drift.py (LONEWOLF, assert_conn_state_stays, wait_for_stored_drift_version)
- NEW  complete_e2e/tester/tests/test_007_drift/{__init__.py, test_fresh_client_no_lonewolf.py, test_enter_lonewolf.py}
- EDIT test_006_reconcile/test_reconcile_mismatched_page.py and test_reconcile_client_only.py (state-name fix)

### Unverified (Claude appended to or relied on files it never read)
- support/test_events.py: appended `enter_lonewolf(page)` without reading the file.
- support/narrate.py, speed.py, timeouts.py, speed_config.json were not read. The new test modules
  (test_reconcile_mismatched_page, test_fresh_client_no_lonewolf, test_enter_lonewolf) have no entry in
  speed_config.json. If `speed.multiplier_for` errors on unknown modules, add entries.
