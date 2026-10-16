# Shared Memory - Zndroid Project

## Architecture Highlights
- **Authoritative Source**: Yrs CRDT blobs in the `pages` table (`blobbed_page` column).
- **Queryable Source**: SQLite table `every_block_in_existence`.
- **Sync Strategy**: Intent-based. `uncommitted_diffs` stores `LoveLetterSketch` (serialized intent) + Yrs diff snapshot.
- **DB Concurrency**: All database access is serialized via a `Mutex` in `DbManager.kt` to prevent race conditions (Android port of WASM `askQueue`).
- **Dual-Layer Storage**: `StorageAccess` uses `FastStorage` (RAM) first for whitelisted keys, then falls back to SQLite.
- **Buffered Batching & Squash-First Rule**: Edits are collected in `BlockUiState.diffBuffer` and "squashed" using `text_diff.combineGetdiffResults` before persistence (500ms debounce). All non-plain-text structural operations (split, merge, delete, tapping block) flush the pending buffer first in a single transaction before running. `syncUiStates` skips text updates when a block has a non-empty `diffBuffer`.
- **Fail-Hard Flush**: Edits either fully commit or are discarded. On failure, a mandatory page reload is triggered in `ViewPage` to rebuild in-memory state from DB.
- **Mapping Safety & Drift Prevention**: `SafeRowMapper` utility verifies column names during Rust-to-SQLite mapping. Parallel `ColumnDescriptor<T>` schemas (`PagesTableSchema`, `EveryBlockInExistenceTableSchema`, etc.) and `SchemaCanaryTest` enforce hardcoded type expectations for Android schema drift detection.
- **Schema Migrations**: Handled by a transactional walk-loop via `MigrationManager` upon database opening (`DbManager.init`). Database versions are managed sequentially (`SchemaVersion` enum, currently at LATEST = V1) and tracked atomically using the `schema_version` key in `key_value_storage`. A snapshot integrity check in `MigrationTest` ensures that any future drift in live Rust blueprints triggers an immediate build-time error until a new version is explicitly frozen in Kotlin (`Version1.kt`).
- **Editing Logic**:
  - **Split**: `Enter` (without Shift) flushes pending edits and splits a block at the cursor in a single transaction. Text after moves to a new block.
  - **Merge**: `Backspace` at position 0 flushes pending edits, merges current block into previous one, and deletes current block in a single transaction.
  - **Re-use**: `ViewPage` reuses `BlockUiState` to preserve cursor/selection during UI refreshes.
- **Page Titles & Navigation**:
  - `page_id` is the stable identity and never changes when renaming.
  - Page title is read from the title block (`is_title = "true"`) in `every_block_in_existence`.
  - Empty titles are displayed as "Untitled" in a muted color in `NavPage`.
  - Rename-from-menu edits the title block's content via `DbManager.renamePage` through the builder/executor path.
- **AI Lab**: Experimental UX testing ground in `lab/` package.
  - **Modular**: Experiments in `lab/experiments/` (e.g., `scroll`, `selection`, `undoredo`).
  - **Core**: `LabNavigator` handles routing; `LabContainer` wraps the app for overlays.
  - **Rules**: Governed by `docs/AI_LAB_RULE.md`.
- **Logging System**: Modular system in `log/` package writing to Logcat and SQLite `logs` table.
  - **Engine**: `Logger.kt` processes logs in background `CoroutineScope` and handles transaction boundaries.
  - **Transactional Buffering**: Outside transaction -> inserts directly into `logs` table. Inside transaction -> buffers in `DbTransactionElement.logBuffer`. On commit -> drops buffer. On rollback -> flushes buffer to `logs` table outside failed transaction.
  - **Explicit Context**: Every call site passes `source` parameter explicitly (e.g., `ViewPageLogs.kt`). No fragile stack trace introspection.
- **Schema**:
  - Table functions named `newTableX()`.
  - `every_block_in_existence`: `id`, `is_title` ("true"/"false"), `is_part_of_main_menu_page`, `content`, `my_id_as_given_by_yrs`, `id_of_page_i_belong_to`, `position`, `page_is_disabled`.
  - `key_value_storage`: `id` (INTEGER autoincrement primary key), `key` (TEXT unique), `value` (TEXT).
  - `logs`: `id`, `timestamp`, `level`, `category`, `source`, `session_id`, `message`, `details`, `details_type`.
  - `incoming_love_letters`: `id`, `love_letter`, `target_page_id`.

## AI Documentation Index
All architectural notes and rules are located in the `docs/` folder:
- [SHARED_MEMORY.md](file:///home/zakke/ProgStuff/ZakkeN/android_app/docs/SHARED_MEMORY.md): (This file) Architecture highlights and learnings.
- [AI_READ_THIS.md](file:///home/zakke/ProgStuff/ZakkeN/android_app/docs/AI_READ_THIS.md): Library guide and API reference.
- [INVENTED_ON_ANDROID.md](file:///home/zakke/ProgStuff/ZakkeN/android_app/docs/INVENTED_ON_ANDROID.md): Designs created on Android without a Rust or web reference yet.
- [AI_LAB_RULE.md](file:///home/zakke/ProgStuff/ZakkeN/android_app/docs/AI_LAB_RULE.md): Guidelines for the Lab.
- [UX_INTENT.md](file:///home/zakke/ProgStuff/ZakkeN/android_app/docs/UX_INTENT.md): Philosophy and behavior of the editor UX.
