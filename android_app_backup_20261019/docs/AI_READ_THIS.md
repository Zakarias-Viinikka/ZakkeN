ZakkeN Project – Library Guide for AI
This project uses Rust libraries exposed to Android via UniFFI. The main ones:

z_db: SQLite database wrapper (db_wrapper)
protocol: data types (records, enums) and serialization
client_table_blueprints: table schemas and row creation helpers
yrs: Yrs CRDT wrappers for page content and sync
love_letter: edit contract types and serialization for client–server communication
text_diff: character-level diffing and "squashing" logic for text editing

Kotlin generated bindings:
com.z_db.android_mascot (from db_wrapper)
uniffi.protocol (from protocol)
rustlib.client_table_blueprints (from client_table_blueprints)
rustlib.my_yrs_lib (from yrs)
rustlib.love_letter (from love_letter)
rustlib.text_diff (from text_diff)

Core database object: LiveForever
Methods: create_table, create_foreign_table, list_tables, get_data, get_data_ordered, insert_data, drop_table, edit_col_in_row, check_table, delete_row, swap_columns, create_index, check_index, add_column, remove_column, export_database, export_tables, create_table_from_export, copy_table.

Use `awaitReady()` before calling DB methods in UI components.

Database Lifecycle & Schema Migrations:
All table schemas are managed via `z.zndroid.db.migrations.MigrationManager`. It executes a transactional walk-loop on initialization inside `DbManager.init(context)`. Version states are tracked inside the `SchemaVersion` enum (LATEST is V1) and persisted under the `schema_version` key in `key_value_storage`.

Table blueprints (client_table_blueprints - Note: Historic snapshots are frozen inside `z.zndroid.db.migrations.schemas` package):
pages: id, page_id, blobbed_page, page_status, version, is_main_menu_page
backlinks: id, page_that_holds_link_id, page_being_linked_to_id, disabled, version
every_block_in_existence: id, is_title, is_part_of_main_menu_page, content, my_id_as_given_by_yrs, id_of_page_i_belong_to, position, page_is_disabled
client_love_letter: id, snapshot_of_edit, love_letter_sketch, session_id, target_id
key_value_storage: id, key, value
incoming_love_letters: id, love_letter, target_page_id
logs: id, timestamp, level, category, source, session_id, message, details, details_type

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
newBacklinkRow(pageThatHoldsLinkId, pageBeingLinkedToId, disabled, version)
newKeyValueItem(key, value)
newEveryBlockInExistenceRow(isTitle, isPartOfMainMenuPage, content, myIdAsGivenByYrs, idOfPageIBelongTo, position)
newLogRow(timestamp, level, category, source, sessionId, message)
newIncomingLoveLetterRow(loveLetter, targetPageId)

LoveLetter library:
LoveLetterSketch – describes the user’s intent. Variants:
EditBlock { text_edit, edit_target, target_page_id, block_id }
CreateNewBlock { position_to_insert, target_page_id }
RemoveBlock { block_id, target_page_id }

Document Events (z.zndroid.DocEvents):
AddBlock: insert new block (persists to data + sync tables). Returns blockId.
  Position rule: SQLite `position` is an f64 sort key, gap-based (avg of above/below when inserting in the middle, max+1 at end, min-1 at top). New rows get position computed in `AddBlock.computeDbPosition`; there is no reindexing of neighbors on insert.
  Yrs array order is authoritative for content; SQLite `position` only orders the queryable table.
EditTextInBlock: updates text using text_diff. Generates Yrs TextEdit.
RemoveBlock: deletes block from CRDT and SQLite.
CheckIfTablesInSync: compares SQLite row content vs CRDT blob content.

Logging (z.zndroid.log):
Modular system for background logging and database persistence.
Logger.kt: Engine using Dispatchers.IO. Writes to Logcat and `logs` SQLite table.
  - Outside transaction: inserts log directly into SQLite.
  - Inside transaction: buffers in `DbTransactionElement.logBuffer`. On commit, drops buffer. On rollback, flushes buffer to `logs` table outside failed transaction.
  - Source parameter (file/class/function) is passed explicitly at call sites.
ViewPageLogs.kt: Feature-specific helpers (logPageInit, logPageLoadSuccess, etc.).
LogDateHelpers.kt: Unix timestamp conversion for query ranges.

AI Lab (z.zndroid.lab):
Experimental UX testing ground, isolated from production code.
Rules: See AI_LAB_RULE.md. Modular folder structure in experiments/.
Core: LabNavigator (routing) and LabContainer (overlays).

Buffered Editing Pattern:
1. BlockUiState: maintains diffBuffer (List<DiffResult>) and uses TextFieldValue for cursor tracking.
2. EditableBlock: onValueChange appends to buffer and triggers 500ms debounce flush.
3. Flush:
   - Calls text_diff.combineGetdiffResults(buffer).
   - Applies squashed diffs to BossOfYrs and syncs uncommitted_diffs.
   - Updates every_block_in_existence content and page snapshot once per batch.
   - Clears buffer.
4. Squash-First Rule: Any structural operation (split, merge, delete) flushes pending edits first in a single transaction before running. `syncUiStates` skips text syncing for blocks with non-empty `diffBuffer`.

Special Key Events (onPreviewKeyEvent):
- Enter: splitBlock (flushes buffer and splits text at cursor, spawning new block in a single transaction).
- Backspace at Pos 0: mergeWithPreviousBlock (flushes buffer, appends content to previous block, deletes current in a single transaction).

Navigation & Page Titles:
- `page_id` is the stable identity and never changes during rename.
- Title is read from the title block (`is_title = "true"`) in `every_block_in_existence`.
- Empty title blocks display "Untitled" in a muted color in `NavPage`.
- Rename-from-menu calls `DbManager.renamePage` which updates the title block content via builder/executor.

AI documentation:
All architectural notes and rules are located in the docs/ folder.
- SHARED_MEMORY.md: Current project state and learnings.
- AI_READ_THIS.md: This library guide.
- INVENTED_ON_ANDROID.md: Choices and designs created on Android without a reference yet.
- AI_LAB_RULE.md: Rules for the AI Lab.
- UX_INTENT.md: UX philosophy and intended editor behavior.

## End-to-end testing (2026-10-17)

The app is tested by an external Python/Appium harness that drives the UI on a
physical phone over adb. It lives at `../e2e_android_tests/`, outside this
project tree, and is not part of the Android build.

Two things the app provides to the harness:

**`TestTransactionReceiver`** (in `src/debug/java/z/zndroid/`) — debug-only
broadcast receiver. Three actions:

    z.zndroid.debug.BEGIN_TEST_TX         — DbManager.beginExternalTransaction()
    z.zndroid.debug.END_TEST_TX_ROLLBACK  — DbManager.endExternalTransaction(commit = false)
    z.zndroid.debug.END_TEST_TX_COMMIT    — DbManager.endExternalTransaction(commit = true)

The harness broadcasts BEGIN before a run and ROLLBACK at teardown. Every DB
write the app makes during the run happens inside that transaction; rollback
discards them, so the harness cannot leave user data modified. A run that is
force-killed leaves the transaction open on the phone until the app process is
killed (adb force-stop, or reboot).

**DbManager external transaction hook.** When `externalTransactionActive` is
true (set by `beginExternalTransaction`), `withTransaction` runs its block
inside the already-open transaction instead of starting a nested one. This
must stay. Removing it breaks every UI action that wraps writes in
`withTransaction` while the harness holds a transaction open.

## Popup severity (2026-10-17)

`GlobalPopupManager` has two entry points:

    show(msg)      — informational popup. Harness records, does not fail.
    showError(msg) — error popup. Harness fails the step that produced it.
                     Rendered with an "ERROR: " prefix in the snackbar text.

Both are logged to logcat under tag `GlobalPopup` (`Log.i` for info, `Log.e`
for errors). The harness reads popups from logcat, not the a11y tree, because
a Compose `Snackbar` renders in the activity window while a `Dialog` (used by
`NewPagePopup` and other modals) is a separate, higher window. When a dialog
is open, the snackbar is behind it and not visible to Appium.

Rule: if the popup is telling the user something went wrong, use `showError`.
If it is telling them something informational happened, use `show`.

## Fixed bugs (2026-10-17)

**`DbManager.addPage` insert order.** (Fixed) `addPage` creates `boss` as `BossOfYrs(userId, pageId)` and inserts the `pages` record into SQLite *before* `AddBlock.execute()` so foreign keys referencing `pages.page_id` succeed.

**`DbManager.renamePage` persistence.** (Fixed) `renamePage` now runs inside `withTransaction { ... }`, applies the edit, and writes the updated snapshot back to the `pages` table via `updatePageSnapshot(pageId, updatedSnapshot)`.

## Known bugs (2026-10-17)

**Popup renders behind dialogs.** `GlobalPopup`'s `Snackbar` lives in the
activity's `Box`; `NewPagePopup` is a Compose `Dialog`, a separate window.
When both are visible the snackbar is behind the dialog's scrim. Visual only
— the harness reads popups from logcat so it is not blocked. Fixable by
moving the snackbar to a top-level `Popup`/`Dialog`, but not urgent.

## E2E tests — how you run them

You run the tests yourself. One command, full path:

    /home/zakke/ProgStuff/ZakkeN/e2e_android_tests/run-ui-tests.sh
    (equivalently: ~/ProgStuff/ZakkeN/e2e_android_tests/run-ui-tests.sh)

It handles connecting to the phone, starting Appium if needed, and running
the suite against the already-installed app. It does not build or install
the APK — do that first if you changed app code:

    cd ~/ProgStuff/ZakkeN/android_app && ./build_and_install.sh

Then run the test script. Read its terminal output. The script stops at
the first failing step, so each run shows you one failure at a time. Fix
what it reports, rebuild, rerun.

The terminal output tells you everything you need: which step failed, the
exception, any app `logcat:` lines with `GlobalPopup:` messages, and any
ERROR popups. Do not go into `../e2e_android_tests/artifacts/` — that is
diagnostic detail for the test system itself, not for fixing the app.

Do not edit anything under `../e2e_android_tests/`. If a test is failing
because the test itself is wrong (not the app), report that to the human
and stop — do not try to fix the test.

## Additional test hooks (2026-10-17)

**`CHECK_SYNC`** — debug broadcast `z.zndroid.debug.CHECK_SYNC`, takes an
`--es pageId <id>` extra. Runs `CheckIfTablesInSync.checkPageSync(pageId)`
and logs the result to logcat under `TestTransactionReceiver`:
`SyncCheck page=X synced=true/false total=N mismatches=M`. The harness uses
this as a backend assertion: after every mutating step in some tests, Yrs
blob and `every_block_in_existence` must agree.

**`NOTIFY_TEST_DONE`** — debug broadcast `z.zndroid.debug.NOTIFY_TEST_DONE`,
takes `--es result PASS|FAIL` and `--es summary <text>`. Posts a silent
notification so the human sees the run finished without checking the
terminal. No sound, no vibration. `POST_NOTIFICATIONS` is declared in the
debug manifest; on Android 13+ that needs a runtime grant once.

## 2-block minimum (deliberate)

Every page has at least two blocks: the title (index 0) and a body
placeholder (index 1). `DocLogic.kt` enforces this — backspace-at-pos-0
on either block is a no-op. Tests that assert "1 EditText remains after
deleting the last body block" are wrong; 2 is correct. The body
placeholder's ghost text ("Type '/' for commands...") is the cue that
there's always an editable block below the title.

## Force-stop on failure

When a test run fails, the harness force-stops the app (`am force-stop
z.zndroid`) before exiting. The human is told their phone is free. On
success, the app is left on the nav page instead. A force-stopped app
loses its in-memory Yrs state, which is fine — the DB on disk is
authoritative and the next launch reloads from it.

## E2E suite — what changed 2026-10-19

Two tests now, both user-visible behavior:

- `test_multiple_blocks.py` — create a page, add three body blocks, verify all four blocks (title + three) appear in order
- `test_page_lifecycle.py` — create a page, add a body block, reopen the page, verify the body persisted

Run with `~/ProgStuff/ZakkeN/e2e_android_tests/run-ui-tests.sh`. Full-path form: `/home/zakke/ProgStuff/ZakkeN/e2e_android_tests/run-ui-tests.sh`.

For iteration, run pytest directly with Appium started separately:
1. `export ANDROID_HOME=/home/zakke/Android/Sdk` in the terminal that starts Appium, then `appium` (leave it running)
2. In a different terminal: `cd ~/ProgStuff/ZakkeN/e2e_android_tests && .venv/bin/pytest tests/test_page_lifecycle.py -v`
3. When done, Ctrl+C in the Appium terminal

Test data should avoid doubled letters. `mobile: type` drops characters on some Samsung devices, most often the second of a doubled character. Use "help" instead of "hello", "delta" instead of "gamma". The readback helper in `ui.type_text` retries when a character drops, but avoiding doubles eliminates the most common trigger.
