# Invented on Android

Things Android does that have no Rust or web counterpart yet, or where the
Android answer is different from web for a phone-shaped reason. Each entry
says what Android does and where the equivalent should eventually live, so
it can be checked once the web side catches up.

When you design something without a reference, add an entry here.

---

### Parallel Column Schema Descriptors & Type Canary Tests

Android: `z.zndroid.db.schema` holds `ColumnDescriptor<T>` objects per table (name, typeName, unwrap(Col) -> T). `SchemaCanaryTest` asserts hardcoded type expectations per column as a compile/test time substitute for Rust's generic `SchemaColumn<T>`. Every read in `db/reads` goes through these descriptors; there are no positional column indexes elsewhere.

Reference location: `client_table_blueprints` in Rust.

To verify: Compare `ColumnDescriptor` field types and `SchemaCanaryTest` assertions against the generic `SchemaColumn<T>` definitions in the Rust schema crate.

---

### In-Memory Transaction Log Buffer & Rollback Flush

Android: `DbTransactionElement` maintains a coroutine-context in-memory log buffer (`logBuffer`). Logs emitted via `Logger` during an active transaction are appended to `logBuffer`. On commit, `logBuffer` is cleared without writing. On rollback, `logBuffer` is flushed to the `logs` SQLite table outside the failed transaction. Logs emitted outside a transaction insert directly into `logs`.

Reference location: web's DB worker. Whether web preserves error logs from a rolled-back transaction and where they end up should be checked.

To verify: Confirm web's worker handles transaction-failure logging by preserving error logs outside rolled-back transactions.

---

### PageBatcher with optimistic delete/move

Android: `editor/PageBatcher` holds an ordered queue of `Entry.Edit / Add / Delete / Move`. Consecutive edits to a block are squashed via `combineGetdiffResults` before the transaction; any non-edit entry flushes the pending edit run first, then does its own op inside the same transaction. Local UI state (`PageSession.rows`) is updated optimistically for delete and move before the write lands.

Web: `db/batcher` queues only `Edit` and `AddBlock`. Delete and move go straight through `ui_actions` entry points (`delete_block`, `move_block`, `move_block_to_another_page`) which flush the batcher first, then take the write lock. Web does not queue them.

Reference location: `cqrs_state_synchronization/web_interface/src/db/batcher` and `web_interface/src/db/ui_actions`.

To verify: Decide whether Android's queue-them or web's flush-then-act is correct. Android needs the queue because touching a hardware keyboard can produce Enter-split while the user is still typing in another block, and a structural op that flushes immediately would block that other block's edit.

---

### Flush-failure reload instead of `APP_LOCKED`

Android: if a flush inside `PageBatcher.flushNow` fails, the transaction rolls back, the in-memory boss is now ahead of the DB, so `PageSession` rebuilds everything from the DB (destroy the boss, drop the queue) and shows a popup.

Web: on a failed flush, `APP_LOCKED` flips to `true`, the page background turns red, and the user is told to copy anything they need and refresh. Web can't force a reload because the user might lose in-memory state they care about.

Reference location: `cqrs_state_synchronization/web_interface/src/db/batcher/flush.rs` (the `lock_app_after_failed_flush` path).

To verify: Web can't auto-reload, so the Android answer will always differ here. Kept as a note so a future UI rewrite can decide whether to also introduce a visible APP_LOCKED-style state on Android instead of an automatic reload.

---

### Move Up / Down instead of drag

Android: block reorder is two buttons in the long-press block action menu ("Up" / "Down"). No drag. The title can't move and nothing moves above it.

Web: drag the block handle to reorder, implemented by `drag_reorder_container`.

Reference location: `cqrs_state_synchronization/web_interface/src/drag_reorder_container`.

To verify: On a phone, drag-to-reorder competes with text selection and the scroll gesture. This is a UX choice, not a bug. Kept as a note so a future mobile-drag experiment can replace it if it works.

---

### Session id per process, not per navigation

Android: `SessionManager.initialize` reads the last stored id and increments once per process start. `MainActivity` no longer increments on every navigation change. `SessionMonotonicityTest` asserts it stays stable during a session.

Web: `init_session_id` in `db/init.rs` increments on each page reload.

Reference location: `cqrs_state_synchronization/web_interface/src/db/init.rs`.

To verify: Web reloads on navigation (browser refresh), so its "once per reload" and Android's "once per process" are the same shape conceptually. Android does not reload on navigation, so incrementing per navigation would mean an idle tap between pages burns a session id with no writes in it.

---

### Nav "Delete" button means disable

Android: the nav row button labelled "Delete" calls `actions/disablePage`, which sets the page's blocks and status to disabled and removes the page from the menu. No data is deleted. Renamed conceptually but the button label is frozen because the e2e harness (and other tests) click the "Delete" text.

Web: the menu button next to each title is an eye icon (SVG) labelled with the tooltip "disable page". Same behaviour.

Reference location: `web_interface/src/leptos_components/action_buttons/disable_page_button.rs`.

To verify: any future UI redesign can re-label the button. Nothing depends on the label except the e2e test text matcher.

---

### Dev Tools page + DevCheck registry

Android: debug-only page at route `test_results` (nav button "Dev Tools"). `DevCheckRegistry.checks` is the one list of checks. `DevToolsPage` runs them sequentially. Each logs `Log.i("DevCheck", "id=... status=PASS|FAIL detail=...")`; the last line is `Log.i("DevCheck", "summary pass=N fail=M")`. `test_dev_checks` in the e2e harness navigates to the page, taps "Run all checks", waits for the summary line, asserts `fail=0`, prints every failed check's id and message.

Web: `web_interface/src/db/dev_actions.rs` has `insert_three_pages`, `delete_everything`, `reset_everything` exposed as buttons in a dev sidebar. No pass/fail check registry.

Reference location: `cqrs_state_synchronization/web_interface/src/db/dev_actions.rs` and `web_interface/src/leptos_components/dev_panel`.

To verify: If web ever wants an equivalent check registry, the shape here is the template: one list, one runner, one summary log line that e2e can read.

---

### External transaction hook for e2e testing

Android: `TestTransactionReceiver` in `src/debug/` listens for three broadcast actions and calls `DbManager.beginExternalTransaction()`, `endExternalTransaction(commit = false)`, or `endExternalTransaction(commit = true)`. While `externalTransactionActive` is true, `withTransaction` runs its block inside the already-open transaction instead of starting a nested one. Used by the external Python/Appium test harness to wrap a whole test run in one transaction that rolls back at teardown. Debug-only; absent from release builds.

Reference location: none. This is testing infrastructure, not a domain feature. A matching "server test mode" (see root TODO) is planned for the server.

To verify: N/A - this is a testing-apparatus choice, not a domain design.

---

### V3 migration drops and recreates `every_block_in_existence`

Android: `SchemaVersion.V2.updateOnce` performs the V2 to V3 migration by dropping `every_block_in_existence` and recreating it. `position` REAL became `frac_position` TEXT; no attempt is made to migrate existing rows, so block rows already on the device are lost on first launch after V3.

Reference location: none - one-time project-state choice. Web accepted the same data loss.

To verify: N/A once the project has real data. Until then, this is accepted data loss, not a design to keep.