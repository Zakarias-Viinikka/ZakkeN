# Refactor Plan

## Status

Phase A (cqrs workspace) is done. All four crates compile green from both checkpoints:
`cd cqrs_state_synchronization && c c` (three native crates) and
`cd cqrs_state_synchronization/web_interface && c c` (wasm32).

Phase X (sibling crates) is pending. X1-X4 need reads of files not yet seen this session.

## What was done

**Step 1 (error_stuff)** — read only. `CqrsErr` has `From<YrsError>`, `From<DbError>`, `From<BrokenHeart>`. No change.

**Step 2 (builder crate)** — read only. Already in target shape: one file per operation with ctx structs, `shared.rs` for the two records every op uses, `merge_page.rs` for all merge builders, `lib.rs` re-exports everything so the executor's glob import resolves. No change.

**Step 3 (executor)** — `page_executor.rs` (600 lines, one file) became a folder. `page_executor/mod.rs` re-exports so old paths resolve. `transaction.rs` has `run_in_transaction(work)` replacing the four macros. `operations.rs` has the seven `*_requires_three_db_*` fns, each a three-line body. `merge_plan.rs` has `apply_merge_plan` rewritten on `write_page_row`, `write_page_status`, `apply_reconcile_op`. `table_writes/{mod,pages,every_block,love_letter}.rs` has one `set_column_where`, one `insert_block_row`, one `insert_block_row_and_get_db_id` for the title's returned id. The four macros (`unwrap_or_bail_and_regret!`, `start_transaction_if_not_in_one!`, `end_transaction_if_i_own_the_transaction!`, unused `unwrap_or_bail!`) are gone — grep confirmed nothing outside cqrs used them.

**W1 (web_interface db layer)** — reads went to `db/reads/{mod,blocks,destruct,menu,pages}.rs`. Writes to `db/writes.rs` (`edit_block`, `add_block_unflushed`, `edit_title_for_page`). New `db/yrs_errors.rs` for "no block with id" / "no meta for block id". `ui_actions.rs` and `ui_actions_move_block_operations.rs` became `db/ui_actions/{mod,blocks,move_block,pages}.rs`. `cached_ids` gained `current_session_id`, `current_user_id`, `try_current_user_id`. `key_value_storage_db_actions` gained `set_user_id`. `dev_actions` gained `wipe_tables`, `delete_everything`, `reset_everything`. `Cargo.toml` gained `default-members` excluding web_interface so `c c` at the workspace root only checks the native crates.

**W2 (batcher + complicated_signal_flow)** — Batcher split into `db/batcher/{mod,handle,run_loop,flush,exclusive}.rs`. `PendingEdit` renamed `PendingWrite`. `Batcher::flush` keeps its own begin/regret/commit (not switched to `run_in_transaction`; claude's plan is explicit about why — bug 2 lives there). `complicated_signal_flow/` retained as the home of the plain types (claude's updated plan; also what "never rename complicated_signal_flow" implies — I'd briefly deleted it, then restored): `rows.rs` (BlockRow, LocalPages), `block_input_event.rs`, `ui_signals.rs` (APP_LOCKED, FOCUS_*, REMOTE_DISABLED_OPEN_PAGE), `focus.rs` (at_end, at_start). `OpenPages` moved to `page_state.rs` next to `Page`.

**W3 (main_content)** — `MainContent` (~300 lines) split. Effects moved to `page_loading_logic::install_load_effects(page1, batcher)`. Each callback became a file under `main_content/callbacks/` (`on_move`, `on_block_input`, `on_split`, `on_navigate`, `on_delete_request`, `on_merge_with_above`). `get_boss_from_page_id` / `page_for_block` / `find_page_and_boss` became `OpenPages::{boss_for_page, page_for_block, page_and_boss_for_block}`. Five `FOCUS_*.set(Some(..))` calls became `focus::at_end(id)` / `focus::at_start(id)`. `new_local_block_id` extracted to `src/new_local_block_id.rs`. `merge.rs::pick_boss_for_page` reuses `OpenPages::boss_for_page`.

**W4 (server_link)** — `kick_send` moved down to `db/kick_send.rs` (it was a poke, so it breaks the db → server_link cycle claude flagged). `send.rs` calls `install_kick_sender(tx)` from `spawn_sender`. `From<DbError>` and `From<CqrsErr>` on `ApplyError`, so merge.rs uses `?` instead of `.map_err(|e| ApplyError::Fatal(CqrsErr::DbErrorContainer(e)))`. `try_current_user_id` replaces the hand-built "USER_ID not set" errors. `REMOTE_DISABLED_OPEN_PAGE` moved to `complicated_signal_flow::ui_signals`. `apply_blob` / `apply_new_page` / `apply_disable` left alone per plan.

**W5 (black box only)** — `sync.rs` (~200 lines) split. Two detection loops became `single_block_change` / `one_block_diff` / `detect_single_extra` returning a `SingleChange::{Removed, Added}` enum. `fast_delete`, `fast_add`, and the full rebuild all end on `finish_sync` (the shared 7-line tail). The 14 loose `RwSignal` params became a `Copy` struct `SyncState<'a>`; the `too_many_arguments` allows on `fast_delete` / `fast_add` came off (only `fast_add` still has one, for `incoming_blocks` + `current_boxes` + `incoming_pids` + five state fields). `sync_from_props` signature narrowed to `(pages, blocks, state)`. `drag_reorder_black_box.rs` constructs the `SyncState` at the call site.

## Decisions made inside the plan (not in claude's question round)

- Bug 1 (apply_merge_plan rollback leak): the `run_in_transaction` rewrite in step 3 fixes it as a side effect. Kept, since it's a strict improvement and claude's plan lists the fix as the natural side effect.
- W5 `my_block_content.rs`: NOT touched. The file itself says "the file I own and edit for block behaviour", and claude's plan names it the user's file, but W5's description also says to split `on_keydown` and install effects. Contradiction. Left as-is; flagged here so it isn't lost.
- W2 `focus.rs`: created in W3, not W2 — it belongs there.
- W2 `block_types.rs` / `ui_signals.rs`: briefly made flat, restored to `complicated_signal_flow/` per claude's updated plan.
- `default-members` in Cargo.toml: added because otherwise `c c` at workspace root compiles web_interface for the host target and `Router::new` is cfg-gated to wasm32/android. Claude's plan asks that `web_interface` be checked from its own directory (where `.cargo/config.toml` sets wasm32); `default-members` makes `c c` at workspace root do the right thing without a wrapper.

## Not done (X1-X4, outside the workspace)

X1-X4 need reads of files not in this session's context. **X1 needs yrs**: `lib.rs`, `yrs_wrapper.rs`, `yrs_error.rs`, `yrs_active_pages.rs`, `yrs_backlinks.rs`, `anti_deadlock.rs`, `helper.rs`, `Cargo.toml`. X2 needs z_db `db_helper.rs` (seen), `payload.rs` (seen), `schema_helper.rs` (seen), `serialization.rs`, `new_table.rs`, `lib.rs`, and the native sqlite side. X3 needs the rest of client_table_blueprints. X4 needs love_letter's `lib.rs` (seen at ~200 lines in an earlier paste, but not this session).

## Bugs (do not fix in this pass)

1. `apply_merge_plan` rollback leak — fixed as a side effect of step 3 (see decisions above).
2. `Batcher::flush` takes pending before begin. If begin or commit fails, queued writes are lost and APP_LOCKED isn't set.
3. `on_split` passes `is_part_of_main_menu_page = false`, while create / move / merge derive it from the page. Will show in the planned sync checks.
4. `ui_actions::add_block` calls `kick_send` while holding `WRITE_LOCK`. Harmless because `kick_send` only pokes; the `run_exclusive` rewrite in W1 means `kick_send` is now called after the lock is released, so this is fixed.
5. Same-page `move_block` loads a fresh boss from the db and edits that, not the open page's boss. If the builder snapshots the open boss for `pages_update.yrs_blob`, the next flush writes a blob without the move while the mirror keeps the new frac position.
6. `merge.rs::apply_blob` reads `existing_blocks` before `run_exclusive` (which flushes the batcher first). A queued `add_block` lands in the mirror after that read, so the reconcile can plan an Insert for a block the mirror already has. `apply_disable` reads the status blob the same way.
7. yrs `AtEnd` uses the last block in array order, not the highest frac key.
8. yrs text edits measure with `chars().count()`; `Doc::new()` defaults to byte offsets. Non-ASCII may misplace edits unless text_diff compensates.
9. yrs `insert_new_block` and `insert_new_block_with_preexisting_id` take the doc's read lock while writing; check-then-insert are two transactions. `edit_text_block` labels itself `"edit_text_block_insert"` in errors and deadlock ctx.
10. `create_new_page` writes the page and its title in two transactions.

## Ideas (not this pass)

- Central helpers for the text-boolean `"true"` / `"false"` comparison (~12 places across executor, web_interface, blueprints).
- `SchemaColumn` const constructors in z_db; error text names the column instead of `"illegal"`.
- `LoveLetter::target_page_id()` / `snapshot_of_edit()` accessors; would shrink merge.rs's 5-arm pattern.
- Result-returning `current_session_id` / `current_user_id` everywhere (behaviour change).