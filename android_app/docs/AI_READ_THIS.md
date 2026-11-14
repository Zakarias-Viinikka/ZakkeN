ZakkeN Project - Library Guide for AI
This project uses Rust libraries exposed to Android via UniFFI. The main ones:

z_db: SQLite database wrapper (db_wrapper)
protocol: data types (records, enums) and serialization
client_table_blueprints: table schemas and row creation helpers
yrs: Yrs CRDT wrappers for page content and sync
love_letter: edit contract types and serialization for client-server communication
text_diff: character-level diffing and "squashing" logic for text editing

Kotlin generated bindings:
com.z_db.android_mascot (from db_wrapper)
uniffi.protocol (from protocol)
rustlib.client_table_blueprints (from client_table_blueprints)
rustlib.my_yrs_lib (from yrs)
rustlib.love_letter (from love_letter)
rustlib.text_diff (from text_diff)

Core database object: LiveForever
Methods: create_table, create_foreign_table, list_tables, get_data, get_data_ordered, insert_data, drop_table, edit_col_in_row, edit_col_in_row_where, check_table, delete_row, delete_row_where, begin_all_or_nothing, everything_went_perfectly, regret_everything, swap_columns, create_index, check_index, add_column, remove_column, export_database, export_tables, create_table_from_export, copy_table.

Use `awaitReady()` before calling DB methods in UI components.

Database Lifecycle & Schema Migrations:
All table schemas are managed via `z.zndroid.db.migrations.MigrationManager`. It executes a transactional walk-loop on initialization inside `DbManager.init(context)`. Version states are tracked inside the `SchemaVersion` enum (LATEST is V3; frozen snapshot files exist only for Version0-2, and V3 reuses Version2 for every_block_in_existence and Version1 for the rest) and persisted under the `schema_version` key in `key_value_storage`.

Table blueprints (client_table_blueprints - Note: Historic snapshots are frozen inside `z.zndroid.db.migrations.schemas` package):
pages: id, page_id, blobbed_page, page_status, version, is_main_menu_page
backlinks: id, page_that_holds_link_id, page_being_linked_to_id, disabled, version
every_block_in_existence: id, is_title, is_part_of_main_menu_page, content, my_id_as_given_by_yrs, id_of_page_i_belong_to, frac_position, page_is_disabled
client_love_letter: id, snapshot_of_edit, love_letter_sketch, session_id, target_id
key_value_storage: id, key, value
incoming_love_letters: id, love_letter, target_page_id
logs: id, timestamp, level, category, source, session_id, message (no details / details_type columns)

Column descriptors (Android's substitute for Rust's generic SchemaColumn<T>):
`z.zndroid.db.schema` holds one object per table (PagesTableSchema, EveryBlockInExistenceTableSchema, etc.). Each column is a `ColumnDescriptor<T>` with name, typeName, and unwrap(Col) -> T. SchemaCanaryTest asserts each descriptor's type is what's hardcoded.

Reads:
`z.zndroid.db.reads` is the main place that queries tables, and every read in it is descriptor-based. It is NOT the only place: StorageAccess, MigrationManager, DbManager (updatePageSnapshot, deleteRowsByColumn), SchemaVersion.V0, InspectTable and several Tests still read columns by position (see MISSING_AND_STALE.md). Most reads return Result<T>; isMainMenuPage and isPageDisabled return Boolean and throw on failure.
- `getPageBlob(pageId)`
- `isMainMenuPage(pageId)`
- `getActivePagesBlob(pageId)`
- `isPageDisabled(pageId)`
- `getBlocksForPageOrdered(pageId)` -> List<BlockMirrorRow>, ordered by (frac_position ASC, my_id_as_given_by_yrs ASC)
- `getMenuPages()` -> main-menu pages that aren't disabled

Table column creation functions:
newTablePages()
newTableClientLoveLetter()
newTableBacklinks()
newTableEveryBlockInExistence()
newTableKeyValueStorage()
newTableIncomingLoveLetters()
newTableLogs()

Row creation helpers:
newPageRow(pageId, isMainMenuPage, blobbedPage, version)
newBacklinkRow(pageThatHoldsLinkId, pageBeingLinkedToId)
newKeyValueItem(key, value)
newEveryBlockInExistenceRow(isTitle, isPartOfMainMenuPage, content, myIdAsGivenByYrs, idOfPageIBelongTo, fracPosition)
newLogRow(timestamp, level, category, source, sessionId, message)
newIncomingLoveLetterRow(loveLetter, targetPageId)

LoveLetter library:
LoveLetterSketch - describes the user's intent. Variants:
EditBlock { text_edit, edit_target, target_page_id, block_id }
CreateNewBlock { position_to_insert, target_page_id }
CreateNewBlockWithId { block_id, position_to_insert, target_page_id }
RemoveBlock { block_id, target_page_id }
MoveBlock { block_id, target_page_id, new_frac_position }
DisablePage { page_id }
CreateNewPage { page_id, is_main_menu_page }

Document operations:
Adding, editing, deleting, moving blocks and pages all go through the Rust builder and a Kotlin executor that mirrors it. There is no "DocEvents" package any more - AddBlock, EditTextInBlock, DeleteBlock and RemoveBlock are gone.

The layers are:
- builder (`rustlib.data_builder_...`): pure. Takes user intent, returns a ctx struct.
- executor (`z.zndroid.executor`): mirrors
  `executor_of_what_the_builder_built_.../page_executor` in Rust. One Kotlin function per operation, each calling `z.zndroid.executor.TableWrites` which mirrors `table_writes/*.rs`.
- actions (`z.zndroid.actions`): public entry points. `createNewPage`, `renamePage`, `disablePage`, `moveBlockToAnotherPage`. Each takes `WriteLock`, then one `DbManager.withTransaction`, then one committed write. Anything that means "something went wrong" uses `GlobalPopupManager.showError`.
- `z.zndroid.WriteLock` is a process-wide Mutex. Only one writer at a time. Batcher flushes, actions, and cross-page moves all take it. It is not re-entrant: nothing inside a locked section may take it again (that deadlocks).

Editor:
`z.zndroid.editor` holds the page-scoped editing machinery. There is no BlockUiState or diffBuffer any more.

- `PageSession` owns the rows (`mutableStateListOf<BlockRow>`), the batcher, and a `pendingFocus`. Every editing rule is a method here: `onInput`, `onEnter`, `onBackspaceAtStart`, `onDeleteBlock`, `onMoveUp`, `onMoveDown`, `onArrow`. The UI (`EditableBlock`) translates key events into these calls. The dev-page ops checks drive the same methods headlessly. That's why the same rules apply to keyboard, soft keyboard and headless calls.
- `PageBatcher` holds an ordered queue of `Entry.Edit / Add / Delete / Move`. `push*` appends and re-arms a 500ms debounce. `flushNow` drains the queue first (under a per-batcher flush mutex), then takes `WriteLock`, then runs ONE `DbManager.withTransaction` walking the entries in order. Consecutive Edits are grouped per block and squashed with `combineGetdiffResults`; any non-edit entry flushes the pending edit run first, then does its op. This mirrors `write_pending_in_order` in the Rust batcher.
- On any failure inside the transaction, `PageBatcher` calls `GlobalPopupManager.showError` and invokes `onFailed`, which makes ViewPage rebuild everything from the DB (destroy the old boss, build a new session). Only the failed batch is dropped: it was drained from the queue before the write. Edits pushed to the old batcher after that drain stay queued there and would flush later against the destroyed boss (from reading, not run). If the failure happens while leaving the page, the page is gone before the reload matters. This is Android's replacement for web's `APP_LOCKED`, because a phone can't tell the user to "refresh the page".
- `closeAndFlush` is just `flushNow`: it closes nothing, the batcher stays usable, and nothing ever cancels its scope. `closeAndFlushAsync`, used from `DisposableEffect.onDispose`, launches a flush on the batcher's own scope so composition-scope cancellation doesn't abort it. ViewPage calls `closeAndFlush` on top-bar back, system BackHandler and lifecycle ON_STOP, and ignores its Result. Nothing destroys the page's `BossOfYrs` when the page is left (only a reload does); the async flush still needs it.

Editor rules:
- Input: on text change (ignore selection-only changes) compute `getDiff(old, new)`, `pushEdit`, update the state optimistically.
- Enter (hardware Key.Enter or soft-keyboard '\n'): splits at the cursor (hardware Enter exactly; the soft-keyboard text path splits one character too far right unless the cursor is at the end of the block, see MISSING_AND_STALE.md). `head` stays, `tail` becomes a new block. If `head != old` a trim edit is queued first. New block id is 32 random hex chars. `pushAdd` with `Between(above = thisId, below = next row's id or null)`. Insert the row after this one, queue focus at position 0. Works in the title too (matches web's split-in-title).
- Backspace at position 0 with collapsed selection:
  - Title: consume, do nothing.
  - Above is the title: do nothing.
  - This block is empty: delete this block, focus the end of above.
  - Above is empty: delete above, keep cursor at 0.
  - Otherwise: append this block's text to above, delete this block, focus above at the join offset.
  This subsumes the "at least one body block" rule, so a page always keeps title + at least one body block.
- Arrow keys with a collapsed selection at the boundary: Down at end -> focus start of next block; Up at pos 0 -> focus end of previous block. Only Shift skips this. The selection is not checked for being collapsed (only selection.start is read).
- Move Up / Move Down via the block action menu (tap the `::` handle; it is a plain clickable, not a long-press). The menu's H1 and Todo buttons are placeholders that do nothing. Title can't move, nothing moves above it (the menu's Down button is still enabled on the title and does nothing). Uses the OPEN boss.
- Focus via `pendingFocus = (blockId, offset)`. Each block's effect, when it matches, awaits a frame, calls `requestFocus`, sets the selection, clears the request. Retries for a few frames if the block isn't laid out yet. Scrolls the row into view with `listState.animateScrollToItem(index)` from the same per-block effect (there is no page-level effect), so it only runs once that block is composed.
- Load: build the boss with `docFromSnapshot(blob, userId, pageId, now)`. Rows come from `getBlocksForPageOrdered`. If there's no non-title row, queue an empty body Add. If there are no rows at all, `showError`, and the page still opens with an empty list (the session is set anyway; there is no explicit missing-title check and no reload button). A fresh page (exactly two rows, the second empty) queues focus on the body block at position 0.
- Leave: always flush. Top-bar arrow, system BackHandler, lifecycle ON_STOP, DisposableEffect.onDispose (async).

Navigation:
- `createNewPage(name)`: one transaction. `createPage(true, sessionId)` builder, `insertPageRequiresThreeDbInserts`, load the boss from the fresh blob, `editBlock` on the title block with `getDiff("", name)`. Despite its name, `insertPageRequiresThreeDbInserts` does four inserts (page, title block, body block, love letter), so a new page emits two love letters (CreateNewPage, then EditBlock); web emits one.
- `renamePage(pageId, newTitle)`: finds the title block via the mirror (not via yrs ordering). Sends `Replace(currentTitle, newTitle, 0)` explicitly. This used to be a workaround for a `text_diff.getDiff` bug that no longer exists — the current `getDiff` returns `Replace` correctly. Sending `Replace` directly is still fine for a rename.
- `disablePage(pageId)`: skips if already disabled, reads the active-pages blob, `disablePage` builder, `disabledPageRequiresThreeDbActions`. Nav lists only enabled main-menu pages, in whatever order the mirror query returns (no sort anywhere). After a create or rename `NavPage` scrolls to the last row, which is timing-dependent (see MISSING_AND_STALE.md). The button label on the nav row is still "Delete" (frozen for the e2e harness); it means disable, not hard delete. If the page being disabled is the one currently open, the user gets a popup and keeps working.
- `hardDeletePage(pageId)` in DbManager is dev-only. Used by ops checks for scratch-page cleanup. It leaves the page's `client_love_letter` rows behind and takes no `WriteLock`. The dev page's "Wipe all page data" does not use it (it deletes rows from four tables directly).
- `page_id` is the stable identity and never changes during rename.
- Title is the content of the block whose mirror row has `is_title = "true"`. Not the first block in yrs order.
- Empty title blocks display "Untitled" in a muted colour in `NavPage`.

Session id:
`SessionManager.initialize` bumps the stored id once per process start (fire-and-forget, so `currentSessionId` is briefly still the old value). INTENDED: nothing else bumps it. ACTUAL (2026-11-13): `MainActivity` still calls `SessionManager.incrementAndStore()` on every navigation destination change, so the id also bumps per navigation (and once more at every launch, plus after each activity recreation). `SessionMonotonicityTest` only checks the id is >= 1 and unchanged over 100 ms, so it does not catch this. See MISSING_AND_STALE.md. The stored id may never be persisted at all: `setValue` takes the edit-existing-row branch for FastStorage keys and nothing inserts the `session_id` row (unverified; check `key_value_storage` in Database Inspector).

Logging (z.zndroid.log):
Modular system for background logging and database persistence.
Logger.kt: Engine using Dispatchers.IO. Writes to Logcat and `logs` SQLite table.
  - Outside transaction: inserts log directly into SQLite.
  - Inside transaction: buffers in `DbTransactionElement.logBuffer`. On commit, drops buffer. On rollback, flushes buffer to `logs` table outside failed transaction. CAVEAT: `Logger` finds the transaction through `DbManager.activeTransactionThreadLocal`, not the coroutine context, so after a thread switch inside a transaction it sees no transaction and writes the log directly (after the transaction ends). Under the e2e external transaction nothing is buffered at all.
  - Source parameter (file/class/function) is passed explicitly at call sites.
  - `Logger.e(details, detailsType)` and `LogEntry.details` / `detailsType` exist but are dropped: the `logs` table has no such columns.
ViewPageLogs.kt: Feature-specific helpers (logPageInit, logPageLoadSuccess, etc.).
LogDateHelpers.kt: Unix timestamp conversion for query ranges. Returns seconds, but `LogEntry.timestamp` is milliseconds; unused today.

AI Lab (z.zndroid.lab):
Experimental UX testing ground, isolated from production code.
Rules: See AI_LAB_RULE.md. Modular folder structure in experiments/.
Core: `LabNavigator` (lab/core/LabNavigator.kt: routes Production and ScrollLab, plus undo-bar and multi-select toggles) and `LabContainer` (lab/core/LabManager.kt: overlays and the smart button). Experiments: scroll, selection, undoredo.

Dev Tools page (only the nav button is gated by BuildConfig.DEBUG; the route and page compile into release):
Replaces the old "Internal Test Suite". Route is still `test_results`. The nav button label is "Dev Tools".
- `DevCheckRegistry.checks` is the one list of checks. `DevToolsPage` runs them sequentially.
- Each check logs `Log.i("DevCheck", "id=<id> status=<PASS|FAIL> detail=<first 200 chars of the check's one-line message>")`.
- The last line is `Log.i("DevCheck", "summary pass=N fail=M")`. The e2e harness reads that.
- Sections: schema.* (blueprints, canary, live DB), migration, session.monotonic, mirror.vs_yrs, outbox.integrity, outbox.replay, ops.*, client_vs_server (reserved, not implemented).
- Actions section (wipe / seed) is separate from "Run all". Not part of the pass/fail count. Wipe and seed run on a single tap with no confirmation.
- A check card runs that check when tapped while nothing is running; its detail expands only when tapped during a run. The check's real detail (`errorContext`) is on a second logcat line, `id=<id> st=<status> stack=<first 1500 chars>`; read failure detail with `adb logcat -d -s DevCheck:V`.

## Native libraries and bindings

`./update.sh` (in android_app/) copies the Rust build outputs into this tree: `.so` files into `app/src/main/jniLibs/arm64-v8a/` and the generated UniFFI Kotlin into `app/src/main/java/` (`com/z_db/android_mascot`, `rustlib/*`, `uniffi/protocol`). It deletes and recreates those Kotlin folders every run, so never hand-edit generated files there. It patches package names with sed (`uniffi.my_yrs_lib` and `uniffi.text_diff` become `rustlib.*`), fails if a source file is missing (the matching `build_android.sh` must have run first, `../build_all_android.sh` builds everything), and prints "a change was made" or "nothing changed". Hashed `libmy_yrs_lib-<hash>.so` files come from the `client_table_blueprints` output and are removed and recopied each run.

Only arm64-v8a is shipped, minSdk is 31, compileSdk 37. `build_and_install.sh` runs `:app:assembleDebug`, refuses emulators, needs exactly one physical device (or `ANDROID_SERIAL`), installs with `adb install -r`, and exits non-zero if the `z.zndroid.debug.BEGIN_TEST_TX` receiver is not registered.

## End-to-end testing

The app is tested by an external Python/Appium harness at `../e2e_android_tests/`.
Three tests:
- `test_multiple_blocks` and `test_page_lifecycle`: user-visible flow tests.
- `test_dev_checks`: navigates to Dev Tools, taps "Run all checks", reads the `DevCheck summary` log line, asserts `fail=0`. If a check fails, the pytest output shows each failing check's id and one-line message only; the full detail is on the `st=... stack=` logcat line.

Both user-visible flow tests and the dev-checks test run inside one external transaction opened by `TestTransactionReceiver` (debug-only broadcast receiver); teardown rolls it back. Removing or gating this receiver makes setup fail after a 20 s wait ("BEGIN_TEST_TX did not report OK"), and the report is marked session failed.

Popup severity: `GlobalPopupManager.show(msg)` (informational) vs `showError(msg)` (error, prefixed `ERROR:` in the snackbar and logged as `Log.e`). Harness reads popups from logcat tag `GlobalPopup` because snackbars render behind Dialog windows. The logcat lines are written only when BuildConfig.DEBUG. The harness fails any step during which a `GlobalPopup` line was logged, whether it came from `show` or `showError`, so an informational popup fails an e2e test too. `show` / `showError` are suspend functions over a `MutableSharedFlow` with no replay or buffer: a popup emitted while `GlobalPopup()` is not collecting (before first composition, during activity recreation) is dropped silently. Info popups render in the same error colours as errors. Rule: anything that means "something went wrong" is `showError`.

The `CHECK_SYNC` broadcast still works: `am broadcast -a z.zndroid.debug.CHECK_SYNC --es pageId <page_id>` runs `z.zndroid.Tests.CheckIfTablesInSync.checkPageSync(pageId)` and logs `SyncCheck page=X synced=true/false total=N mismatches=M`. Nothing currently calls it. Kept for future logic tests.

E2E commands:
- `cd ~/ProgStuff/ZakkeN/android_app && ./build_and_install.sh` (builds and installs)
- `~/ProgStuff/ZakkeN/e2e_android_tests/run-ui-tests.sh` (runs the three tests)
- Do not edit anything under `../e2e_android_tests/` unless the task is explicitly about the harness. If a test is wrong, report it to the human.
- `./gradlew test` does not compile today: `src/test/.../BuildVerificationTest.kt` references `RegisterTests`, which does not exist (see MISSING_AND_STALE.md).