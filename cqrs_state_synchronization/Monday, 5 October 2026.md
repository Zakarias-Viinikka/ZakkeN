# Monday, 5 October 2026

Add an "inactive" marker to the `every_block_in_existence` table.

Add the column in `tbl_every_block_in_existence.rs`: `new_table_every_block_in_existence`, plus a schema const (`INACTIVE`) and update `new_every_block_in_existence_row`.

Write a migration so existing rows get the column. Default to active.

Then in the delete-page flow:

`delete_page` in `page.rs` — currently returns `EverythingForDeletePage` with a `PagesUpdateCtx`. That part is fine, but the pages update should target `PAGE_STATUS` (active_pages blob), not `BLOBBED_PAGE`.

`delete_page_requires_three_db_actions` in `page_executor.rs` — remove the two `delete_row_where` calls. Replace with: update pages row's `page_status` (new active_pages blob), mark every block under that page inactive, insert uncommitted diff. So it's three updates/inserts, but none of them delete anything. Rename the fn to match.

Add a helper in the executor like `mark_all_blocks_for_page_inactive(page_id)` that does `edit_col_in_row_where` with `id_of_page_i_belong_to = page_id`, setting `inactive = true`.

Decide how reads handle inactive blocks. `get_data`, `get_entire_page`, `get_title_and_id_of_all_menu_pages` — do they filter out inactive rows, or is that the caller's job? This affects every read path.

Open question: blocks are only marked inactive when their page is. Does an individual block ever get marked inactive on its own? If not, is a per-block flag actually needed, or is "page inactive" enough? Worth confirming before writing the migration.
