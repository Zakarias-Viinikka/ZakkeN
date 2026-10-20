# Current state

- web_interface is a Leptos csr app, built and served with trunk.
- Routes: `/` and `/Menu` (Menu, a nav page), `/DbGui` (the orm, comes
  from the web_internal_db crate in z_db).

- create_page in the builder works. It builds a BossOfYrs, inserts two blocks (title, then a normal one), snapshots the doc, takes a version bookmark, and returns all the prepared data holding three ctx structs.

- the executor takes all the prepared data and does the three inserts in one transaction: pages, every_block_in_existence, uncommitted_diffs. The executor fns check whether a transaction is already open; if one is, they don't begin or commit, and the outer call owns it.

- there's a method for editing a block, and it's also used for adding a title to a fresh page.

- disable_page marks a page disabled instead of deleting it. The page row stays, every block for it gets page_is_disabled = "true", and a new active_pages blob goes into pages.page_status. The menu filters disabled pages out on load.

- the menu reads its page list from db on load. LocalPages holds db_id, title, block_id, page_id, is_disabled. There's no page counter anymore.

- the dev sidebar has two buttons: delete everything and insert three pages. Both do their db work, then reload the page.

- the eye button next to each menu title disables that page.

- main.rs creates 3 tables if they don't exist.

## Update — 2026-10-19

Routes: `/`, `/Menu`, `/DbGui`, and `/MainContent/:page_id` (reads the page id from the URL).

`App` in main.rs gates the whole router behind table creation, so a direct URL load doesn't race the tables.

New crate modules in web_interface:

- `complicated_signal_flow.rs` — `Page` (one page's live state: page_id, boss_of_yrs, blocks, page_state, preload_request, preload_response), `PageLoadState` (Loading / PreloadLoaded(usize) / Loaded / Online / Offline / LoadFailed), `BlockRow` (the one block type), `LocalPages`.
- `leptos_components/main_content/` — MainContent, ConnectionIndicator, BackgroundActivityIndicator (placeholder), page_loading_logic.
- `drag_reorder_container/` — DragReorderContainer (wrapper), my_block_content (block UI, human-edited), drag_reorder_black_box (rapier + drag + reorder + vacuum, untouched).

Loading is two effects per page: one matches on page_state and advances one step (preload_pages, load_offline, load_from_server), one watches preload_response and applies blocks. load_from_server is a no-op until the server path exists; preload_pages seeds an empty response because no preloader exists yet.

on_block_input goes through edit_title_for_page into the executor and persists.

## Update 2026-10-20

Order now comes from a fractional position on every block, in both yrs
and the db mirror. Reads sort by (frac_position, block_id). The
fracindex crate generates the strings, wrapped by a new
ZakkeN/smart_frac_index crate so the same logic is available outside
yrs. every_block_in_existence.position became frac_position, Text, and
get_all_blocks_for_page uses get_data_ordered with
ORDER BY frac_position ASC, my_id_as_given_by_yrs ASC.

yrs gains move_block(block_id, block_id_of_above, block_id_of_below),
a frac write rather than a delete+insert, and
insert_new_block_with_preexisting_id for the eventual cross-page path.

love_letter gains MoveBlock { block_id, target_page_id,
new_frac_position }. The builder is move_block, the executor
move_block_requires_three_db_actions. ui_actions::move_block wraps both.
on_move in MainContent calls it for same-page moves only; cross-page
drops log and do nothing yet.

add_block_requires_four_db_actions renamed to
add_block_requires_three_db_actions, because set_block_position_to_id
is gone now that the frac position is known before the row is inserted.

Moving a block will not re-order the page until reload. The drag
component already moved it visually; the block signal is re-read from db
on load.

## Update 2026-10-20

Order now comes from a fractional position on every block, in both yrs
and the db mirror. Reads sort by (frac_position, block_id). The
fracindex crate generates the strings, wrapped by a new
ZakkeN/smart_frac_index crate so the same logic is available outside
yrs. every_block_in_existence.position became frac_position, Text, and
get_all_blocks_for_page uses get_data_ordered with
ORDER BY frac_position ASC, my_id_as_given_by_yrs ASC.

yrs gains move_block(block_id, block_id_of_above, block_id_of_below),
a frac write rather than a delete+insert, and
insert_new_block_with_preexisting_id for the eventual cross-page path.

love_letter gains MoveBlock { block_id, target_page_id,
new_frac_position }. The builder is move_block, the executor
move_block_requires_three_db_actions. ui_actions::move_block wraps both.
on_move in MainContent calls it for same-page moves only; cross-page
drops log and do nothing yet.

add_block_requires_four_db_actions renamed to
add_block_requires_three_db_actions, because set_block_position_to_id
is gone now that the frac position is known before the row is inserted.

Moving a block won't re-order the page until reload. The drag component
already moved it visually; the block signal is re-read from db on load.

dev_actions::insert_three_pages seeds three pages with varied titles and bodies.
