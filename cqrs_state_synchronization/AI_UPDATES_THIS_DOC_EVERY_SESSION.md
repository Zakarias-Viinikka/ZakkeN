Monday, 5 October 2026
======================

changes since last write:
- yrs: create_bookmark_of_synced_state renamed to create_bookmark. Now on BossOfYrs (as a free fn), YrsActivePages, and YrsBacklinks.
- yrs: generate_diff_snapshot added as a method on YrsActivePages and YrsBacklinks, matching the free fn on BossOfYrs.
- z_db: delete_row_where added. sql_builder generates DELETE FROM ... WHERE ..., and refuses SelectArguments that resolve to nothing (refuses unbounded delete). End to end: sql_builder, black_magic.
- cqrs builder: delete_page in page.rs. Takes page_id, boss, session_id, and the active_pages blob. Returns EverythingForDeletePage (pages_update + uncommitted_diffs).
- cqrs executor: delete_page_requires_three_db_actions still has the old delete-row code. Not yet updated.

new context learned this session:

"delete a page" is not a delete. Nothing gets removed. The pages row stays, the every_block rows stay. The page just gets marked inactive in the active_pages blob, and (once it exists) the every_block rows get an inactive flag too. The old executor code did hard deletes; that's wrong for this project.

active_pages is not its own table. It's a yrs doc, serialized, stored in the tbl_pages.PAGE_STATUS column. Load it via YrsActivePages::new(blob), and write it back by editing the PAGE_STATUS column on the pages row.

every_block_in_existence has no inactive column yet. Columns are: is_title, is_part_of_main_menu_page, content, my_id_as_given_by_yrs, id_of_page_i_belong_to, position. Adding "inactive" means a new column plus a migration.

three columns want a yrs blob written to them and each wants a different source. pages.BLOBBED_PAGE is the page's own yrs doc snapshot. pages.PAGE_STATUS is the active_pages blob. uncommitted_diffs.snapshot_of_edit is the diff since a bookmark. They look alike but aren't.

the delete_page flow needs both blobs. The active_pages blob (updated, page marked inactive) goes into pages.PAGE_STATUS. The diff from active_pages goes into uncommitted_diffs.snapshot_of_edit. No BLOBBED_PAGE change.

Saturday, 3 October 2026
=========================

changes since last write:
- client_table_blueprints: get_table_name_*() fns replaced with TABLE_NAME consts in tbl_pages, tbl_backlinks, tbl_every_block_in_existence, tbl_uncommitted_diffs.
- z_db: delete_all_rows and count_rows added end to end. payload.rs, sql_builder, black_magic_extension, macro_core, worker.js, both db_helper.rs files.
- cqrs web_interface: call sites updated for TABLE_NAME (init.rs, db_helpers_for_web_client.rs, ui_actions.rs). do_delete_all_rows now async. delete_everything and insert_three_pages use spawn_local.
- checkbox callbacks: type is now Box<dyn Fn() -> Pin<Box<dyn Future<Output = ()>>>>>. run_all_seeds is sync and spawns each callback's future itself.
- web_interface/Cargo.toml: most deps now point at path = "..." instead of git. Original git lines commented above each.

new context learned this session:

a db command touches four files by hand. macro_core.rs in db_wrapper defines the method. worker.js maps the command string to it. And two db_helper.rs files declare the async fn: web_internal_db/src/db_helper.rs (this is what compiles into cqrs via the git dep) and db_wrapper/src/web_output/db_helper.rs (this is what the copy script grabs). Nothing enforces that the four agree. When a new command is added, all four have to be updated by hand. wasm-pack does not regenerate db_helper.rs or worker.js — only db_wrapper.js and db_wrapper_bg.wasm.

tell_worker_to_do now checks for the error path. The worker posts ['error', err.toString()] when a JS exception escapes a handler (wasm panic, thrown error). Slot 1 in that case is a string, not a Uint8Array. tell_worker_to_do checks slot 0 for the string "error" via response_is_error, and if so returns extract_error. That surfaces the worker's message instead of the misleading "response[1] was not a Uint8Array".

nightly changes hash daily. every workspace with a target dir built on the old hash fails every dep with "found crate X compiled by an incompatible version of rustc". Affects: z_db, client_table_blueprints, yrs, text_diff, love_letter, and the cqrs workspace root (which has its own target/). cargo clean in each, then rebuild.

SQLite allows only one write transaction at a time. BEGIN IMMEDIATE inside an already-open transaction fails with "cannot start a transaction within a transaction". insert_three_pages calls create_new_page three times in parallel and each one opens its own BEGIN IMMEDIATE, so two of three fail. create_new_page logs the error but keeps going, still pushing into local_pages, so the UI shows three pages and the DB has one. Either serialize the calls or wrap all three inserts in one outer transaction.

checkbox callbacks are async. run_all_seeds is a sync fn and spawns each callback's future itself. It can't be async, because CheckboxToCallbackMap holds non-Send closures that can't survive an await boundary in a spawn_local'd future, and RwSignal::new refuses non-Send inner types, so it can't be put in a signal either. When a non-'static, non-Send value needs to live inside an Effect, wrap it in StoredValue, not Rc and not a signal.

still true from before (no change):

the yrs crate is compiled for wasm, and wasm has no threads and no SystemTime::now(). That's why BossOfYrs::new takes a unix_time string — it can't ask the clock itself. Ids are built from that string plus an AtomicU64 counter plus a random part, so a boss only needs the time once at construction and every id after that derives from what it already holds.

anti_deadlock.rs has DEBUG_MODE = false. That's a global const, so it turns off deadlock detection on every target, not just wasm. Android would have wanted it. This was a quick fix to stop the wasm build from panicking on thread::spawn. The clean version is a cfg(target_arch = "wasm32") split, false on wasm and true elsewhere.

the browser db is a wasm build artifact committed into the repo. It lives in z_db/db_wrapper, built with wasm-pack, then copied into web_interface/zdb_web_output by update_hard_copied_z_db_web_output.sh. The copy overwrites worker_wrapper.js, which holds a hardcoded connection name. That name has to be changed back to "cqrs" after every copy. If two projects use the same name they share the same OPFS database.

the wire format is split by return type. Commands that return nothing send Ok::<(), DbError>(()) on success and Err::<(), DbError>(e) on error, and db_helper decodes them as Result<(), DbError>, so both cases parse. Commands that return data (get_data, list_tables, check_table and so on) send the bare output struct on success through finish_output!, and db_helper decodes them as the bare struct. On error, unwrap_or_bail! still sends Err::<(), DbError>(e), which the bare-struct decode can't read, so a failing data command shows up in the browser as a bincode decode error (CureFail) instead of the real DbError.

cargo prints "skipping duplicate package" warnings on every invocation. That's because /home/zakke/ProgStuff/ZakkeN is itself a clone of the Zakarias-Viinikka/ZakkeN repo, so cargo sees the same git URL at a local path and in its checkout cache. It picks one and moves on. Cosmetic, not a problem.

the builder and executor split isn't by purpose. It's by target. The builder prepares all the data and touches nothing outside itself, so it can run anywhere. The executor is the only one that reaches into web_internal_db, which is wasm-only. Same job, different constraints on where each half can run.
