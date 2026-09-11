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

Table blueprints (client_table_blueprints):
pages: page_id, blobbed_page, page_status, version, is_main_menu_page
backlinks: page_that_holds_link_id, page_being_linked_to_id, disabled, version
every_block_in_existence: is_title (bool flag), content, my_id_as_given_by_yrs, id_of_page_i_belong_to
uncommitted_diffs: snapshot_of_edit, love_letter_sketch, session_id, target_id
key_value_storage: key, value

Row creation helpers:
new_page_row(page_id, is_main_menu_page, user_id)
new_backlink_row(owner, target)
new_key_value_item(key, value)
new_every_block_in_existence_row(is_title: bool, content, my_id_as_given_by_yrs, id_of_page_i_belong_to)

LoveLetter library:
LoveLetterSketch – describes the user’s intent. Variants:
EditBlock { text_edit, edit_target, target_page_id, block_id }
CreateNewBlock { position_to_insert, target_page_id }
RemoveBlock { position, target_page_id }

Document Events (z.zndroid.DocEvents):
AddBlock: insert new block (persists to data + sync tables). Returns blockId.
EditTextInBlock: updates text using text_diff. Generates Yrs TextEdit.
RemoveBlock: deletes block from CRDT and SQLite.
CheckIfTablesInSync: compares SQLite row content vs CRDT blob content.

Logging (z.zndroid.log):
Modular system for background logging.
Logger.kt: Engine using Dispatchers.IO. Methods: i, w, e.
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
   - Clears buffer and updates lastPersistedText.

Special Key Events (onPreviewKeyEvent):
- Enter: splitBlock (splits text at cursor, spawns new block).
- Backspace at Pos 0: mergeWithPreviousBlock (appends content to previous block, deletes current).

AI documentation:
All architectural notes and rules are located in the docs/ folder.
- AI_MEMORY.md: Current project state and learnings.
- AI_READ_THIS.md: This library guide.
- AI_LAB_RULE.md: Rules for the AI Lab.
- UX_INTENT.md: UX philosophy and intended editor behavior.
- FORME.txt: Personal notes.
