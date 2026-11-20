# Invented on Android

Things Android does that have no Rust or web counterpart yet, or where the
Android answer is different from web for a phone-shaped reason. Each entry
says what Android does and where the equivalent should eventually live, so
it can be checked once the web side catches up.

When you design something without a reference, add an entry here.

---

### Parallel Column Schema Descriptors & Type Canary Tests

Android: `z.zndroid.db.schema` holds `ColumnDescriptor<T>` objects per table (name, typeName, unwrap(Col) -> T). `SchemaCanaryTest` asserts hardcoded type expectations per column as a compile/test time substitute for Rust's generic `SchemaColumn<T>`. Every read in `db/reads` goes through these descriptors. Positional column indexes still exist in StorageAccess, MigrationManager, DbManager, InspectTable and some tests (2026-11-13; see MISSING_AND_STALE.md).

Reference location: `client_table_blueprints` in Rust.

To verify: Compare `ColumnDescriptor` field types and `SchemaCanaryTest` assertions against the generic `SchemaColumn<T>` definitions in the Rust schema crate.

---

### In-Memory Transaction Log Buffer & Rollback Flush

Android: `DbTransactionElement` maintains a coroutine-context in-memory log buffer (`logBuffer`). Logs emitted via `Logger` during an active transaction are appended to `logBuffer`. On commit, `logBuffer` is cleared without writing. On rollback, `logBuffer` is flushed to the `logs` SQLite table outside the failed transaction. Logs emitted outside a transaction insert directly into `logs`. Known gap (2026-11-13): `Logger` locates the open transaction through a ThreadLocal, not the coroutine context, so a thread switch inside a transaction defeats the buffering.

Reference location: web's DB worker. Whether web preserves error logs from a rolled-back transaction and where they end up should be checked.

To verify: Confirm web's worker handles transaction-failure logging by preserving error logs outside rolled-back transactions.

---

### PageBatcher with optimistic delete/move

Android: `editor/PageBatcher` holds an ordered queue of `Entry.Edit / Add / Delete / Move`. Debounce (`debounceJob`, guarded by `debounceLock`) cancels only the 500ms sleep coroutine and launches each flush in its own coroutine so a new push cannot cancel an in-flight transaction. Consecutive edits to a block are squashed via `combineGetdiffResults` before the transaction; any non-edit entry flushes the pending edit run first, then does its own op inside the same transaction. Local UI state (`PageSession.rows`) is updated optimistically for delete and move before the write lands, and `PageSession.applyRemoteRows` flushes pending edits before updating rows in place on Main.

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

(Note 2026-11-20: a drag/cascade physics engine now exists in the app as an invisible foundation, see "Invisible cascade engine" below. There is still NO drag handle and no manual dragging; Up/Down in the block menu is still how the user reorders.)

Android: block reorder is two buttons in the block action menu (opened by tapping the `::` handle, not a long-press) ("Up" / "Down"). No drag. The title can't move and nothing moves above it.

Web: drag the block handle to reorder, implemented by `drag_reorder_container`.

Reference location: `cqrs_state_synchronization/web_interface/src/drag_reorder_container`.

To verify: On a phone, drag-to-reorder competes with text selection and the scroll gesture. This is a UX choice, not a bug. Kept as a note so a future mobile-drag experiment can replace it if it works.

---

### Session id per process, not per navigation

Android: `SessionManager.initialize` reads the last stored id and sets `currentSessionId` to that + 1, synchronously, and persists it. Navigation does not bump it — `MainActivity` no longer registers a destination-changed listener. A read error or unparsable stored value leaves the counter alone rather than resetting to 0. `SessionMonotonicityTest` asserts it stays stable during a session.

Web: `init_session_id` in `db/init.rs` increments on each page reload.

Reference location: `cqrs_state_synchronization/web_interface/src/db/init.rs`.

To verify: Web reloads on navigation (browser refresh), so its "once per reload" and Android's "once per process" are the same shape conceptually. Android does not reload on navigation, so incrementing per navigation would mean an idle tap between pages burns a session id with no writes in it.

---

### (Superseded 2026-11-20, see "Eye hides the page after a drop" below) Nav "X" (Delete) button means disable

Android: the nav row button labelled `"X"` opens a "Delete Page" dialog (confirm button `"Delete"`) which calls `actions/disablePage`, setting the page's blocks and status to disabled and removing the page from the menu. No data is deleted.

Web: the menu button next to each title is an eye icon (SVG) labelled with the tooltip "disable page". Same behaviour.

Reference location: `web_interface/src/leptos_components/action_buttons/disable_page_button.rs`.

To verify: any future UI redesign can re-label the button. Nothing depends on the label except the e2e test text matcher.

---

### Dev Tools page + DevCheck registry

Android: page at route `test_results` (nav button "Dev Tools"). Both the nav button and the route are gated behind `BuildConfig.DEBUG` in `MainActivity`, so the page is unreachable in release even though its class is compiled in. `DevCheckRegistry.checks` is the one list of checks (`PASS`, `FAIL`, or `SKIP`). `DevToolsPage` runs them sequentially. Each logs `Log.i("DevCheck", "id=... status=PASS|FAIL|SKIP detail=...")`; the last line is `Log.i("DevCheck", "summary pass=N fail=M skip=K")`. `test_dev_checks` in the e2e harness navigates to the page, taps "Run all checks", waits for the summary line, asserts `fail=0`, prints every failed check's id and message.

Web: `web_interface/src/db/dev_actions.rs` has `insert_three_pages`, `delete_everything`, `reset_everything` exposed as buttons in a dev sidebar. No pass/fail check registry.

Reference location: `cqrs_state_synchronization/web_interface/src/db/dev_actions.rs` and `web_interface/src/leptos_components/dev_panel`.

To verify: If web ever wants an equivalent check registry, the shape here is the template: one list, one runner, one summary log line that e2e can read.

---

### Server sync via uniffi foreign-trait listener

Android: Rust owns the websocket (`shared_internals::socket_android::spawn`). Kotlin implements `LinkListener`, an uniffi `with_foreign` trait. Rust calls it from a background thread; Kotlin's implementation only does `trySend` on a `Channel`, then returns immediately.

Web: the spontaneous-push callback is a wasm-bindgen `Closure` that runs on the JS thread, so there is no thread handoff and no channel.

Reference location: `zakken_server/client_internals/server_link/src/lib.rs` (`forward_spontaneous`), `android_app/app/src/main/java/z/zndroid/sync/owner_of_interacting_with_server/SyncListener.kt`.

To verify: N/A — this is the correct uniffi shape for a foreign callback. No behavioural parity check needed.

---

### SpontaneousInbox channels

Android: `SpontaneousInbox.kt` holds the Channels that carry pushes from the Rust callback thread into the receive loop's coroutine. Four channels: `letters` (UNLIMITED, one per incoming letter), `socketReconnected` (CONFLATED, "log in again"), `loggedIn` (CONFLATED, "pull unsent"), `driftChanged` (UNLIMITED, one per drift push).

Web: no equivalent. Wasm runs on one JS thread, so the receive loop reads directly from the router's response stream.

Reference location: `android_app/app/src/main/java/z/zndroid/sync/owner_of_interacting_with_server/SpontaneousInbox.kt`.

To verify: N/A — Kotlin-thread plumbing.

---

### Auto-mint server_url

Android: `connectToSocket()` reads `server_url` from `key_value_storage`. If unset, it mints `ws://127.0.0.1:3000/ws`, stores it (so it's visible and editable via Database Inspector), and uses it. With `adb reverse tcp:3000 tcp:3000` set up, a fresh install just connects.

Web: hard-codes `ws://127.0.0.1:<port>/ws` in `owner_of_interacting_with_server/core.rs` because the browser and server share a machine.

Reference location: `android_app/app/src/main/java/z/zndroid/sync/owner_of_interacting_with_server/Core.kt` (`mintServerUrl`), `cqrs_state_synchronization/web_interface/src/owner_of_interacting_with_server/core.rs`.

To verify: fresh install with `adb reverse` set up should connect without touching Database Inspector.

---

### recoverUnknownPage

Android: when a letter arrives for a page the client doesn't have, `ReceiveLoop.recoverUnknownPage` fetches the page via `getPagesByIds`, applies it as `CreateNewPage` (and `DisablePage` when disabled), then re-applies the letter. If the server doesn't have the page either, the letter is acked and dropped (persisted in `seen_server_letters` so it doesn't replay).

Web: same logic in `main_for_applying_love_letter_from_server_to_app.rs`, no separate function name.

Reference location: `android_app/app/src/main/java/z/zndroid/sync/owner_of_interacting_with_server/ReceiveLoop.kt` (`recoverUnknownPage`), web's `main_for_applying_love_letter_from_server_to_app.rs`.

To verify: two clients, one creates a page, the other receives an edit for it before its own bootstrap has pulled. Compare with web's behavior on the same edge.

---

### Auto-mint default drift version on first connect

Android: fresh install has no `client_drift_version`, so `afterLogin` accepts the server's version and stores it. Lonewolf fires only when a stored version disagrees with the server's.

Web: same logic, `owner_of_interacting_with_server/main_for_login_and_registration.rs`.

Reference location: `android_app/app/src/main/java/z/zndroid/sync/owner_of_interacting_with_server/Login.kt` (`afterLogin`).

To verify: N/A — mirrors web.

---

### External transaction hook for e2e testing

Android: `TestTransactionReceiver` in `src/debug/` listens for three broadcast actions and calls `DbManager.beginExternalTransaction()`, `endExternalTransaction(commit = false)`, or `endExternalTransaction(commit = true)`. While `externalTransactionActive` is true, `withTransaction` runs its block inside the already-open transaction instead of starting a nested one. Used by the external Python/Appium test harness to wrap a whole test run in one transaction that rolls back at teardown. The receiver is debug-only (`src/debug`). `beginExternalTransaction`, `endExternalTransaction` and the `externalTransactionActive` branch in `withTransaction` live in `DbManager` in the main source set and ship in release (nothing in release calls them). In this mode a failing block is not rolled back: there is no savepoint, so partial writes from a failed batch stay in the open transaction until teardown, and the log buffer's rollback flush never runs. The e2e run therefore exercises the success path only.

Reference location: none. This is testing infrastructure, not a domain feature. A matching "server test mode" (see root TODO) is planned for the server.

To verify: N/A - this is a testing-apparatus choice, not a domain design.

---

### createNewPage emits two love letters

Android: `actions/createNewPage` inserts the page (page row, title block, body block, and a CreateNewPage letter), then runs `editBlock` on the title with `getDiff("", name)`, which inserts a second letter (EditBlock). Both happen in one transaction.

Web: `create_page` emits exactly one letter, a CreateNewPage carrying a title-bearing snapshot.

Reference location: web's `create_page` and the builder's `create_page`.

To verify: both orderings converge on the server, but Android sends two round-trips where web sends one. Decide whether the builder should take the initial title so one letter is enough.

---

### Eye hides the page after a drop

Android: the main-menu row button is an eye (content-desc `Hide <title>`), like web's disable button. No confirmation dialog. Press: eye closes, row dims; 1.5 s later the row plays a drop animation; only after it ends does `disablePage` run. Pressing the eye again before the drop starts cancels the hide (Android invention). Leaving the menu during the wait cancels it silently. If `disablePage` fails the row springs back and an error popup shows.

Web: eye button with tooltip "disable page", immediate, no animation.

Reference location: `web_interface/src/leptos_components/action_buttons/disable_page_button.rs`; Android `MainPages/NavRow.kt`.

To verify: the 1.5 s delay is Zakke's ask ("1.5 sec after the eye has been pressed"); decide whether web should get the same delayed drop.

---

### Ghost rows for delete animation

Android: a deleted block is removed from `PageSession.rows` and its DB delete is pushed immediately (so indices, focus and IME behave exactly as before). For the drop animation a display-only `GhostRow` (text plus the id of the block above) is added to `PageSession.ghosts` and drawn by `GhostBlock`; it removes itself when its animation ends. Backspace merge spawns no ghost.

Web: no delete animation.

Reference location: `editor/PageSession.kt` (`spawnGhost`), `MainPages/ViewPage/GhostBlock.kt`.

To verify: N/A, presentation only. Never let ghosts into `rows` or the batcher.

---

### Indent transition on first typed character

Android: the row's indent settle animation (22dp to 0, 500 ms, -5.5dp at 200 ms) plays when the first character is typed into an empty non-title block (`BlockRow.indentGen`), not on Enter.

Web: none.

Reference location: `PageSession.onInput`, `EditableBlock.kt`; lab `BlockTransitionLab`.

To verify: Zakke's ask. Remote edits and multiline paste deliberately do not trigger it.

---

### Invisible cascade engine in ViewPage

Android: blocks are positioned by a Rust physics engine (`rapier2d_wrapper`, prebuilt `.so` copied from `android_labs`) through `cascade/PageCascade` and `CascadeColumn`, so inserts, deletes and moves make the blocks below fall into place. There is no drag handle and no manual dragging. ViewPage is a scrolling `Column`, not a LazyColumn, so every block is composed.

Web: `web_interface/src/drag_reorder_container` is a visible drag-and-drop component (a different engine, with handles).

Reference location: `android_app/app/src/main/java/z/zndroid/cascade/`, lab `notes/CascadeController.kt`.

To verify: never run on a device when written (2026-11-20). Check typing, wrapping lines, long pages and IME resize first.

---

### Settings page behind a cog

Android: main-menu top bar has a cog opening `SettingsPage`. Debug builds list Recovery, Database Inspector and Dev Tools there (they were NavPage buttons). Web has a dev sidebar instead.

Reference location: `MainPages/SettingsPage.kt`.

To verify: N/A, Zakke's ask. Add real settings sections here later.

---

### V3 migration drops and recreates `every_block_in_existence`

Android: `SchemaVersion.V2.updateOnce` performs the V2 to V3 migration by dropping `every_block_in_existence` and recreating it. `position` REAL became `frac_position` TEXT; no attempt is made to migrate existing rows, so block rows already on the device are lost on first launch after V3.

Reference location: none - one-time project-state choice. Web accepted the same data loss.

To verify: N/A once the project has real data. Until then, this is accepted data loss, not a design to keep.