# Refactor Plan

## Status

All of Phase A (cqrs workspace) and Phase X (sibling crates X1-X4) are applied. Both workspaces compile green:

- `cd cqrs_state_synchronization && c c` — three native crates (web_interface is excluded by `default-members`).
- `cd cqrs_state_synchronization/web_interface && c c` — checks on every trunk save.
- `cd yrs && c c` — X1.
- `cd z_db/web_internal_db && c c` — X2.
- `cd client_table_blueprints && c c` — X3.
- `cd love_letter && c c` — X4.
- `cd zakken_server && c c` — all of the above downstream.

## What was done

**Phase A — cqrs workspace**

Step 1 (error_stuff) — read only. `CqrsErr` has `From<YrsError>`, `From<DbError>`, `From<BrokenHeart>`. No change.

Step 2 (builder crate) — read only. Already in target shape: one file per operation with ctx structs, `shared.rs` for the two records every op uses, `merge_page.rs` for all merge builders, `lib.rs` re-exports everything so the executor's glob import resolves. No change.

Step 3 (executor) — `page_executor.rs` (600 lines, one file) became a folder. `page_executor/mod.rs` re-exports so old paths resolve. `transaction.rs` has `run_in_transaction(work)` replacing the four macros. `operations.rs` has the seven `*_requires_three_db_*` fns, each a three-line body. `merge_plan.rs` has `apply_merge_plan` rewritten on `write_page_row`, `write_page_status`, `apply_reconcile_op`. `table_writes/{mod,pages,every_block,love_letter}.rs` has one `set_column_where`, one `insert_block_row`, one `insert_block_row_and_get_db_id` for the title's returned id. `web_interface/src/db/dev_actions.rs` updated to use `run_in_transaction`. The four macros are gone — grep confirmed nothing outside cqrs used them.

W1 (web_interface db layer) — reads went to `db/reads/{mod,blocks,destruct,menu,pages}.rs`. Writes to `db/writes.rs` (`edit_block`, `add_block_unflushed`, `edit_title_for_page`). New `db/yrs_errors.rs` for "no block with id" / "no meta for block id". `ui_actions.rs` and `ui_actions_move_block_operations.rs` became `db/ui_actions/{mod,blocks,move_block,pages}.rs`. `cached_ids` gained `current_session_id`, `current_user_id`, `try_current_user_id`. `key_value_storage_db_actions` gained `set_user_id`. `dev_actions` gained `wipe_tables`, `delete_everything`, `reset_everything`. `Cargo.toml` gained `default-members` excluding web_interface so `c c` at the workspace root only checks the native crates.

W2 (batcher + complicated_signal_flow) — Batcher split into `db/batcher/{mod,handle,run_loop,flush,exclusive}.rs`. `PendingEdit` renamed `PendingWrite`. `Batcher::flush` keeps its own begin/regret/commit (not `run_in_transaction` — bug 2 lives there). `complicated_signal_flow/` retained as the home of the plain types (rows.rs for BlockRow/LocalPages, block_input_event.rs, ui_signals.rs, focus.rs). `OpenPages` moved to `page_state.rs` next to `Page`.

W3 (main_content) — `MainContent` (~300 lines) split. Effects moved to `page_loading_logic::install_load_effects(page1, batcher)`. Each callback became a file under `main_content/callbacks/` (`on_move`, `on_block_input`, `on_split`, `on_navigate`, `on_delete_request`, `on_merge_with_above`). `get_boss_from_page_id` / `page_for_block` / `find_page_and_boss` became `OpenPages::{boss_for_page, page_for_block, page_and_boss_for_block}`. Five `FOCUS_*.set(Some(..))` calls became `focus::at_end(id)` / `focus::at_start(id)`. `new_local_block_id` extracted to `src/new_local_block_id.rs`. `merge.rs::pick_boss_for_page` reuses `OpenPages::boss_for_page`.

W4 (server_link) — `kick_send` moved down to `db/kick_send.rs` (it was a poke, so it breaks the db → server_link cycle). `send.rs` calls `install_kick_sender(tx)` from `spawn_sender`. `From<DbError>` and `From<CqrsErr>` on `ApplyError`, so merge.rs uses `?` instead of `.map_err(|e| ApplyError::Fatal(CqrsErr::DbErrorContainer(e)))`. `try_current_user_id` replaces the hand-built "USER_ID not set" errors. `REMOTE_DISABLED_OPEN_PAGE` moved to `complicated_signal_flow::ui_signals`.

W5 (black box only) — `sync.rs` (~200 lines) split. Two detection loops became `single_block_change` / `one_block_diff` / `detect_single_extra` returning a `SingleChange::{Removed, Added}` enum. `fast_delete`, `fast_add`, and the full rebuild all end on `finish_sync`. The 14 loose `RwSignal` params became a `Copy` struct `SyncState<'a>`. `sync_from_props` signature narrowed to `(pages, blocks, state)`. `drag_reorder_black_box.rs` constructs the `SyncState` at the call site.

**Phase X — sibling crates**

X1 (yrs) — `yrs_wrapper.rs` (~700 lines) became `yrs_wrapper/{mod,types,keys,errors,apply_update,block_lookup,frac_position,text_edit,snapshots,boss}.rs`. Three blanket allows (`dead_code`, `unused_variables`, `unused`) removed. `generic_error!`, `lock_poisoned!`, `read_doc!`, `write_doc!`, `guarded!` macros replace the repeated `GenericError{ErrorInfo{file: file!(), method: ...}}` construction, the "lock poisoned" map_err, and the `prevent_deadlock(DeadlockCtx::new(name, file!(), ProbablyJustADeadlock), closure)` wrapper. Three find-by-id loops (`get_block_index_from_id`, `get_block_from_id`, `find_block_with_id`) collapsed to one `find_block` returning `(u32, MapRef)`. `string_field` replaces the eight copies of "get key, cast to String". `apply_update_bytes` replaces the decode-and-apply-update code that appeared three times (`merge_with_snapshot`, `merge_with`, `doc_from_snapshot`). `edit_block` (Text and Meta) split into `edit_block_text` and `edit_block_meta`. `smart_frac_index` import alias renamed to `frac_core` (was shadowing the crate name). Stale comments removed ("Helpers (defined after impl...)", "Use new helper" ×2, the misplaced "returns a list of blocks ordered by frac_index" doc on `insert_new_block_with_preexisting_id`). `yrs_error.rs` imports `FracIndexError` once; explicit `source()` returning `None` dropped. `yrs_active_pages.rs` and `yrs_backlinks.rs` gained the same macro treatment. `pub` paths unchanged: `my_yrs_lib::yrs_wrapper::{BossOfYrs, TextEdit, EditTarget, PositionToInsert, Block}` and the crate-root re-exports (`generate_unique_key`, `doc_from_snapshot`, `create_bookmark`, `generate_diff_snapshot`) still resolve because `yrs_wrapper/mod.rs` re-exports them.

X2 (z_db) — `db_helper.rs`: 36 public fns collapsed from 4 lines each to one line each over four small generic helpers (`no_input_no_output`, `no_input_with_output`, `input_no_output`, `input_with_output`). Command strings, public names, `javascript_im_begging_you`, and the long variable names untouched. **Not applied**: `payload.rs` regroup (claude's plan says ask first — mechanical churn over ~50 types, low benefit) and `schema_helper.rs` macro (same). `CureFail`'s stale Display text mentioned in the plan, not edited.

X3 (client_table_blueprints) — `row_values.rs` with `text_value(&COL, v)` and `blob_value(&COL, v)` replaces the ~16 `ColumnValue { column_name, value }` struct literals. `new_table_*` in every tbl_ file now reads column names from the `SchemaColumn` consts instead of repeating the strings. `use` lines mid-file moved to the top. `#![allow(non_snake_case)]` removed from every file (it was never needed there). Exported signatures unchanged, including the `Result<_, YrsError>` on the two rows that can't fail.

X4 (love_letter) — `lib.rs` became a table of contents. `broken_heart.rs`, `sketch.rs`, `letter.rs`, `build_a_love_letter.rs`, `bytes.rs` split out. `uniffi::setup_scaffolding!` stays in `lib.rs`. Variant and field order frozen (the wire/FFI rule). All public paths resolve through `lib.rs` re-exports.

## Deliberately not applied (documented per the plan)

- **`payload.rs` regroup** into `payload/` submodules — the plan marks this "ask first, low priority". Skipped.
- **`schema_helper.rs` macro** for the four `DestructDbReturnCol` impls — the plan marks this "ask first". Skipped.
- **`my_block_content.rs` split** — W5's description says to split `on_keydown` into `handle_arrow` / `handle_enter` / `handle_backspace` and each effect into `install_*` fns, but the file itself says "the file I own and edit for block behaviour" and the plan names it the user's file. Contradiction. Left untouched.
- **`yrs/src/helper.rs` and `yrs/src/test.rs`** — both pre-existing fully-commented-out stubs, neither declared in `lib.rs`, neither compiled. Left untouched.
- **`db_wrapper/src/web_output/db_helper.rs`** — the copy-script-target version of `db_helper.rs`. Nothing compiles or reads it; the fact that it's slightly out of sync with `web_internal_db/src/db_helper.rs` predates this pass. No `build_web_and_android.sh` / `update_hard_copied_z_db_web_output.sh` run was needed because the JS + wasm boundary didn't change.

## Bugs found (do not fix in this pass)

1. `apply_merge_plan` rollback leak — the `?` on row-builders used to sit outside the rollback macro, so a failure left our transaction open. Fixed as a side effect of `run_in_transaction` in step 3.
2. `Batcher::flush` takes pending before begin. If begin or commit fails, queued writes are lost and `APP_LOCKED` isn't set. Left as-is (claude's plan says keep `flush` off `run_in_transaction` because begin-versus-join matters there).
3. `on_split` passes `is_part_of_main_menu_page = false` while create / move / merge derive it from the page. Will show in the planned sync checks.
4. `ui_actions::add_block` called `kick_send` while holding `WRITE_LOCK` — fixed by the `run_exclusive` rewrite in W1; `kick_send` is now called after the lock is released.
5. Same-page `move_block` loads a fresh boss from the db and edits that, not the open page's boss. If the builder snapshots the open boss for `pages_update.yrs_blob`, the next flush writes a blob without the move while the mirror keeps the new frac position.
6. `merge.rs::apply_blob` reads `existing_blocks` before `run_exclusive` (which flushes the batcher first). A queued `add_block` lands in the mirror after that read, so the reconcile can plan an Insert for a block the mirror already has. `apply_disable` reads the status blob the same way.
7. yrs `AtEnd` uses the last block in array order, not the highest frac key.
8. yrs text edits measure with `chars().count()`; `Doc::new()` defaults to byte offsets. Non-ASCII may misplace edits unless text_diff compensates.
9. yrs `insert_new_block` and `insert_new_block_with_preexisting_id` take the doc's read lock while writing; check-then-insert are two transactions. `edit_text_block` labels itself `"edit_text_block_insert"` in errors and deadlock ctx (kept for compatibility, but it's stale).
10. `create_new_page` writes the page and its title in two transactions.

## Ideas (not this pass)

- Central helpers for the text-boolean `"true"` / `"false"` comparison (~12 places across executor, web_interface, blueprints).
- `SchemaColumn` const constructors in z_db; error text names the column instead of `"illegal"`.
- `LoveLetter::target_page_id()` / `snapshot_of_edit()` accessors — would shrink merge.rs's 5-arm pattern and the server's copy.
- Result-returning `current_session_id` / `current_user_id` everywhere (behaviour change).