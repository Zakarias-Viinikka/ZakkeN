# nov24plan

Goal: LinkToExistingPageComp shows existing page titles that match what the user types, read from the db with full-text search (sqlite fts5).

Rules: one chunk at a time, wait for "next". Paths are absolute. Re-ask for a file if it may have changed. Never rename Zakke's things, never refactor his code. No code until he says go. Errors are never unwrapped or swallowed: return CqrsErr to the caller (the zutil example uses unwrap everywhere, do not copy that).

## Where the work is

The component is in /home/zakke/ProgStuff/ZakkeN/cqrs_state_synchronization/web_interface/src/leptos_components/special_menu_test.rs. create_matching_titles was copied from create_keyword_matcher and still reads KEYWORD_PICKER. Two bugs in that file: LinkToExistingPageComp stores the matcher in page_title_matches but its view reads keyword_matches, which doesn't exist there; and create_matching_titles returns &str items while its signature says String.

## Decisions made

Titles are searched with fts5, not by reading all titles and filtering in Rust.

The full-text mirror table for every_block_in_existence does not exist yet. In z_db the mirror is named fts5_every_block_in_existence, indexes the columns passed to create, and has triggers per column for insert, update and delete. It indexes every block, not just titles. A search returns only rowid and the indexed column, so title-only matching is two reads: search, then read those rows from every_block_in_existence where is_title is "true".

The create has no IF NOT EXISTS (table or triggers), so it must run exactly once. Rows that already exist need one rebuild_fts5_index before they are searchable.

Web side: web_internal_db::db_helper has create_fts5_table(CreateFts5TableIn { source_table_name, columns }), rebuild_fts5_index(RebuildFts5In) and search_fts5(SearchFts5In { table_name, text_to_lookup }). Zakke says db_helper is up to date, so list_tables, insert_data and edit_col_in_row are treated as available (not verified by Claude).

Prefix matching: search_fts5_sql_builder in /home/zakke/ProgStuff/z_db/sql_builder/src/lib.rs quotes each word so only whole words match. Zakke approved adding a * after each quoted word. The code comment says the fts5 builders are untested.

Migration: it is a migration, not a start-over. Chunk 1 concluded this. The web app has no migration today (confirmed by Zakke, he made the project). Zakke wants proper migrations added to the schema crate and to the web app. Starting the db over is acceptable to him if it is needed to get there, but the plan is to keep the existing db and treat it as version 0.

The migration files go in a new module web_migrations inside /home/zakke/ProgStuff/ZakkeN/client_table_blueprints (src/web_migrations). Android does not matter for this, so that crate may depend on web_internal_db. Its Cargo.toml does not have it today, so the migration chunk adds it. It can NOT depend on error_stuff, because error_stuff depends on client_table_blueprints and cargo refuses cycles. The migration code returns DbError from protocol, and web_interface converts it to CqrsErr with ? (CqrsErr has From<DbError>, decided). The live db has every column in today's client_table_blueprints schema, including page_is_disabled (confirmed by Zakke). version0 copies today's columns.

## What was learned about migration

Reference example: /home/zakke/ProgStuff/egui/zutil/db/src, files init.rs, migration/helpers.rs, migration/schema_versions.rs, migration/update_from_old_schema.rs, migration/schemas/current.rs, migration/schemas/version0.rs to version8.rs, tables/mod.rs. Older, simpler template: /home/zakke/ProgStuff/z_db/migration_template (src, and simple_real_usage_example with a tests/migration.rs). z_db/migration_helper is an empty hello world, ignore it.

How the zutil pattern works. A one-row table schema_version holds the version number. On startup, if the main table (texts) is missing the db is fresh: create the newest shape directly and write the newest version. Otherwise read the stored number (none stored counts as version 0) and run the version steps one by one up to the newest, then write the new number. SchemaVersion is an enum with to_int and from_int and a CURRENT_VERSION const. update_once matches on the version and calls the next versionN::update_from_old_version, which is usually one db call. Each versionN.rs also has entire_table(), the full table list for that version. current.rs points at the newest. Existing versions are frozen, never edited, so version files hold copied column lists and do not call the tbl_ functions that may change later. The test creates version0, runs the chain, then drops everything, recreates from the newest shape and compares check_table output.

Zutil runs setup_fts5 outside the versions on every startup and skips itself if any table starting with fts5_ exists. The table list (ADbTable) has no place for a full-text table.

What this means for the web app:

1. The version steps would be async and call web_internal_db::db_helper instead of the sync LiveForever.
2. Current /home/zakke/ProgStuff/ZakkeN/cqrs_state_synchronization/web_interface/src/db/init.rs creates five tables on every launch with plain create_table and swallows the "Table already exists" error: tbl_client_love_letter, tbl_every_block_in_existence, tbl_key_value_storage, tbl_pages, tbl_seen_server_letters. It never calls get_foreign_def_every_block_in_existence, so the existing db has no foreign keys. version0 must describe exactly these five tables as they exist now, without foreign keys, or an existing db and version0 disagree. tbl_backlinks, tbl_incoming_love_letters and tbl_logs exist in client_table_blueprints but init.rs does not create them.
3. The existing db has no version table, so it counts as version 0.
4. A fresh db needs the fts table created right after the tables, with no rebuild since it is empty. An old db gets it from version1: create_fts5_table, then rebuild_fts5_index.
5. How "fresh" is detected on the web side (zutil checks whether texts exists) is not decided.
6. What happens to the swallowed "Table already exists" handling in create_all_tables once migration exists is not decided.

## Open questions

Is get_all_menu_pages the right set of pages? It only returns enabled pages that are on the main menu page. Not decided.

What happens when a title is clicked? No link action exists yet. Not part of this plan until decided.

Does Zakke want a migration test like the template's (migrated db equals fresh db)? Not asked yet. Zakke's workflow says to run migration tests first when debugging db issues.

"Mint" (Zakke said his db stuff is supposed to be mint): ask him what it meant only if it matters.

## Chunk 1: understand migration (done)

Answer: it is a migration. See "What was learned about migration". Nothing was edited.

## Chunk 2: prefix matching in the search (EDITED, build and tests NOT confirmed by Zakke)

Change: search_fts5_sql_builder puts a * after each quoted word, so "ho" finds "home". Approved by Zakke.

Files to see:
/home/zakke/ProgStuff/z_db/sql_builder/src/lib.rs (already seen once, ask again before editing)
Any z_db test that checks the search sql text (not found yet, ask for a grep)

After the edit, build_web_and_android.sh has to run so zdb_web_output gets the new worker.

## Chunk 3: the migration system in client_table_blueprints (EDITED, cargo check NOT confirmed by Zakke)

Change: add the web_migrations module with schema_versions, update_from_old_schema, schemas (current, version0, version1) and its helpers, add the dependencies to Cargo.toml, and wire it into web_interface startup so it replaces the create-every-launch in init.rs. Version tracking uses the schema_version table. Port the zutil pattern with the async db_helper calls and CqrsErr errors. Start with version0 only, so the existing db is recognized and nothing else changes. The fts table is not part of this chunk.

Zakke said to go slowly. Talk the design through in plain terms before any code, and settle the open points in "What this means for the web app" (items 5 and 6) with him first.

Files to see:
/home/zakke/ProgStuff/ZakkeN/client_table_blueprints/Cargo.toml
/home/zakke/ProgStuff/ZakkeN/client_table_blueprints/src/lib.rs
/home/zakke/ProgStuff/ZakkeN/client_table_blueprints/src/tbl_client_love_letter.rs
/home/zakke/ProgStuff/ZakkeN/client_table_blueprints/src/tbl_every_block_in_existence.rs
/home/zakke/ProgStuff/ZakkeN/client_table_blueprints/src/tbl_key_value_storage.rs
/home/zakke/ProgStuff/ZakkeN/client_table_blueprints/src/tbl_pages.rs
/home/zakke/ProgStuff/ZakkeN/client_table_blueprints/src/tbl_seen_server_letters.rs
/home/zakke/ProgStuff/ZakkeN/cqrs_state_synchronization/web_interface/src/db/init.rs
/home/zakke/ProgStuff/ZakkeN/cqrs_state_synchronization/web_interface/zdb_web_output/db_helper.rs
The zutil example files listed above, as needed

## Chunk 4: create the full-text table

Change: version1 creates fts5_every_block_in_existence on the content column and rebuilds the index, so existing rows are searchable. The fresh-db path creates the same table right after the tables. Bump CURRENT_VERSION to version 1.

Files to see:
Whatever chunk 3 added
/home/zakke/ProgStuff/ZakkeN/client_table_blueprints/src/tbl_every_block_in_existence.rs
/home/zakke/ProgStuff/ZakkeN/cqrs_state_synchronization/web_interface/zdb_web_output/db_helper.rs
/home/zakke/ProgStuff/z_db/db/src/fts5_magic.rs

## Chunk 5: search read for titles

Change: a function in reads_wrapper that runs the full-text search, then reads the matching rows from every_block_in_existence where is_title is true, and returns LocalPages. The search returns only rowid and content, so this is two reads.

Files to see:
/home/zakke/ProgStuff/ZakkeN/cqrs_state_synchronization/web_interface/src/db/reads_wrapper/menu.rs
/home/zakke/ProgStuff/ZakkeN/cqrs_state_synchronization/web_interface/src/db/reads_wrapper/destruct.rs
/home/zakke/ProgStuff/ZakkeN/cqrs_state_synchronization/web_interface/src/db/reads_wrapper/mod.rs
/home/zakke/ProgStuff/ZakkeN/cqrs_state_synchronization/web_interface/src/complex_state/rows.rs
/home/zakke/ProgStuff/ZakkeN/cqrs_state_synchronization/web_interface/zdb_web_output/db_helper.rs
/home/zakke/ProgStuff/ZakkeN/client_table_blueprints/src/tbl_every_block_in_existence.rs
/home/zakke/ProgStuff/ZakkeN/cqrs_state_synchronization/web_interface/src/db/test_ops_for_db.rs (read_titles there is a read-all-titles example, reads is_title and content)

## Chunk 6: create_matching_titles and the component

Change: create_matching_titles calls the chunk 5 read as the user types, with the same stale-result guard menu.rs uses. LinkToExistingPageComp reads the right signal (it currently reads keyword_matches, which doesn't exist there) and shows titles keyed by page_id.

Files to see:
/home/zakke/ProgStuff/ZakkeN/cqrs_state_synchronization/web_interface/src/leptos_components/special_menu_test.rs
/home/zakke/ProgStuff/ZakkeN/cqrs_state_synchronization/web_interface/src/leptos_components/menu.rs
Whatever chunk 5 added

## Reminders for Claude (read these first after a reset)

Chunk 2 edited /home/zakke/ProgStuff/z_db/sql_builder/src/lib.rs (a * after each quoted word) and the tests in /home/zakke/ProgStuff/z_db/sql_builder/tests/fts5_test.rs. Zakke has not run cargo test or the build yet because his target folder is cleared and the build is slow. Treat it as correct and keep working. If a search later returns nothing or errors, suspect this edit first.

Still to run, in this order: cargo test in /home/zakke/ProgStuff/z_db/sql_builder, then build_web_and_android.sh so zdb_web_output gets the new worker. Ask Zakke for the script's absolute path, it is not known yet.

Remind Zakke to run those two when the work first needs the search to actually run, which is chunk 4 (the fts table creation) at the latest. Also remind him any time a later chunk edits anything in z_db (sql_builder, db, db_wrapper, web_internal_db), since the worker has to be rebuilt after each of those.

Chunk 3 wrote /home/zakke/ProgStuff/ZakkeN/client_table_blueprints/src/web_migrations (mod.rs, schema_versions.rs, update_from_old_schema.rs, helpers.rs, db_actions_for_schema_version_table.rs, main_for_bringing_the_db_to_the_newest_schema_version.rs, schemas/mod.rs, schemas/current.rs, schemas/version0.rs), added web_internal_db to client_table_blueprints Cargo.toml and pub mod web_migrations to its lib.rs. web_interface init.rs init_all now calls bring_the_db_to_the_newest_schema_version and returns early on error. Only Version0 exists, so a live db just gets a schema_version table with 0. create_all_tables and create_each_table_one_by_one were deleted from init.rs (rg showed nothing else called them), along with the imports only they used. No migration test exists yet (the open question about one is still open). Chunk 4 adds Version1 (fts table), so it must edit schema_versions.rs, update_from_old_schema.rs, current.rs and add schemas/version1.rs.
