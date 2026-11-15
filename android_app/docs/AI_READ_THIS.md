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
All table schemas are managed via `z.zndroid.db.migrations.MigrationManager`. It executes a transactional walk-loop on initialization inside `DbManager.init(context)`. Version states are tracked inside the `SchemaVersion` enum (LATEST is V4; frozen snapshot files exist for Version0-4, and V3 onward use a `FrozenTable` list). The version number is persisted under the `schema_version` key in `key_value_storage`. A host-side schema drift detector lives at `schema_drift_detector/` and compares the Rust blueprints against `Version3.kt` and `TableDescriptors.kt`; it runs as a `preBuild` Gradle task and in `build_and_install.sh`.

Table blueprints (client_table_blueprints - Note: Historic snapshots are frozen inside `z.zndroid.db.migrations.schemas` package):
pages: id, page_id, blobbed_page, page_status, version, is_main_menu_page
backlinks: id, page_that_holds_link_id, page_being_linked_to_id, disabled, version
every_block_in_existence: id, is_title, is_part_of_main_menu_page, content, my_id_as_given_by_yrs, id_of_page_i_belong_to, frac_position, page_is_disabled
client_love_letter: id, snapshot_of_edit, love_letter_sketch, session_id, target_id
key_value_storage: id, key, value
incoming_love_letters: id, love_letter, target_page_id
logs: id, timestamp, level, category, source, session_id, message (no details / details_type columns)
seen_server_letters: id, letter_id (added in V4; Android doesn't read or write it yet — reserved for the sync receive loop)

Column descriptors (Android's substitute for Rust's generic SchemaColumn<T>):
`z.zndroid.db.schema` holds one object per table (PagesTableSchema, EveryBlockInExistenceTableSchema, etc.). Each column is a `ColumnDescriptor<T>` with name, typeName, and unwrap(Col) -> T. The host-side `schema_drift_detector` parses this file and compares it against the live Rust blueprints; no in-app canary test exists any more (SchemaCanaryTest and ExpectedSchema were deleted).

Reads:
`z.zndroid.db.reads` is the main place that queries tables, and every read in it is descriptor-based. It is NOT the only place: SchemaVersion.V0, InspectTable and the CHECK_SYNC fallback in TestTransactionReceiver still read columns by position (see MISSING_AND_STALE.md). Most reads return Result<T>; isMainMenuPage and isPageDisabled return Boolean and throw on failure.
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
- `flush` is `flushNow`. `flushAsync`, used from `DisposableEffect.onDispose`, launches a flush on the batcher's own scope so composition-scope cancellation doesn't abort it. ViewPage calls `flush` on top-bar back, system BackHandler and lifecycle ON_STOP, and ignores its Result. On a session replacement (reload), `PageSession.destroyAfterFlush()` flushes, destroys the boss, then destroys the batcher (which cancels its scope and refuses later pushes).

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
- `hardDeletePage(pageId)` in DbManager is dev-only. Used by ops checks for scratch-page cleanup. It deletes the page's mirror rows, both backlink directions, and its `client_love_letter` rows. Takes no `WriteLock`. The dev page's "Wipe all page data" does not use it (it deletes rows from four tables directly).
- `page_id` is the stable identity and never changes during rename.
- Title is the content of the block whose mirror row has `is_title = "true"`. Not the first block in yrs order.
- Empty title blocks display "Untitled" in a muted colour in `NavPage`.

Session id:
`SessionManager.initialize` reads the stored id, sets `currentSessionId` to that + 1 synchronously, and persists the new value. Navigation does not bump it, and `MainActivity` no longer registers a destination-changed listener. A read error or unparsable stored value shows an error popup and leaves the counter where it was rather than resetting it to 0, so ids already written to `client_love_letter` are not re-issued. `StorageAccess.setValue` writes to the DB first and updates `FastStorage` only after that succeeds, so the first write for `session_id` takes the insert path (previously it read FastStorage first and always took the edit path, which likely updated zero rows).

Logging (z.zndroid.log):
Modular system for background logging and database persistence.
Logger.kt: Engine using Dispatchers.IO. Writes to Logcat and `logs` SQLite table.
  - Outside transaction: inserts log directly into SQLite.
  - Inside transaction: buffers in `DbTransactionElement.logBuffer`. On commit, drops buffer. On rollback, flushes buffer to `logs` table outside failed transaction. The transaction is found through the coroutine context (`DbManager.DbTransactionElement`), so a thread switch inside a transaction still routes to the right buffer. `Logger.i/w/e/log` and `ViewPageLogs.*` are `suspend` because of that. Under the e2e external transaction nothing is buffered at all.
  - Source parameter (file/class/function) is passed explicitly at call sites.
  - `logScope` has a `SupervisorJob`, so a failed write does not cancel the scope. A failed write is reported to Logcat.
ViewPageLogs.kt: Feature-specific helpers (logPageInit, logPageLoadSuccess, etc.). In practice these are the only log calls in the app: `ViewPage` calls three of the five helpers, so the `logs` table holds only page-open and page-load lines, and nothing logs inside a transaction today (grep 2026-11-13, including direct `Logger.log(` calls).
LogDateHelpers.kt: Unix timestamp conversion for query ranges. Returns milliseconds in the device's zone, matching `LogEntry.timestamp`.

## Server sync

Android talks to `zakken_server` over a websocket. The wire is Rust behind uniffi. The Kotlin business logic mirrors web's `cqrs_state_synchronization/web_interface/src/owner_of_interacting_with_server/` one file per file.

Rust layers (shared with web unless noted):
- `zakken_server/client_internals/shared_internals/` — router, request IDs, pending replies, conn_status, drift_version. `socket_web.rs` runs on WASM, `socket_android.rs` on Android (tokio-tungstenite).
- `zakken_server/client_internals/server_link/` — uniffi wrapper, Android-only. Exposes `ServerLink` object, `LinkListener` foreign trait, all reply types, `setDriftVersion` / `getDriftVersion` / `fingerprintOfBlob` / `newDriftVersion`.
- `zakken_server/shared/server_protocol/` — Request/Response/payload types. Serde only, no uniffi.

Kotlin files under `z/zndroid/sync/owner_of_interacting_with_server/`:
- `Core.kt` — `connectToSocket()` reads `server_url` from `key_value_storage` (mints `ws://127.0.0.1:3000/ws` on first run), calls `ServerLink.connect`, spawns auto-reconnect / drift-changed / sender / receiver loops.
- `Login.kt` — login/register, `afterLogin` adopts the server's drift version, `enterLonewolf` when mismatched.
- `SendLoop.kt` — drains `client_love_letter` oldest-first, sends, deletes on ok. Triggered by `kickSend()` (`actions/Kick.kt`, a Conflated `Channel`).
- `ReceiveLoop.kt` — handles pushed letters and pulls unsent after login. `applyLetter` returns `UnknownPage` → `recoverUnknownPage` fetches the page, applies it, then re-applies the letter.
- `ApplyLetter.kt` — applies a `LoveLetter` to yrs + mirror. Never writes back to `client_love_letter`.
- `Bootstrap.kt` — `startOverFreshGuard` pulls everything from server once when local has no pages.
- `Reconcile.kt` — `loadAndClassify` / `verifyNothingChangedSinceMount` / `sendReconcileBatch`. Called from `Recovery.kt`.
- `Recovery.kt` — glue for the recovery screen. Offers "server wins" reconcile and "wipe self and adopt server". Debug builds also get "wipe server".
- `SeenLetters.kt` — in-memory set + `seen_server_letters` persistence, dedupes pushed letters.
- `SpontaneousInbox.kt` — Channels that carry Rust-callback-thread pushes to the coroutine loops.
- `KeyValue.kt`, `ConnState.kt`, `SyncListener.kt` — support.
- `MainPages/RecoveryPage.kt` — the recovery screen, reached from `NavPage`'s lonewolf banner.
- `editor/OpenPageRegistry.kt` — tracks which pages have a live `ViewPage` open. A letter for an open page merges into the live doc; otherwise it's deferred.

Storage keys (all in `key_value_storage`):
- `server_url` — full websocket URL. Auto-minted.
- `server_user_name` — the name registered/logged in with.
- `client_drift_version` — the drift version the client is on. Mismatch → lonewolf.
- `should_i_drift_popup` — flag so the lonewolf popup shows once.

Manifest:
- Main: `INTERNET` permission.
- Debug: `usesCleartextTraffic="true"`. No TLS feature; `ws://` only, `wss://` would need one.
- Dev setup: `adb reverse tcp:3000 tcp:3000` + default `ws://127.0.0.1:3000/ws` just works. LAN dev: overwrite `server_url` with the PC's IP.

Verified 2026-11-15: connect, login, receive queued letters, apply, ack. Recovery UI exists (lonewolf banner → `RecoveryPage`, server-wins reconcile, wipe-self, debug wipe-server); reachable only while Lonewolf / LonewolfAndOffline. ViewPage merges remote edits into the open doc via `OpenPageRegistry`. Unverified end-to-end: send path, two-client visibility, drift/reconcile, wipe.

AI Lab (z.zndroid.lab):
Experimental UX testing ground, isolated from production code.
Rules: See AI_LAB_RULE.md. Modular folder structure in experiments/.
Core: `LabNavigator` (lab/core/LabNavigator.kt: routes Production and ScrollLab, plus undo-bar and multi-select toggles) and `LabContainer` (lab/core/LabManager.kt: overlays and the smart button). Experiments: scroll, selection, undoredo. NOT MOUNTED (2026-11-13): `MainActivity` uses `MainContainer`, not `LabContainer`, so none of this is reachable from the app today (see MISSING_AND_STALE.md). The undo bar is a mock (empty click handlers) and the selection overlay only draws a trail.

Dev Tools page (nav button and the `test_results` route are both gated by BuildConfig.DEBUG; the page class compiles into release but is unreachable):
Replaces the old "Internal Test Suite". Route is still `test_results`. The nav button label is "Dev Tools".
- `DevCheckRegistry.checks` is the one list of checks. `DevToolsPage` runs them sequentially.
- Each check logs `Log.i("DevCheck", "id=<id> status=<PASS|FAIL> detail=<first 200 chars of the check's one-line message>")`.
- The last line is `Log.i("DevCheck", "summary pass=N fail=M")`. The e2e harness reads that.
- Sections: schema.* (blueprints, canary, live DB), migration, session.monotonic, mirror.vs_yrs, outbox.integrity, outbox.replay, ops.*, client_vs_server (reserved, not implemented).
- Actions section (wipe / seed) is separate from "Run all". Not part of the pass/fail count. Wipe and seed run on a single tap with no confirmation.
- Tapping a check card toggles its detail (when there is one); a separate Run button on the card runs that check. The check's real detail (`errorContext`) is on a second logcat line, `id=<id> st=<status> stack=<first 1500 chars>`; read failure detail with `adb logcat -d -s DevCheck:V`.

## Native libraries and bindings

`./update.sh` (in android_app/) copies the Rust build outputs into this tree: `.so` files into `app/src/main/jniLibs/arm64-v8a/` and the generated UniFFI Kotlin into `app/src/main/java/` (`com/z_db/android_mascot`, `rustlib/*`, `uniffi/protocol`). It deletes and recreates those Kotlin folders every run, so never hand-edit generated files there. It patches package names with sed (`uniffi.my_yrs_lib` and `uniffi.text_diff` become `rustlib.*`), fails if a source file is missing (the matching `build_android.sh` must have run first, `../build_all_android.sh` builds the five ZakkeN crates, not z_db's `protocol` and `db_wrapper` (those come from `z_db/build_web_and_android.sh`), and then runs `update.sh`), and prints "a change was made" or "nothing changed". Hashed `libmy_yrs_lib-<hash>.so` files come from the `client_table_blueprints` output and are removed and recopied each run.

Only arm64-v8a is shipped, minSdk is 31, compileSdk and targetSdk 37. Build tooling (gradle/libs.versions.toml): AGP 9.3.3, Kotlin 2.2.10, Compose BOM 2026.02.01, navigation-compose 2.10.0. JNA is hard-coded in app/build.gradle.kts as 5.16.0 (aar); the catalog's 5.19.1 entry is unused. `build_and_install.sh` runs `:app:assembleDebug`, refuses emulators unless `ANDROID_SERIAL` names one, needs exactly one physical device otherwise, installs with `adb install -r`, and exits non-zero if the `z.zndroid.debug.BEGIN_TEST_TX` receiver is not registered.

## End-to-end testing

The app is tested by an external Python/Appium harness at `../e2e_android_tests/`.
Three tests:
- `test_multiple_blocks` and `test_page_lifecycle`: user-visible flow tests.
- `test_dev_checks`: navigates to Dev Tools, taps "Run all checks", reads the `DevCheck summary` log line, asserts `fail=0`. If a check fails, the pytest output shows each failing check's id and one-line message only; the full detail is on the `st=... stack=` logcat line.

Both user-visible flow tests and the dev-checks test run inside one external transaction opened by `TestTransactionReceiver` (debug-only broadcast receiver); teardown rolls it back. Removing or gating this receiver makes setup fail after a 20 s wait ("BEGIN_TEST_TX did not report OK"), and the report is marked session failed.

Popup severity: `GlobalPopupManager.show(msg)` (informational) vs `showError(msg)` (error, prefixed `ERROR:` in the snackbar and logged as `Log.e`). Harness reads popups from logcat tag `GlobalPopup` because snackbars render behind Dialog windows. The logcat lines are written only when BuildConfig.DEBUG. The harness fails any step during which a `GlobalPopup` line was logged, whether it came from `show` or `showError`, so an informational popup fails an e2e test too. Both are fire-and-forget over an unbounded `Channel`, so a popup requested before `GlobalPopup()` first composes (or during activity recreation) is queued and shown by the next collector rather than dropped; `collect` (not `collectLatest`) means back-to-back popups queue rather than replace. Info popups use `secondaryContainer`, errors use `errorContainer`. Rule: anything that means "something went wrong" is `showError`.

The `CHECK_SYNC` broadcast still works: `am broadcast -a z.zndroid.debug.CHECK_SYNC --es pageId <page_id>` runs `z.zndroid.Tests.CheckIfTablesInSync.checkPageSync(pageId)` and logs `SyncCheck page=X synced=true/false total=N mismatches=M`. Nothing currently calls it. Kept for future logic tests.

E2E commands:
- `cd ~/ProgStuff/ZakkeN/android_app && ./build_and_install.sh` (builds and installs)
- `~/ProgStuff/ZakkeN/e2e_android_tests/run-ui-tests.sh` (runs the three tests)
- Do not edit anything under `../e2e_android_tests/` unless the task is explicitly about the harness. If a test is wrong, report it to the human.
- `./gradlew test` runs the tests listed in `RegisterTests` (`app/src/main/java/z/zndroid/Tests/RegisterTests.kt`). Tests that need the native libs (anything touching `client_table_blueprints`) skip on the host JVM.