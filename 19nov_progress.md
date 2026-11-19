# 19nov_progress.md

Written by AI for AI. First thing a fresh AI reads after STORED_AI_CONTEXT.md. Update it after every meaningful step. If code and this file disagree, code wins.

## How to work with Zakke

- Zakke only runs commands given to him, in zap or his terminal. He does not read code, make decisions about details, or debug.
- Keep replies short. Give one zap chain for changes and a separate terminal command for testing or inspection.
- Never ask him to perform manual edits.
- Update this file in place. Do not append repetitive progress logs. Keep one current status section and preserve useful historical information.
- Update affected Android docs as work progresses. The docs are AI_READ_THIS.md, SHARED_MEMORY.md, MISSING_AND_STALE.md, INVENTED_ON_ANDROID.md.
- Write docs for an AI with zero memory.
- Allowed edits: everything under android_app/, and test-related folders.
- Do not edit Rust crates, cqrs_state_synchronization, zakken_server, or z_db. Report problems there and stop.
- Android should mirror the cqrs web app. Deliberate Android differences belong in INVENTED_ON_ANDROID.md.
- Use `t` in one specific folder at a time when a tree is needed. Then request text files with one absolute-path cat command.
- Paths in hidden.txt are off limits.

## Making changes

- All changes use zap chains, in one code block.
- Verbs: create, replace, overwrite, append, delete, zapmv, mkdir, rmdir, chmod +x, rename, astgrep rename.
- Write paths must be absolute or start with ~/.
- Content markers `@old`, `@new`, `@end` occupy whole lines. Escape marker-like content with an extra @.
- replace requires exactly one match unless an explicit count is supplied.
- On failure, use retry. Retry reruns the entire chain; do not resend successful steps.
- Do not add build or run steps to zap chains by default.

## Testing

- Android build/install: `cd /home/zakke/ProgStuff/ZakkeN/android_app && ./build_and_install.sh`
- Android e2e: `cd /home/zakke/ProgStuff/ZakkeN && ./e2e_android_tests/run-ui-tests.sh`
- Zakke runs commands and pastes output.
- Never claim something verified without output showing it. Mark unrun tests as not run.

## Current task (2026-11-19)

Fix the Android PageBatcher to mirror the cqrs web batcher.

### Current status

- [x] Android batcher files read.
- [x] Web batcher files read and compared.
- [x] Five batcher issues identified.
- [x] PageBatcher rewritten.
- [x] PageSession.applyRemoteRows reworked.
- [x] ApplyLetter.refreshOpenPage updated for the suspend fetch-lambda contract.
- [x] MoveBlock.moveBlockToAnotherPage updated to flush registered source/target sessions before taking WriteLock. Its current scratch-test caller flushes its unregistered sessions explicitly.
- [x] Diagnostic logging added to DevCheckRegistry, MirrorVsYrsCheck, and OutboxReplayCheck.
- [x] Fixed diagnostic compile error: UniFFI exposes `Block`, not `BlockForReading`.
- [x] Android build succeeded.
- [x] APK installed on physical device R58M70AF1BY.
- [x] E2E run `20261119_154630` executed.
- [x] Logcat examined.

### Latest e2e result

12 dev checks passed; 2 failed. Pytest stops at the first failed test (`-x`), so the two user-visible flow tests were not reached.

1. `mirror.vs_yrs`: one failure across 27 pages. The only reported issue is page `-15-15150354638807403063-1`: Yrs and mirror both contain exactly one block. The check fails solely because it imposes `mirror.size >= 2`. The page is disabled. The logged counts agree; this is not evidence of mirror divergence.

2. `outbox.replay`: 27 failures across 27 pages. Logcat shows `client_love_letter` contained zero rows (`loaded 0 letters from 0 rows`). The check then attempts to replay nothing into empty documents and compares those against all existing page blobs. This is an invalid comparison when no retained letters are available. It does not demonstrate a PageBatcher failure.

3. `outbox.integrity` passed, reporting zero letters and no invalid targets.

Log source:
`/home/zakke/ProgStuff/ZakkeN/e2e_android_tests/artifacts/20261119_154630/logcat.txt`

### Next steps

- [x] Inspect DevCheckStatus and OpsChecks. DevCheckStatus is NOT_RUN/RUNNING/PASS/FAIL only; no SKIP exists. OpsChecks hard-deletes its scratch pages in finally blocks.
- [x] Inspect SendLoop. CONFIRMED in SendLoop.kt: on SendResult.Ok the outbox row is deleted immediately. Draining is oldest-first (id ASC) and stops on any failure, so the outbox is always a contiguous suffix of the full letter history. An empty outbox on a synced device is normal, not data loss.
- [x] Add SKIP to DevCheckStatus (+ skip() helper, DevToolsPage colour branch, skip count in the summary log line). Harness safety checked: flows.py regex is `summary pass=(\d+) fail=(\d+)` (search, not fullmatch) and test_dev_checks.py asserts only fail==0, so appending ` skip=N` and having SKIP results is safe. Also flip the reserved `client_vs_server` check from fake PASS to SKIP.
- [x] MirrorVsYrsCheck: remove the `mirror.size < 2` rule. The 2-block minimum is an editor behaviour invariant (guarded in PageSession, tested by ops.delete_rules), not a mirror/yrs agreement invariant; historical disabled pages can legitimately have 1 block. Keep every agreement check (counts, ids, order, text, title row, disabled flag).
- [x] OutboxReplayCheck: empty outbox -> SKIP with explicit message, never PASS. Non-empty: a page is replayable only if its CreateNewPage (genesis) letter is still retained. Soundness argument: the outbox is a suffix of history, and a retained genesis means it was never successfully sent, so the server never got the page and no other client can have edited it; complete history is therefore local. Replay = docFromSnapshot(genesis snapshot) + mergeWithSnapshot of later page letters in id order, compared to the stored blob. Pages without retained genesis count as pruned, not failed. Zero replayable pages -> SKIP.
- [x] Rebuild and rerun e2e.
- [x] Verify user-visible e2e tests also execute and pass.
- [x] Update AI_READ_THIS.md, SHARED_MEMORY.md, MISSING_AND_STALE.md and INVENTED_ON_ANDROID.md to reflect the final code.
- [x] Fix remaining UI/editor bugs from MISSING_AND_STALE.md (IME insets on ViewPage/GlobalPopup, multi-line paste + sound unequal-length diff in PageSession + EditableBlock Shift+Enter/selection.collapsed/canMoveDown guards, inline dialog errors + NewPagePopup submit guard, BossOfYrs/batcher cleanup after onDispose flush + awaitPendingClose + leavePage flush check, and ops.multiline_paste dev check).

### Verified prerequisites for the OutboxReplayCheck rewrite

- `LoveLetterSketch.CreateNewPage` exposes `pageId` in the Kotlin bindings (`sketch.rs` + `OutboxIntegrityCheck.kt`).
- `CreateNewPage`'s `snapshot_of_edit` is the initial page blob (`data_builder.../src/create_page.rs`).
- Cross-page move (`MoveBlock.kt`) emits `deleteBlock` (`RemoveBlock` for source page) and `addBlock` (`CreateNewBlockWithId` for target page), so per-page replay stays complete.

### Five batcher bugs and fixes

1. **Mid-flush cancellation could lose a batch.** `armDebounce` cancelled `debounceJob` on each push, which could cancel a running transaction after draining the queue. FIXED: debounce now launches the flush in its own coroutine. Cancelling the debounce only cancels the delay.

2. **Shared debounceJob race.** A plain `Job?` was accessed by multiple coroutines without synchronization. FIXED: reads and writes use `synchronized(debounceLock)`.

3. **Cross-page move might run without flushing.** FIXED in the second pass: registered sessions for source/target are flushed before taking `WriteLock`. The current scratch-test caller flushes its unregistered sessions explicitly.

4. **flushThen ignored flush result and could double-fire onFailed.** FIXED: callback renamed `onSuccess`, fires only on successful flush, and dispatches to Main before mutating Compose state.

5. **applyRemoteRows reloaded unnecessarily after successful flush.** FIXED: on success it updates rows in place, preserving focus and IME. On failure it does not apply remote rows; failure handling remains with the batcher.

### Web batcher reference

Source: `cqrs_state_synchronization/web_interface/src/db/batcher/` and `complex_state/batcher_handle.rs`.

- Global `OnceLock<BatcherHandle>` owns an `UnboundedSender<BatcherMsg>`.
- `BatcherMsg` is `Push(PendingWrite)` or `Flush { reply: oneshot }`.
- `PendingWrite` is Edit or AddBlock. Delete and Move use `run_exclusive`, which flushes first and then takes the write lock.
- One run-loop task owns the pending Vec. It selects between a fresh 500 ms timer and the next message.
- Flush drains the batch, takes `WRITE_LOCK`, and executes the entire batch in one transaction.
- Success calls `main_send_unsent_love_letters()`.
- Failure locks the app and reports data corruption. No retry or reload.
- Consecutive edits to one block are coalesced in first-occurrence order. AddBlock flushes the current edit run first.

### Android batcher

File: `android_app/app/src/main/java/z/zndroid/editor/PageBatcher.kt`

- One batcher per PageSession.
- Queue contains Edit, Add, Delete and Move.
- Debounce delay is 500 ms. The actual flush runs independently and is not cancelled by a later push.
- `flushNow()` cancels only the delay.
- Flush shape: `WriteLock.withLock { DbManager.withTransaction { writeBatch } }`.
- Successful flush kicks the sender.
- Failed flush reports an error, invokes onFailed and reloads the page because the in-memory Yrs boss is ahead of the rolled-back DB.
- Consecutive edits are coalesced. Same empty metadata and main-menu handling as web.

### Deliberate architecture differences

Web uses a global message-driven batcher with a single run loop. Android uses a per-page batcher with a queue and debounce coroutine.

Android additionally batches Delete and Move. These differences are documented in `INVENTED_ON_ANDROID.md`; do not silently remove them.

### Files touched so far

- `/home/zakke/ProgStuff/ZakkeN/19nov_progress.md`
- `android_app/app/src/main/java/z/zndroid/editor/PageBatcher.kt`
- `android_app/app/src/main/java/z/zndroid/editor/PageSession.kt`
- `android_app/app/src/main/java/z/zndroid/sync/owner_of_interacting_with_server/ApplyLetter.kt`
- `android_app/app/src/main/java/z/zndroid/actions/MoveBlock.kt`
- `android_app/app/src/main/java/z/zndroid/Tests/DevCheck.kt`
- `android_app/app/src/main/java/z/zndroid/Tests/DevCheckRegistry.kt`
- `android_app/app/src/main/java/z/zndroid/Tests/DevToolsPage.kt`
- `android_app/app/src/main/java/z/zndroid/Tests/MirrorVsYrsCheck.kt`
- `android_app/app/src/main/java/z/zndroid/Tests/OutboxReplayCheck.kt`
- `android_app/docs/AI_READ_THIS.md`
- `android_app/docs/SHARED_MEMORY.md`
- `android_app/docs/MISSING_AND_STALE.md`
- `android_app/docs/INVENTED_ON_ANDROID.md`
- `android_app/app/src/main/java/z/zndroid/MainPages/NavPage.kt`
- `android_app/app/src/main/java/z/zndroid/MainPages/NewPagePopup.kt`
- `android_app/app/src/main/java/z/zndroid/MainPages/ViewPage/ViewPage.kt`
- `android_app/app/src/main/java/z/zndroid/MainPages/ViewPage/EditableBlock.kt`
- `android_app/app/src/main/java/z/zndroid/editor/OpenPageRegistry.kt`
- `android_app/app/src/main/java/z/zndroid/components/GlobalPopup.kt`
- `android_app/app/src/main/java/z/zndroid/Tests/OpsChecks.kt`
- `android_app/docs/UX_INTENT.md`
- `android_app/app/src/main/java/z/zndroid/MainActivity.kt`
- `e2e_android_tests/support/flows.py`

### Notes on Rust crates (report only — not edited)

- `text_diff/src/diff_logic.rs::get_diff`: when `old_text.chars().count() < new_text.chars().count()` and the strings have no matching prefix/suffix (e.g. `"aX"` -> `"hello"`), `get_diff` returns `DiffResult::Insert("hello", 0)` instead of `Replace`, prepending `"hello"` onto `"aX"`. Exposed during `ops.multiline_paste` verification. Fixed on Android in `PageSession.computeDiff` using a Unicode code-point prefix/suffix differ that emits `DiffResult.Replace` when both middles are non-empty.

### Open questions

- Does the disabled one-block page come from an intentional scratch operation or an invalid fixture? Unresolved and low priority; it stops blocking the mirror check once the size rule is removed.
- ANSWERED: SendLoop deletes each successfully sent outbox row immediately (SendLoop.kt). Empty outbox after sync is expected behaviour.
- ANSWERED: DevCheckStatus had no SKIP; adding SKIP is the chosen not-applicable representation.

### Verification record

Verified:
- `./build_and_install.sh` and `./e2e_android_tests/run-ui-tests.sh` ran after the `DevCheckStatus.SKIP`, `MirrorVsYrsCheck`, and `OutboxReplayCheck` changes; Zakke reported "all green" (full 3-test e2e suite: `test_dev_checks` + both user-visible flow tests).

- `./build_and_install.sh` and `./e2e_android_tests/run-ui-tests.sh` ran after updating `NavPage.kt` (plain-text clickable titles, `HorizontalDivider` between rows, removed Admin and Rename buttons, changed row Delete button to `"X"`), `MainActivity.kt` (`onOpenAdminView` removed from `NavPage`), and `e2e_android_tests/support/flows.py` (`_row_sibling` / `"X"`); Zakke reported "all green" (full 3-test e2e suite).
- `./build_and_install.sh` and `./e2e_android_tests/run-ui-tests.sh` ran in artifact `20261119_173245` after the UI/editor bugfix pass (`PageBatcher`, `PageSession`, `OpenPageRegistry`, `ViewPage`, `EditableBlock`, `GlobalPopup`, `NewPagePopup`, `NavPage`, `OpsChecks.multilinePaste`, `DevCheckRegistry`); all 3 tests passed (`test_dev_checks` including `ops.multiline_paste`, `test_multiple_blocks`, `test_page_lifecycle`).

Not specifically exercised by the automated suite (verified by compilation/reading only):
- Visual IME keyboard inset behaviour on `ViewPage` / `GlobalPopup`.
- Inline dialog error rendering on DB failure inside `NewPagePopup` / `NavPage` delete dialog.
- Activity-recreation `onDispose` flush + `awaitPendingClose` timing race.
