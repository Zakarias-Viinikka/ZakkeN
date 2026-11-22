# 19nov_progress.md

Written by AI for AI. First thing a fresh AI reads after STORED_AI_CONTEXT.md. Update it after every meaningful step, IN PLACE (use replace or overwrite, never append). If code and this file disagree, code wins.

## CURRENT TASK (2026-11-21): spike_for_markdown - blackboxed <MarkdownConversion /> component

Scope: `spike_for_markdown` (do NOT touch `cqrs_state_synchronization/web_interface` until requested).
Goal: Replace `<textarea>` in `cqrs_state_synchronization/web_interface/.../my_block_content.rs` with blackbox component `<MarkdownConversion />`.
Status: Component built and verified in `spike_for_markdown/api_example`. Props directly mirror `MyBlockContent`.

Zakke's workflow: AI interviews him in plain language, saves answers durably, then builds autonomously while he only runs commands. Interview is DONE. His verbatim answers and the interpreted spec are in `android_app/docs/LAB_WISHES.md` (read it). `android_app/docs/UX_INTENT.md` points to it.

### What Zakke wants (short form)

1. Drag-reorder/cascade engine integrated as an INVISIBLE foundation. No drag handle, no manual dragging. It exists so the animations below work.
2. Indent-style block transition when the user types the first character into an empty block (not on Enter).
3. Drop effect when a block is deleted.
4. New page popup + lab colours/look in the real app. Fix: the "Type a title" placeholder must disappear when the user types.
5. Main menu: eye replaces the X on each row. Drop animation plays about 1.5 s after the eye is pressed. The X today opens a "Delete Page" dialog and calls `disablePage` (hide, data stays), so eye = hide/disable page; the confirm dialog goes away.
6. Lab popups (toast stack) in the app.
7. Settings page behind a cog wheel in the main menu. All the "annoying buttons that don't belong in the menu" (Recovery, Database Inspector, Dev Tools; debug only) move there.

Not requested: drag-across-blocks multi-select, undo/redo, scroll feel.

### Status

ALL 7 STEPS BUILT 2026-11-20, all compile green (Zakke's output). Only steps 0, 1 and 2 were e2e-verified; steps 3 to 7 have NOT been run on a device or in e2e (Zakke's rule: build check only). REMAINING: (a) DONE 2026-11-20: docs pass for steps 1 to 7 written into android_app/docs (AI_READ_THIS "UI port from the lab", SHARED_MEMORY bullet, MISSING_AND_STALE "2026-11-20 lab UI port: open items", INVENTED_ON_ANDROID new entries); the "docs NOT updated" notes in the step lines above are resolved, (b) Zakke eyeballs the app on the phone and reports anything odd, the engine (7b) is the most likely to need tuning.

### Planned order (AI's decision; lowest risk first)

- [x] 0. Baseline: `cd /home/zakke/ProgStuff/ZakkeN/android_app && ./build_and_install.sh` then `cd /home/zakke/ProgStuff/ZakkeN && ./e2e_android_tests/run-ui-tests.sh`. VERIFIED GREEN on 2026-11-20 (artifact 20261120_071440: 3 passed). Also fixes the pre-existing `ops.multiline_paste` failure by adding `computeDiff` to PageSession.kt (text_diff's `getDiff` returns Insert/Delete when it should return Replace on a no-shared-prefix replacement; `computeDiff` verifies the raw result and falls back to a codepoint prefix/suffix differ).
- [x] 1. Theme: `ui/theme/Motion.kt` (ported verbatim from lab), `ui/theme/FeelColors.kt` (FeelColors + DarkFeelColors + LocalFeelColors + LocalReducedMotion), `ui/theme/Theme.kt` rewritten (always dark, uses lab palette, maps onto Material3 scheme, reads animator scale for LocalReducedMotion). VERIFIED GREEN on 2026-11-20 (artifact 20261120_071440).
- [x] 2. Popups (VERIFIED: Zakke reported e2e passing on 2026-11-20 after the change; the one earlier failure was the phone being off): `GlobalPopupManager.show/showError` API kept, Snackbar replaced with the lab toast stack in `components/GlobalPopup.kt`. Each toast is ONE a11y node, content-desc `popup|<error|info>|<message>` (clearAndSetSemantics), tap dismisses. Frame loop only runs while a toast is live. `e2e_android_tests/support/ui.py` popup helpers (`popup_present`, `get_popup`, `get_popup_message`, `dismiss_popup`) rewritten to read that content-desc (old "Dismiss" text is gone). Verified. Risk: if `runner.py` fails a step on ANY popup rather than only error popups, info toasts could now trip it. Check output.
- [x] 3. (BUILT 2026-11-20, build ASSUMED green per Zakke, no output pasted = not verified, e2e not run; new file `ui/theme/Primitives.kt` has Btn/BtnAccent/Label; NewPagePopup.kt rewritten with lab panel, per-letter shake, snapshot-list letters, same params, title text "New Page" not uppercased; docs AI_READ_THIS/SHARED_MEMORY/MISSING_AND_STALE/INVENTED_ON_ANDROID NOT yet updated for steps 1-3, do it when next reading them) NewPagePopup: lab look, per-letter shake, placeholder fixed (use `mutableStateListOf`, simple reconcile). Keep e2e-visible text (see E2E section).
- [x] 4. (BUILT 2026-11-20, build GREEN (Zakke said "green"), e2e not run. New `MainPages/SettingsPage.kt` (SettingsPage + CogButton, cog content-desc "Settings"), route "settings" in MainActivity, NavPage now takes onOpenSettings/onOpenRecovery and its debug text buttons are gone, Recovery/Database Inspector/Dev Tools rows live in the debug-only DEVELOPER section. flows.py: open_settings added, open_dev_tools taps cog first, go_back_from_dev_tools goes back twice. Docs for steps 1-4 still NOT updated: cat android_app/docs/{AI_READ_THIS,SHARED_MEMORY,MISSING_AND_STALE,INVENTED_ON_ANDROID}.md and update them in one pass.) Settings page + cog; move dev buttons there; update flows.py `open_dev_tools`.
- [x] 5. (BUILT 2026-11-20, build GREEN per Zakke, e2e not run.) NavPage eye + drop. New `MainPages/NavRow.kt` (NavRow + EyeButton; eye content-desc "Hide <title>"). NavPage.kt rewritten: X + "Delete Page" dialog removed, `onHide` runs `disablePage` AFTER the drop animation and refreshes; failure shows an error popup and the row springs back. Sequence: eye closes + row dims, 1.5 s delay, lift 200 ms, fall 300 ms (tilt 3 deg, fade from 135 ms), 120 ms pause, 120 ms collapse, then disablePage. Pressing the eye again before the drop cancels (Android invention). Leaving the menu during the 1.5 s wait cancels the hide silently (page stays visible, nothing changed). flows.py: `_hide_button`, `delete_page` and `cleanup_test_pages` tap the eye, wait up to HIDE_WAIT=10 s; `_row_sibling` and `rename_page` are now unused/stale. Not done: INVENTED_ON_ANDROID.md entry for the cancel-before-drop behaviour (docs pass pending).
- [x] 6. (BUILT 2026-11-20, build GREEN per Zakke, e2e not run.) Block animations. Indent: `BlockRow.indentGen` bumped in `PageSession.onInput` when a non-title row goes from empty to non-empty (NOT on Enter, NOT on remote apply, NOT on multiline paste); `EditableBlock` plays 22dp->0 / 500 ms / -5.5dp at 200 ms on the whole row via graphicsLayer; `indentPlayedGen` stops replay when the row scrolls back in. Drop: deleted rows are removed from `rows` and the DB delete is pushed IMMEDIATELY as before; a `GhostRow` (text + anchorBlockId = block above) is added to `PageSession.ghosts` and drawn by new `MainPages/ViewPage/GhostBlock.kt` (lab Drop: lift 200, fall 300 + 3deg tilt, fade from 135, 120 pause, 120 collapse, then `removeGhost`). `buildPageEntries` merges rows+ghosts for the LazyColumn (ghost placed after its anchor; orphan ghosts at the end; id collisions skipped). `EditableBlock` got `lazyIndex` (scroll uses merged index, session ops use rows index). Ghost is display-only, semantics cleared. Ghosts spawn for: block menu Del, Backspace on an empty block, Backspace at start of a block whose upper neighbour is empty. NOT for Backspace merge (text moved, not deleted). Known cosmetic: LazyColumn 8dp item spacing disappears in one jump when the ghost is removed. Docs pass for steps 1-6 still pending (INVENTED_ON_ANDROID: eye cancel-before-drop, ghost model, indent trigger; SHARED_MEMORY: PageSession.ghosts / BlockRow.indentGen).
- [x] 7a. BUILD GREEN (Zakke's output 2026-11-20; schema_drift_detector only checks libclient_table_blueprints.so so it ignores the new lib; no build.gradle.kts/keepRules change was needed for debug; the "Unchecked" note below is answered; release/minify NOT checked) (2026-11-20): copied `android_labs/app/src/main/jniLibs/arm64-v8a/librapier2d_wrapper.so` to `android_app/app/src/main/jniLibs/arm64-v8a/` and `android_labs/app/src/main/java/cascade/rapier2d_wrapper.kt` to `android_app/app/src/main/java/cascade/` (terminal cp, not zap). Ported verbatim to `z/zndroid/cascade/CascadeController.kt` and `CascadeColumn.kt` (package line only). NOTHING calls them yet. Unchecked: whether `schema_drift_detector` ("native lib freshness") rejects the new .so, so `build_and_install.sh` may fail even if gradle is green; and whether `app/build.gradle.kts` / `keepRules/rules.keep` need entries for the new lib.
- [x] 7b. BUILT 2026-11-20, build GREEN (Zakke's output), behaviour NOT seen by anyone on a device, e2e not run.
- [x] 8. (2026-11-21) VERIFIED ON DEVICE BY ZAKKE:
  - Enter places caret without lag: VERIFIED FIXED on device.
  - Backspace lands at join point: VERIFIED FIXED on device.
  - Rapier2D Test Page: Added `MainPages/Rapier2dTestPage.kt` (route `rapier_test` via Settings > DEVELOPER) to test CascadeColumn in isolation. Verified fine.
  - Full-screen block spacing: Fixed in `CascadeColumn.kt` by clamping `childConstraints.minHeight = 0` during child measurement so children wrap content instead of filling full screen height.
- [x] 9. (2026-11-21) OPEN ISSUES RESOLVED:
  - Issue 1 (Page vertical centering): FIXED. Replaced SelectionContainer with Column(verticalScroll) in ViewPage.kt. Blocks align to top.
  - Issue 2 (Holding Backspace across blocks): FIXED. In EditableBlock.kt, cleared pendingFocus immediately and requested focus without repeating text overrides. In PageScroll.kt, added viewport bounds check so scrolling only occurs when target block is outside/near visible edges, eliminating the 250ms animation stall when backspacing across adjacent visible blocks.
  - Issue 3 (Multi-block selection across blocks): REVERTED / NOT REQUESTED. BasicTextField does not participate in SelectionContainer across separate nodes. Removed SelectionContainer (was not in LAB_WISHES.md, which noted multi-block drag select was not requested). ViewPage uses Column(verticalScroll) { CascadeColumn }.
- [ ] After each step: build only (Zakke's rule, 2026-11-20: do NOT ask him to run UI/e2e tests; give him only the gradle build command), update android_app/docs (AI_READ_THIS, SHARED_MEMORY, MISSING_AND_STALE, INVENTED_ON_ANDROID).

### Still to read before coding

Not yet read: ZndroidTheme (`ui/theme`), `MainContainer` (components), `PageBatcher.kt`, `OpenPageRegistry.kt`, `android_app` tree (check `app/src/main/jniLibs` and existing UniFFI bindings), `e2e_android_tests/support/ui.py`.

## Lab facts (project is SEPARATE: /home/zakke/ProgStuff/ZakkeN/android_labs)

Package `com.example.feellabs`, source under `android_labs/app/src/main/java/com/example/feellabs/`. Foundation-only Compose (no Material3), minSdk 26, JVM 17. The old plan to "mount LabContainer in android_app" is OBSOLETE: integration means porting code. Lab docs: `android_labs/README.md`, `android_labs/STORED_AI_CONTEXT.md`.

- Theme: `theme/FeelTheme.kt` (`FeelColors` dark palette, `Motion` easings easeOut/easeSpring/easeFall, `LocalReducedMotion` from animator scale 0), `theme/Primitives.kt` (Btn, BtnAccent, Seg, Label which UPPERCASES its text, MockPage).
- Engine: `notes/CascadeController.kt` + `notes/CascadeColumn.kt`, backed by Rust crate `android_labs/rapier2d_wrapper` (UniFFI package `cascade`). Use ONLY `librapier2d_wrapper.so` (arm64-v8a, x86_64) and `cascade/rapier2d_wrapper.kt`; ignore the stale `libcascade_physics.so`/`cascade_physics.kt`. Built by `android_labs/build_android.sh`. JNA `5.16.0@aar` is already in android_app. Modes FALL/SPRING/RIPPLE (Zakke's choice not given; lab drag uses FALL). Rules: read `yState` only inside `graphicsLayer`/`offset {}`, wrap each block in `key(id)`, hoist `lastNanos` out of the inner frame loop, `set_height` skips reflow while anything is moving. Do not edit the Rust crate (rules); use the prebuilt outputs.
- Indent transition (`BlockTransitionLab`): translationX 22dp to 0 over 500 ms, keyframe -5.5dp at 200 ms.
- Delete drop (`DeletingBlockLab`, Drop variant): Marked phase lift 200 ms, then Leaving: translateY 40dp + rotate 3 deg (origin bottom-left) 300 ms with fade starting at 135 ms, then 120 ms delay and 120 ms height collapse, then remove.
- Menu (`MainMenuLab`): canvas-drawn EyeIcon (lid closes over 480 ms), PenIcon wiggle (pen not requested).
- New page popup (`NewPagePopupLab`): Dialog with scrim, panel enter 220 ms, per-letter shake 320 ms. Placeholder bug: `letters` is a plain `mutableListOf` (not snapshot state) so `letters.isEmpty()` never recomposes; the prefix/suffix reconcile also builds `keep` wrongly.
- Popups (`PopupsLab`): toast stack in a `Popup`, Info/Success/Warning/Error, virtual clock that pauses when a toast is held or a modal is open, tap opens a modal.
- Settings (`SettingsModalLab`): mock, fixed 700x560dp, no persistence. Too wide for a phone. Build a real full-screen page with a Developer section.

## android_app facts

- Material3 app, minSdk 31, JVM 11. `MainActivity` wraps the NavHost in `MainContainer`. Routes: nav_page, view_page/{id}, recovery_page, test_results (DevToolsPage, debug), db_gui, schema_viewer, inspect_table/{t}, admin_view/{id} (debug).
- NavPage: LazyColumn rows (clickable title + "X" TextButton), "+ New Page" button, debug-only text buttons Recovery / Database Inspector / Dev Tools, Lonewolf banner with "Fix" (stays on the menu). Callbacks: onOpenPage, onOpenDbInspector, onOpenTests, onOpenRecovery.
- NewPagePopup: Material Dialog + OutlinedTextField (label "Page Name"), title "New Page", buttons Cancel/Create, IME handling via `DialogWindowProvider`, inline error + `isSubmitting` guard. The "Type a title" bug exists only in the lab version.
- GlobalPopup: Snackbar host, `GlobalPopupManager.show/showError` over an unbounded channel, indefinite duration, "Dismiss" action. Keep the API.
- ViewPage: `LazyColumn` of `EditableBlock` keyed by blockId, `imePadding`, `PageSession` + `OpenPageRegistry`, flush on leave. EditableBlock: "::" box opens BlockActionMenu (H1/Todo placeholders, Del, Up, Down) and STAYS (it is not a drag handle); focus via `pendingFocus` + `animateScrollToItem`.
- PageSession (`editor/PageSession.kt`): `rows` is `mutableStateListOf<BlockRow>`; `BlockRow` has `blockId`, `isTitle`, `textFieldValue`, `focusRequester`. `onInput` is where typing is detected (hook for the indent animation: old text empty, new text non-empty, not title). Removals (`onDeleteBlock`, `onBackspaceAtStart`) remove the row immediately and push a batcher delete, so the drop animation needs a "leaving" state kept on the row BEFORE `rows.removeAt`, while the DB delete stays immediate or is deferred (decide when implementing; keep focus handling and IME behaviour intact).

## E2E harness constraints (e2e_android_tests, editable per rules)

`support/flows.py` finds things by visible `@text`. UI changes must keep these or update flows.py in the same step:
- "+ New Page" button on NavPage (`ensure_nav_page` waits on it).
- New page dialog: title text "New Page" (Lab `Label` uppercases, so render the title text exactly "New Page"), an `android.widget.EditText`, button text "Create". After Create the dialog must disappear and the new title must appear in the nav list. Per-letter rendering must not make the typed name unfindable in the nav (it is a plain Text there, fine).
- Row delete flow (`delete_page`, `cleanup_test_pages`): finds the "X" in the same row as the title, taps it, then waits for dialog "Delete Page" and taps "Delete", then waits for the title to vanish. Must change to the eye (needs a test-visible handle, e.g. contentDescription "Hide" or a testTag exposed as resource-id; check `support/ui.py`), with a longer wait for the 1.5 s drop animation. `rename_page` references a Rename button that no longer exists (stale, unused by current tests?).
- "Dev Tools" button on NavPage then "Run all checks" (`open_dev_tools`); back arrow text "←" (`go_back_from_dev_tools`). After the move, open_dev_tools must tap the cog first, then Dev Tools.
- Body placeholder text "Type '/' for commands..." and EditText fields for blocks (`add_body_block`). Blocks must stay EditTexts and the placeholder must stay.
- Popups are found by content-desc prefix `popup|` (see step 2), not by "Dismiss" text. Keep that format or update ui.py in the same step.
- Dev check summary comes from logcat `DevCheck` line `summary pass=N fail=M skip=K`; keep.

## Verification status (TRUTH)

Verified (Zakke reported "all green", full 3-test e2e: test_dev_checks + both flow tests):
- DevCheckStatus.SKIP, MirrorVsYrsCheck, OutboxReplayCheck changes.
- NavPage row redesign (plain-text clickable titles, HorizontalDivider, no Admin/Rename buttons, row button "X", flows.py `_row_sibling`).

NOT RUN: the whole 2026-11-19 editor/IME bugfix pass (PageBatcher, PageSession incl. computeDiff, OpenPageRegistry, ViewPage, EditableBlock, GlobalPopup, NewPagePopup, NavPage, OpsChecks.multilinePaste, DevCheckRegistry). Any earlier claim that e2e artifact `20261119_173245` passed is FALSE; that run never happened (Zakke confirmed). The last real run `20261119_172643` FAILED on `ops.multiline_paste` and predates `computeDiff`. Stays "not run" until Zakke pastes build + e2e output. That is step 0 above.

Not exercised by automation (compile/reading only): IME inset behaviour on ViewPage/GlobalPopup, inline dialog error rendering on DB failure, activity-recreation onDispose flush + awaitPendingClose race.

## How to work with Zakke

- He only runs commands given to him, in zap or his terminal. He does not read code, decide details, or debug. Keep replies short. Never ask him to edit manually.
- Update this file in place with replace or overwrite. NEVER append (he does not always run chains; stale appends duplicated content before). If unsure what is in the file, ask him to cat it first.
- Ask for files with one cat line, absolute paths, no cd. For trees, ask for `t` in one specific folder at a time.
- Update affected docs as work progresses: android_app/docs AI_READ_THIS.md, SHARED_MEMORY.md, MISSING_AND_STALE.md, INVENTED_ON_ANDROID.md. Write for an AI with zero memory.
- Allowed edits: everything under android_app/, and test folders (e2e_android_tests, complete_e2e, other test folders). Do NOT edit Rust crates, cqrs_state_synchronization, zakken_server, z_db, or android_labs/rapier2d_wrapper; report problems and stop. Paths in hidden.txt are off limits.
- Android mirrors the cqrs web app; deliberate differences go in INVENTED_ON_ANDROID.md.

## Making changes (zap)

- One chain per code block. Verbs: create, replace, overwrite, append, delete, zapmv, mkdir, rmdir, chmod +x, rename, astgrep rename. Write paths absolute or `~/`.
- Markers `@old`, `@new`, `@end` occupy whole lines; escape marker-like content with an extra @. `replace` needs exactly one match unless a count is given. `create` fails if the file exists; `overwrite` fails if it does not.
- On failure use retry (reruns the whole chain). Do not add build/run steps by default.

## Testing

- Build/install: `cd /home/zakke/ProgStuff/ZakkeN/android_app && ./build_and_install.sh`
- E2E: Zakke does NOT want to be asked to run it anymore (2026-11-20). Still keep flows.py/ui.py in sync with UI changes by reading, since nobody runs them per step. Mark UI-affecting steps "e2e not run" in this file.
- Build check only: `cd /home/zakke/ProgStuff/ZakkeN/android_app && ANDROID_HOME=/home/zakke/Android/Sdk ANDROID_SDK_ROOT=/home/zakke/Android/Sdk ./gradlew :app:assembleDebug` (plain `./gradlew` fails: no ANDROID_HOME in his shell and local.properties sdk.dir is wrong; build_and_install.sh sets it itself)
- Zakke pastes output. Never claim verified without output; mark unrun as not run.

## History: PageBatcher fix (2026-11-19), code complete

Android PageBatcher was rewritten to mirror the cqrs web batcher.

Five bugs fixed: (1) mid-flush cancellation could lose a batch (debounce now launches the flush in its own coroutine; cancelling only cancels the delay); (2) shared debounceJob race (now `synchronized(debounceLock)`); (3) cross-page move could run without flushing (registered source/target sessions flushed before `WriteLock`); (4) flushThen ignored the flush result and could double-fire onFailed (callback is now `onSuccess`, dispatched to Main); (5) applyRemoteRows reloaded needlessly (now updates rows in place on success, applies nothing on failure).

Android batcher (`editor/PageBatcher.kt`): one per PageSession; queue of Edit/Add/Delete/Move; 500 ms debounce; `flushNow()` cancels only the delay; flush = `WriteLock.withLock { DbManager.withTransaction { writeBatch } }`; success kicks the sender; failure reports an error, calls onFailed and reloads the page (in-memory Yrs boss is ahead of the rolled-back DB); consecutive edits to a block are coalesced.

Web reference: `cqrs_state_synchronization/web_interface/src/db/batcher/` and `complex_state/batcher_handle.rs` (global OnceLock handle, single run loop, Push/Flush messages, Delete/Move via run_exclusive, failure locks the app). Android deliberately differs (per-page batcher, batches Delete and Move too); documented in INVENTED_ON_ANDROID.md; do not silently remove.

Dev check fixes: DevCheckStatus gained SKIP (summary log is `summary pass=N fail=M skip=K`, harness regex is safe). MirrorVsYrsCheck no longer requires mirror size >= 2 (that is an editor invariant, not a mirror/yrs agreement). OutboxReplayCheck: empty outbox -> SKIP; only pages whose CreateNewPage letter is retained are replayed (outbox is a contiguous suffix of letter history because SendLoop deletes each sent row immediately, oldest first). `client_vs_server` reserved check is SKIP.

Last real failing run before the fixes: `e2e_android_tests/artifacts/20261119_154630/logcat.txt` (mirror.vs_yrs and outbox.replay failures, both invalid-check issues, now fixed and verified).

Rust crate note (report only, not edited): `text_diff/src/diff_logic.rs::get_diff` returns `Insert("hello", 0)` for "aX" -> "hello" (no shared prefix/suffix, longer new text) instead of Replace. Worked around on Android in `PageSession.computeDiff` (code-point prefix/suffix differ emitting `DiffResult.Replace`).

Open (low priority): whether the disabled one-block page `-15-15150354638807403063-1` is an intentional scratch result or an invalid fixture.

Files touched in the 2026-11-19 pass (under android_app/app/src/main/java/z/zndroid/ unless noted): editor/PageBatcher.kt, editor/PageSession.kt, editor/OpenPageRegistry.kt, sync/owner_of_interacting_with_server/ApplyLetter.kt, actions/MoveBlock.kt, Tests/{DevCheck, DevCheckRegistry, DevToolsPage, MirrorVsYrsCheck, OutboxReplayCheck, OpsChecks}.kt, MainPages/{NavPage, NewPagePopup}.kt, MainPages/ViewPage/{ViewPage, EditableBlock}.kt, components/GlobalPopup.kt, MainActivity.kt; plus android_app/docs/*.md and e2e_android_tests/support/flows.py.