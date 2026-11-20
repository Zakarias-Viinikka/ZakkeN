# 19nov_progress.md

Written by AI for AI. First thing a fresh AI reads after STORED_AI_CONTEXT.md. Update it after every meaningful step, IN PLACE (use replace or overwrite, never append). If code and this file disagree, code wins.

## CURRENT TASK (2026-11-20): port lab UI into android_app

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

NOTHING BUILT YET. Zero code files touched this session. Only docs: `LAB_WISHES.md` created, `UX_INTENT.md` got a pointer, this file rewritten.

### Planned order (AI's decision; lowest risk first)

- [x] 0. Baseline: `cd /home/zakke/ProgStuff/ZakkeN/android_app && ./build_and_install.sh` then `cd /home/zakke/ProgStuff/ZakkeN && ./e2e_android_tests/run-ui-tests.sh`. VERIFIED GREEN on 2026-11-20 (artifact 20261120_071440: 3 passed). Also fixes the pre-existing `ops.multiline_paste` failure by adding `computeDiff` to PageSession.kt (text_diff's `getDiff` returns Insert/Delete when it should return Replace on a no-shared-prefix replacement; `computeDiff` verifies the raw result and falls back to a codepoint prefix/suffix differ).
- [x] 1. Theme: `ui/theme/Motion.kt` (ported verbatim from lab), `ui/theme/FeelColors.kt` (FeelColors + DarkFeelColors + LocalFeelColors + LocalReducedMotion), `ui/theme/Theme.kt` rewritten (always dark, uses lab palette, maps onto Material3 scheme, reads animator scale for LocalReducedMotion). VERIFIED GREEN on 2026-11-20 (artifact 20261120_071440).
- [ ] 2. Popups: keep the `GlobalPopupManager.show/showError` API, replace the Snackbar UI with the lab toast stack. (implemented, not yet verified)
- [ ] 3. NewPagePopup: lab look, per-letter shake, placeholder fixed (use `mutableStateListOf`, simple reconcile). Keep e2e-visible text (see E2E section).
- [ ] 4. Settings page + cog; move dev buttons there; update flows.py `open_dev_tools`.
- [ ] 5. NavPage eye + drop animation after about 1.5 s; update flows.py delete flow.
- [ ] 6. Block animations: indent on first typed char, drop on delete.
- [ ] 7. Engine: add rapier2d_wrapper `.so` + `cascade` bindings, port CascadeController/CascadeColumn, replace ViewPage's LazyColumn. Highest risk, last.
- [ ] After each step: build, run e2e, update android_app/docs (AI_READ_THIS, SHARED_MEMORY, MISSING_AND_STALE, INVENTED_ON_ANDROID).

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
- E2E: `cd /home/zakke/ProgStuff/ZakkeN && ./e2e_android_tests/run-ui-tests.sh`
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