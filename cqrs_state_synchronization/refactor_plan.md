# Refactor Plan

## Status

Phase A (cqrs workspace) and Phase X (sibling crates X1-X4) are applied, plus the follow-ups the user approved (my_block_content split, payload regroup, receive-task restructure, remote-disable popup-only). All workspaces compile green: cqrs (three native crates), yrs, z_db/web_internal_db, client_table_blueprints, love_letter, zakken_server. web_interface checks on every trunk save.

Two small behaviour changes slipped in alongside the structural work; both are intentional and listed under "Behaviour changes".

## What was done

**Phase A — cqrs workspace**

Step 1 (error_stuff) — read only. No change.

Step 2 (builder crate) — read only. Already in target shape. No change.

Step 3 (executor) — `page_executor.rs` became a folder. `transaction.rs` has `run_in_transaction(work)` replacing the four macros. `operations.rs` has the seven `*_requires_three_db_*` fns, each a three-line body. `merge_plan.rs` has `apply_merge_plan` rewritten on `write_page_row`, `write_page_status`, `apply_reconcile_op`. `table_writes/{mod,pages,every_block,love_letter}.rs` with `set_column_where`, `insert_block_row`, `insert_block_row_and_get_db_id`. The four macros are gone.

W1 (web_interface db layer) — reads to `db/reads/`, writes to `db/writes.rs`, `db/yrs_errors.rs` for "no block with id" / "no meta". `ui_actions.rs` and `ui_actions_move_block_operations.rs` became `db/ui_actions/{mod,blocks,move_block,pages}.rs`. `cached_ids` gained `current_session_id`, `current_user_id`, `try_current_user_id`. `key_value_storage_db_actions` gained `set_user_id`. `dev_actions` gained `wipe_tables`, `delete_everything`, `reset_everything`. `kick_send` moved down to `db/kick_send.rs` so `db` no longer imports upward into `server_link`.

W2 (batcher + complicated_signal_flow) — Batcher split into `db/batcher/{mod,handle,run_loop,flush,exclusive}.rs`. `PendingEdit` renamed `PendingWrite`. `Batcher::flush` keeps its own begin/regret/commit. `complicated_signal_flow/` retained with `rows.rs` (BlockRow/LocalPages), `block_input_event.rs`, `ui_signals.rs`, `focus.rs`. `OpenPages` moved to `page_state.rs`.

W3 (main_content) — `MainContent` split. Effects to `page_loading_logic::install_load_effects`. Callbacks to `main_content/callbacks/`. `get_boss_from_page_id` / `page_for_block` / `find_page_and_boss` became `OpenPages::{boss_for_page, page_for_block, page_and_boss_for_block}`. Five `FOCUS_*.set(Some(..))` calls became `focus::at_end(id)` / `focus::at_start(id)`. `new_local_block_id` extracted to `src/new_local_block_id.rs`. `merge.rs::pick_boss_for_page` reuses `OpenPages::boss_for_page`.

W4 (server_link) — `From<DbError>` and `From<CqrsErr>` on `ApplyError`. `try_current_user_id` replaces hand-built "USER_ID not set" errors. `REMOTE_DISABLED_OPEN_PAGE` moved to `complicated_signal_flow::ui_signals` (and then deleted — see below).

W5 (black box + my_block_content) — `sync.rs` (~200 lines) split. Two detection loops became `single_block_change` / `one_block_diff` / `detect_single_extra` returning `SingleChange::{Removed, Added}`. `fast_delete`, `fast_add`, and the full rebuild all end on `finish_sync`. The 14 loose `RwSignal` params became a `Copy` struct `SyncState<'a>`. `my_block_content.rs` split after user approval: `on_keydown` became `handle_arrow` / `handle_enter` / `handle_backspace`, each `Effect::new` became an `install_*` fn, and the three "height auto, read scroll_height" snippets became `resize_textarea_to_content`. The "FAS" debug logs left in place.

**Post-W follow-ups (user-approved)**

- `server_link::receive` renames: `spawn_worker` → `spawn_task_for_receiving_socket_messages`, `run_worker` → `task_for_receiving_socket_messages`. Channel names now describe what crosses them (`tx_give_love_letter_that_came_from_server_to_task_that_processes_it` / `rx_take_love_letter_from_server_and_process_it`, and the reconnect pair).
- `start_over_fresh` restructure: login moved from once-at-task-start into the reconnect handler, so every connect runs login → kick_send → start_over_fresh (once, guarded by `START_OVER_FRESH_ATTEMPTED`) → pull_unsent. The `first_connected` flag was deleted because there's no longer a first-connect special case.
- `REMOTE_DISABLED_OPEN_PAGE` deleted. `apply_disable` in `merge.rs` calls `create_popup` directly if the disabled page is the one on screen. User keeps working; the page just won't appear in the menu anymore.
- `PagesToNavTo` (dead struct in `menu.rs`) deleted.

**Phase X — sibling crates**

X1 (yrs) — `yrs_wrapper.rs` became `yrs_wrapper/{mod,types,keys,errors,apply_update,block_lookup,frac_position,text_edit,snapshots,boss}.rs`. Three blanket allows removed. `generic_error!`, `lock_poisoned!`, `read_doc!`, `write_doc!`, `guarded!` macros replace the repeated `GenericError{ErrorInfo{file: file!(), method: ...}}` construction, the lock-poisoned map_errs, and the `prevent_deadlock(DeadlockCtx::new(...))` wrappers. `find_block` replaces three find-by-id loops. `string_field` replaces eight "get key, cast to String" copies. `apply_update_bytes` replaces three decode-and-apply-update copies. `edit_block` split into `edit_block_text` and `edit_block_meta`. `smart_frac_index` alias renamed `frac_core`. Stale comments removed. `yrs_active_pages.rs` and `yrs_backlinks.rs` got the same macro treatment. `yrs_error.rs` imports `FracIndexError` once, drops `source() -> None`. Public paths unchanged. The stale `edit_text_block_insert` method label kept for compatibility.

X2 (z_db) — `db_helper.rs`: 36 public fns collapsed from 4 lines each to one line each over four generic helpers. Public names, command strings, `javascript_im_begging_you`, and the long variable names untouched. `protocol/payload.rs` split into `payload/{create,read,update,delete,types}.rs` (user's preferred shape, not the plan's thematic split). `types.rs` holds `JoinType`, `SelectArgument`, `SelectArguments`, `ColumnValue`, `TableColumnInfo`, `TableExport`, `ColumnSource`. `pub use` at `payload/mod.rs` so `protocol::payload::*` still works.

X3 (client_table_blueprints) — `row_values.rs` with `text_value(&COL, v)` and `blob_value(&COL, v)` replaces the ~16 `ColumnValue` struct literals. `new_table_*` in every `tbl_*.rs` reads column names from the `SchemaColumn` consts. `use` lines moved to the top. `#![allow(non_snake_case)]` removed from every file.

X4 (love_letter) — `lib.rs` became a table of contents. `broken_heart.rs`, `sketch.rs`, `letter.rs`, `build_a_love_letter.rs`, `bytes.rs` split out. `uniffi::setup_scaffolding!` stays in `lib.rs`. Variant and field order frozen.

**Workspace-check changes**

- `Cargo.toml` no longer has `default-members` — it hid web_interface from rust-analyzer, which then fell back to the host target and lost `Router::new`.
- `local_cargo.sh` auto-adds `--workspace --exclude web_interface` for `check`/`build`/`test`/`clippy` when run from the workspace root with no scope argument (`-p`/`--package`/`--workspace`/`-w`/`--exclude`). `c c` from the root only checks the three native crates. web_interface still checks on trunk save (or from inside `web_interface/` where `.cargo/config.toml` sets wasm32).
- `.zed/settings.json` in `web_interface/` sets `cargo.target = "wasm32-unknown-unknown"` so rust-analyzer uses the wasm target when Zed's project root is `web_interface/`. A `.zed/settings.json` also exists at the cqrs root from an earlier attempt; harmless, invisible when the root is `web_interface/`.

## Behaviour changes (intentional)

1. The receive task logs in on every connect, not once at task start. The "first connect" case is no longer special.
2. A remote page-disable shows a popup and does not navigate the viewer away. The user keeps working; the page only disappears from the menu.

## Deliberately not applied

- `schema_helper.rs` macro for the four `DestructDbReturnCol` impls — user vetoed: "doesn't need a macro, it's not a lot of code and it's just easier to read as separate methods."
- `yrs/src/helper.rs` and `yrs/src/test.rs` — pre-existing fully-commented stubs, not declared in `lib.rs`, not compiled.
- `db_wrapper/src/web_output/db_helper.rs` — the copy-script target version of `db_helper.rs`. Not compiled, not read by anything. Its out-of-sync state predates this pass.

## Bugs found (do not fix in this pass)

1. `apply_merge_plan` rollback leak — the `?` on row-builders used to sit outside the rollback macro. Fixed as a side effect of `run_in_transaction`.
2. `Batcher::flush` takes pending before begin. If begin or commit fails, queued writes are lost and `APP_LOCKED` isn't set. Kept off `run_in_transaction` because begin-versus-join matters there.
3. `on_split` passes `is_part_of_main_menu_page = false` while create / move / merge derive it from the page.
4. `ui_actions::add_block` `kick_send` under `WRITE_LOCK` — fixed by the `run_exclusive` rewrite.
5. Same-page `move_block` loads a fresh boss from the db and edits that, not the open page's boss. If the builder snapshots the open boss for `pages_update.yrs_blob`, the next flush writes a blob without the move while the mirror keeps the new frac position.
6. `merge.rs::apply_blob` reads `existing_blocks` before `run_exclusive` (which flushes the batcher first). A queued `add_block` lands in the mirror after that read, so the reconcile can plan an Insert for a block the mirror already has. `apply_disable` reads the status blob the same way.
7. yrs `AtEnd` uses the last block in array order, not the highest frac key.
8. yrs text edits measure with `chars().count()`; `Doc::new()` defaults to byte offsets. Non-ASCII may misplace edits unless text_diff compensates.
9. yrs `insert_new_block` and `insert_new_block_with_preexisting_id` take the doc's read lock while writing; check-then-insert are two transactions. `edit_text_block` labels itself `"edit_text_block_insert"` in errors and deadlock ctx.
10. `create_new_page` writes the page and its title in two transactions.

## Ideas (not this pass)

- Central helpers for the text-boolean `"true"` / `"false"` comparison (~12 places across executor, web_interface, blueprints).
- `SchemaColumn` const constructors in z_db; error text names the column instead of `"illegal"`.
- `LoveLetter::target_page_id()` / `snapshot_of_edit()` accessors.
- Result-returning `current_session_id` / `current_user_id` everywhere.