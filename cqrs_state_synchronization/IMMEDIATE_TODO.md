# immediate todo

Two repos: z_db and cqrs_state_synchronization. Everything below is one step; do it together.

## z_db — expose the transaction check

`are_we_in_middle_of_transaction(conn)` already exists in `db/src/transactions.rs`. It is used by the migration macros. It has no worker command, so nothing outside db can reach it.

Add it as a command, the same four places every other command touches:

- `db_wrapper/src/macro_core.rs` — a method that calls `::db::transactions::are_we_in_middle_of_transaction`
- `db_wrapper/src/web_output/worker.js` — the command name mapping
- `web_internal_db/src/db_helper.rs` — the async fn the cqrs crate compiles against
- `db_wrapper/src/web_output/db_helper.rs` — the copy the web build ships

The input is nothing, the output is a bool. Then rebuild the wasm, run `update_hard_copied_z_db_web_output.sh`, and change the connection name back to `cqrs` in `worker_wrapper.js` if the copy overwrote it.

## cqrs — the load path gets the db id

`get_title_and_id_of_all_menu_pages` reads `every_block_in_existence` only. It returns the title and the yrs id. `LocalPages.id` is supposed to be the pages row id, which lives in `pages`. There is no join in the crud and there does not need to be one.

Read `pages` for `id` and `page_id`, match on `id_of_page_i_belong_to == page_id`, in Rust. Two reads, one per call.

## cqrs — create_new_page

- take `page_ctr` off the signature
- title is the placeholder `"title"`; duplicates are fine
- use `insert_data_and_get_col` with `column_to_return = "id"` so the new row's db id comes back
- return both ids: the yrs id, which the builder already has, and the db id from the insert
- check `are_we_in_middle_of_transaction` before calling `begin_all_or_nothing`. If already inside, skip the begin and skip the commit — the outer call owns them

`ui_actions` keeps its `local_pages_set` parameter, because real edits update it. Dev buttons pass a throwaway they never read.

## cqrs — insert_three_pages

The button calls create three times. With the transaction check in place, one outer transaction around all three works. Pick whichever of the two transaction shapes you want in z_db and use it here.

## cqrs — the dev sidebar

`SidebarContent` in `dev_panel/sidebar.rs` becomes a list of button components.

A new folder `dev_panel/sidebar_buttons/` holds one file per button. Each file is the component plus the one async fn it calls. The fn does the db work, then calls `window.location.reload()`.

Two buttons: delete everything, insert three pages. Nothing else.

No props, no shared signals. The reload re-reads from db, so a dev button never updates a signal.

## cqrs — delete

`delete_everything` loses its `ctr` and `local_pages_set` arguments. The db is the only state.

`page_ctr` goes away everywhere. So does `LocalPages` if nothing reads it anymore — check before removing.

`page_edits.rs` is gutted. `disable_page` lives there, does nothing wrong, and has no caller. Leave it somewhere reachable so the real disable UI can use it later. `ui_actions::disable_page` is the one that matters and it stays.

## Notes

- `create_page`, `edit_block`, `disable_page` are now in their own files in the builder crate. Shared ctx structs are in `shared.rs`. `PagesUpdateCtx.new_blobbed_page` was renamed to `yrs_blob` because it holds different things in different operations.
- The build compiles end to end as of this writing.
- Leave the unused-import warnings alone for now.
