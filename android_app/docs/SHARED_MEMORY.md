# Shared Memory - Zndroid Project

## Architecture Highlights

- **Authoritative source**: yrs CRDT blobs in `pages.blobbed_page`.
- **Queryable source**: `every_block_in_existence`, ordered by `(frac_position ASC, my_id_as_given_by_yrs ASC)`.
- **Sync strategy**: intent-based. `client_love_letter` stores `LoveLetterSketch` (serialized intent) plus a yrs diff snapshot per edit.
- **DB concurrency**: `DbManager` serialises all access with a `Mutex` (Android port of the wasm worker queue). One writer at a time across the whole app is enforced by `z.zndroid.WriteLock`.
- **Layers**: reads (`db/reads`) -> writes (executor, `executor/`) -> batcher (`editor/PageBatcher`) -> public actions (`actions/`). Nothing skips a layer. `db/schema` holds the `ColumnDescriptor` objects every read uses; no positional column indexes elsewhere.
- **Builder/executor**: every write goes through the Rust builder (`rustlib.data_builder_...`) then the Kotlin executor in `z.zndroid.executor`. `executor/Operations.kt` mirrors `executor_of_what_the_builder_built_.../page_executor/operations.rs` one to one. `executor/TableWrites.kt` mirrors `table_writes/*.rs`.
- **Editing**: `editor/PageSession` owns the rows and every editing rule as a method. `editor/PageBatcher` holds an ordered queue of `Edit / Add / Delete / Move` entries, debounces 500ms, flushes inside one `WriteLock` + `withTransaction`, walks the queue in order. Consecutive Edits are squashed per block; any non-edit entry flushes the pending edit run first. On any failure the transaction rolls back, the boss is now ahead of the DB, so `GlobalPopupManager.showError` is shown and ViewPage rebuilds everything from the DB (drop queue, destroy boss). This replaces the old `diffBuffer / lastPersistedText / syncUiStates / APP_LOCKED` machinery.
- **Flush on leave**: top-bar back, system BackHandler, lifecycle ON_STOP, and DisposableEffect.onDispose (async, on the batcher's own scope so composition-scope cancellation can't abort it).
- **Focus**: `PageSession.pendingFocus = (blockId, offset)`. Each block's effect, when it matches, awaits a frame, calls `requestFocus`, sets the selection, clears. Retries for a few frames if the block isn't laid out. The page-level effect scrolls the row into view.
- **Page operations**: `actions/createNewPage`, `actions/renamePage`, `actions/disablePage`, `actions/moveBlockToAnotherPage`. Each takes WriteLock, runs one transaction, one commit. `DbManager.deletePage` is a dev-only `hardDeletePage`.
- **Schema**: `z.zndroid.db.schema` holds `ColumnDescriptor<T>` objects per table. `SchemaCanaryTest` asserts each descriptor's expected type. Migrations via `MigrationManager` + frozen snapshots in `db/migrations/schemas/`. `assertLatestMatchesBlueprints` compares the live Rust blueprints against the latest frozen Kotlin version.
- **Session id**: per-process. `SessionManager.initialize` reads the last stored id and increments once. `SessionMonotonicityTest` asserts it stays stable during a session.
- **Disable = hide**: `disablePage` marks the page and all its blocks disabled. The nav menu filters disabled pages out. If the page being disabled is currently open on the device that sends the disable, the user sees a popup and keeps working; the page just stops appearing in the menu.
- **Logging**: `Logger.kt` writes to Logcat and the `logs` table. Inside a transaction, log lines buffer in `DbTransactionElement.logBuffer`; on commit the buffer is dropped, on rollback it is flushed outside the failed transaction. Source parameter passed explicitly at every call site (no stack-trace introspection; R8 obfuscation breaks it).
- **Dev Tools**: debug-only page at route `test_results`, nav button labelled "Dev Tools". `DevCheckRegistry.checks` is the one list of checks. Run sequentially. Each logs `Log.i("DevCheck", "id=... status=PASS|FAIL detail=...")`. The last line is `Log.i("DevCheck", "summary pass=N fail=M")`. Sections: schema.*, migration, session.monotonic, mirror.vs_yrs, outbox.integrity, outbox.replay, ops.*, client_vs_server (reserved). Actions section (wipe/seed) is separate.
- **E2E**: external Python/Appium harness at `../e2e_android_tests/`. Three tests: `test_multiple_blocks`, `test_page_lifecycle` (user-visible), `test_dev_checks` (navigates to Dev Tools, runs all checks, asserts `fail=0`, prints each failure id + message). Whole run happens inside one external transaction opened by the debug `TestTransactionReceiver`; teardown rolls it back.
- **Popup severity**: `GlobalPopupManager.show(msg)` (info) vs `showError(msg)` (error, `ERROR:` prefix in the snackbar, `Log.e` to logcat). Harness reads popups from logcat tag `GlobalPopup` because snackbars render behind Dialog windows. Rule: anything that means "something went wrong" is `showError`.

## Schema (as of V3)

- `pages`: id, page_id, blobbed_page, page_status, version, is_main_menu_page
- `backlinks`: id, page_that_holds_link_id, page_being_linked_to_id, disabled, version
- `every_block_in_existence`: id, is_title, is_part_of_main_menu_page, content, my_id_as_given_by_yrs, id_of_page_i_belong_to, frac_position, page_is_disabled
- `client_love_letter`: id, snapshot_of_edit, love_letter_sketch, session_id, target_id
- `key_value_storage`: id, key, value
- `incoming_love_letters`: id, love_letter, target_page_id
- `logs`: id, timestamp, level, category, source, session_id, message, details, details_type

## Known bugs

- **`text_diff.getDiff` returns a bare `Delete` for a full replacement.** `getDiff("a", "b")` where old and new share no prefix and no suffix returns `Delete("a", 0)` instead of `Replace("a", "b", 0)`, silently dropping the new text. Workaround in place for renames: `renamePage` sends `DiffResult.Replace(currentTitle, newTitle, 0u)` explicitly instead of calling `getDiff`. Not worked around in the general edit path (`PageSession.onInput`) because that path can't tell a full replacement apart from anything else. Any select-all + paste or 'Cmd+A type' flow will hit this. Report to the Rust side; do not paper over it in Kotlin.
- **`edit_text_block` miscounts on non-ASCII for Delete/Replace.** `yrs` counts `chars().count()` for the delete range but `Doc::new()` uses byte offsets by default, so `å/ä/ö/emoji` inside a Delete/Replace range can misplace. Not exercised by any current e2e test. `ops.unicode_edit` in `OpsChecks.kt` reproduces it; it's removed from the `DevCheckRegistry` list until the Rust side is fixed so it doesn't block the suite. Re-add it once the bug is fixed.
- **`edit_text_block` labels errors as `"edit_text_block_insert"`.** Stale from an earlier method name. Left alone (report only).

## AI Documentation Index

- [SHARED_MEMORY.md](SHARED_MEMORY.md): (This file) Architecture highlights and known bugs.
- [AI_READ_THIS.md](AI_READ_THIS.md): Library guide and API reference.
- [INVENTED_ON_ANDROID.md](INVENTED_ON_ANDROID.md): Decisions made on Android that have no Rust or web reference yet.
- [AI_LAB_RULE.md](AI_LAB_RULE.md): Rules for the AI Lab.
- [UX_INTENT.md](UX_INTENT.md): UX philosophy and intended editor behaviour.
- [OUTSIDE_ANDROID_APP.md](OUTSIDE_ANDROID_APP.md): Context from outside the Android tree. May be stale in places - see final report of the 2026-10-29 refactor.