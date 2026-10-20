# Invented on Android

Things Android does that have no Rust or web counterpart yet. Each entry
says what Android does and where the equivalent should eventually live,
so it can be checked once the web side catches up.

When you design something without a reference, add an entry here. When
the web or Rust side catches up, read the entry and verify Android's
choice.

---

### Parallel Column Schema Descriptors & Type Canary Tests

Android: Defined parallel `ColumnDescriptor<T>` Kotlin objects per table (`PagesTableSchema`, `EveryBlockInExistenceTableSchema`, `BacklinksTableSchema`, `ClientLoveLetterTableSchema`, `KeyValueStorageTableSchema`, `IncomingLoveLettersTableSchema`, `LogsTableSchema`) holding column name, type name, and unwrap lambda `(Col) -> T`. Created `SchemaCanaryTest` asserting hardcoded type expectations for each column descriptor as a compile/test time substitute for Rust's generic `SchemaColumn<T>`.

Reference location: `client_table_blueprints` or `schema` crate in Rust.

To verify: Compare `ColumnDescriptor` field types and `SchemaCanaryTest` assertions against the generic `SchemaColumn<T>` definitions in Rust schema crate.

---

### In-Memory Transaction Log Buffer & Rollback Flush

Android: `DbTransactionElement` maintains a thread-local / coroutine-context in-memory log buffer (`logBuffer`). Logs emitted via `Logger` during an active transaction are appended to `logBuffer`. When a transaction commits (`onCommit`), `logBuffer` is cleared without writing. When a transaction rolls back (`onRollback`), `logBuffer` is flushed to the `logs` SQLite table outside the failed transaction. Logs emitted outside a transaction insert directly into `logs`.

Reference location: `web_interface` / DB worker or `cqrs_state_synchronization` logging subsystem.

To verify: Verify that the web interface worker handles transaction failure logging by preserving error logs outside rolled-back worker transactions.

---

### Structural Operations Squash-First Rule & Transaction Wrapping

Android: `splitBlock` and `mergeWithPreviousBlock` flush pending text edit diff buffers (`EditTextInBlock.flushBuffer`) before performing structural actions, and wrap both the flush and the structural change in a single re-entrant SQLite transaction (`DbManager.withTransaction`).

Reference location: `executor_of_what_the_builder_built_because_the_builder_shouldnt_touch_the_db` and `data_builder_for_operations_that_need_to_be_correct` in Rust.

To verify: Ensure that structural builders/executors on the Rust/web side enforce flushing pending text diffs before applying block splits/merges/deletions.

---

### Nav Menu Title Extraction from Title Block Content

Android: `NavPage` reads page titles from `every_block_in_existence` where `is_title = "true"` and `id_of_page_i_belong_to = page_id`. If empty or blank, displays "Untitled" in a muted color. Renaming from the nav menu calls `DbManager.renamePage` which updates the title block's content via `EditTextInBlock` / `AddBlock` builders.

Reference location: `web_interface` / Nav menu component in reference web UI.

To verify: Check that the web client UI displays the title block content as page title in navigation sidebar and edits title block text when renaming.

---

### SafeRowMapper Extension for `List<ColumnValue>`

Android: Extended `SafeRowMapper` to accept `List<ColumnValue>` returned directly by modern UniFFI row builders (`newPageRow`, `newEveryBlockInExistenceRow`, `newKeyValueItem`, `newLogRow`, etc.) and validate column names against table blueprints.

Reference location: `client_table_blueprints` in Rust / Kotlin UniFFI bindings.

To verify: Compare `SafeRowMapper` column name validation logic against UniFFI bindings output for row builders.

---

### External Transaction Hook for E2E Testing

Android: `TestTransactionReceiver` in `src/debug/` listens for three broadcast
actions and calls `DbManager.beginExternalTransaction()`,
`endExternalTransaction(commit = false)`, or
`endExternalTransaction(commit = true)`. While `externalTransactionActive` is
true, `withTransaction` runs its block inside the already-open transaction
instead of starting a nested one. Used by the external Python/Appium test
harness to wrap a whole test run in one transaction that rolls back at
teardown. Debug-only (source folder `src/debug/`), absent from release builds.

Reference location: none. This is testing infrastructure, not a domain
feature. If the web or Rust side ever gains an equivalent harness, it should
live in a similarly isolated test-only module.

To verify: N/A - this is a testing-apparatus choice, not a domain design.

---

### Popup Severity Convention (show vs showError)

Android: `GlobalPopupManager` has two entry points. `show(msg)` is for
informational popups; `showError(msg)` is for errors and is rendered with an
`ERROR: ` prefix in the snackbar text. Both are logged to logcat: `Log.i` for
info, `Log.e` for errors, tag `GlobalPopup`. The convention exists because the
external test harness needs to distinguish "the user should be told something"
from "the operation failed", and it reads popups from logcat rather than the
a11y tree.

Reference location: if the web client has an equivalent of "popup severity"
(it likely does - snackbar vs toast vs alert), this convention should match.
Check `cqrs_state_synchronization/web_interface` UI components.

To verify: confirm the web client distinguishes error notifications from
informational ones with the same semantic split (error -> user knows something
broke; info -> user is informed of an expected outcome).

---

### V3 migration drops and recreates every_block_in_existence

Android: `SchemaVersion.V2.updateOnce` performs the V2 to V3 migration
by dropping `every_block_in_existence` and recreating it. The `position`
REAL column became `frac_position` TEXT; no attempt is made to migrate
existing rows, so any block rows already on the device are lost on first
launch after the V3 install.

Reference location: none - this is a one-time project-state choice, not
a domain design. The web side accepted the same data loss (project is
pre-data and gets wiped).

To verify: N/A once the project has real data. Until then, this is
accepted data loss, not a design to keep.
