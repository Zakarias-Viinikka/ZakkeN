# nov24plan

Goal: LinkToExistingPageComp shows existing page titles that match what the user types, read from the db with full-text search.

Rules: one chunk at a time, wait for "next". Paths are absolute. Re-ask for a file if it may have changed.

## Open questions

Is get_all_menu_pages the right set of pages? It only returns enabled pages that are on the main menu page. Not decided.

What happens when a title is clicked? No link action exists yet. Not part of this plan until decided.

Migration or start the db over? Decided in chunk 1.

## Chunk 1: understand migration (read only, no edits)

Question to answer: does a migration exist for this app, and can it create a full-text table? Is it a migration, or a start-over?

Files to see:
/home/zakke/ProgStuff/z_db/migration_template (all of it, including simple_real_usage_example and src)
/home/zakke/ProgStuff/z_db/migration_helper
/home/zakke/ProgStuff/z_db/db/src/migration.rs (already seen)
/home/zakke/ProgStuff/ZakkeN/cqrs_state_synchronization/web_interface/src/db/init.rs (already seen)
/home/zakke/ProgStuff/ZakkeN/cqrs_state_synchronization/web_interface/src/db/test_ops_for_db.rs

## Chunk 2: prefix matching in the search

Change: search_fts5_sql_builder puts a * after each quoted word, so "ho" finds "home". Approved by Zakke.

Files to see:
/home/zakke/ProgStuff/z_db/sql_builder/src/lib.rs (already seen)
Any z_db test that checks the search sql text (not found yet, ask for a grep)

After the edit, build_web_and_android.sh has to run so zdb_web_output gets the new worker.

## Chunk 3: create the full-text table

Change: create fts5_every_block_in_existence on the content column, once, and rebuild the index so existing rows are searchable. How it runs depends on chunk 1.

Files to see:
/home/zakke/ProgStuff/ZakkeN/cqrs_state_synchronization/web_interface/src/db/init.rs
/home/zakke/ProgStuff/ZakkeN/client_table_blueprints/src/tbl_every_block_in_existence.rs
/home/zakke/ProgStuff/ZakkeN/cqrs_state_synchronization/web_interface/zdb_web_output/db_helper.rs
/home/zakke/ProgStuff/z_db/db/src/fts5_magic.rs

## Chunk 4: search read for titles

Change: a function in reads_wrapper that runs the full-text search, then reads the matching rows from every_block_in_existence where is_title is true, and returns LocalPages. The search returns only rowid and content, so this is two reads.

Files to see:
/home/zakke/ProgStuff/ZakkeN/cqrs_state_synchronization/web_interface/src/db/reads_wrapper/menu.rs
/home/zakke/ProgStuff/ZakkeN/cqrs_state_synchronization/web_interface/src/db/reads_wrapper/destruct.rs
/home/zakke/ProgStuff/ZakkeN/cqrs_state_synchronization/web_interface/src/db/reads_wrapper/mod.rs
/home/zakke/ProgStuff/ZakkeN/cqrs_state_synchronization/web_interface/src/complex_state/rows.rs
/home/zakke/ProgStuff/ZakkeN/cqrs_state_synchronization/web_interface/zdb_web_output/db_helper.rs
/home/zakke/ProgStuff/ZakkeN/client_table_blueprints/src/tbl_every_block_in_existence.rs

## Chunk 5: create_matching_titles and the component

Change: create_matching_titles calls the chunk 4 read as the user types, with the same stale-result guard menu.rs uses. LinkToExistingPageComp reads the right signal (it currently reads keyword_matches, which doesn't exist there) and shows titles keyed by page_id.

Files to see:
/home/zakke/ProgStuff/ZakkeN/cqrs_state_synchronization/web_interface/src/leptos_components/special_menu_test.rs
/home/zakke/ProgStuff/ZakkeN/cqrs_state_synchronization/web_interface/src/leptos_components/menu.rs
Whatever chunk 4 added
