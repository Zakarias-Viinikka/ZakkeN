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
uncommitted_diffs: id, snapshot_of_edit, love_letter_sketch, session_id, target_id
key_value_storage: id, key, value
incoming_love_letters: id, love_letter, target_page_id
logs: id, timestamp, level, category, source, session_id, message, details, details_type

Table column creation functions:
newTablePages()
newTableUncommittedDiffs()
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
RemoveBlock { position, target_page_id }

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
