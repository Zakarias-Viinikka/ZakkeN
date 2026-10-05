context that's hard to figure out from the code alone.

the yrs crate is compiled for wasm, and wasm has no threads and no SystemTime::now(). That's why BossOfYrs::new takes a unix_time string — it can't ask the clock itself. Ids are built from that string plus an AtomicU64 counter plus a random part, so a boss only needs the time once at construction and every id after that derives from what it already holds.

anti_deadlock.rs has DEBUG_MODE = false. That's a global const, so it turns off deadlock detection on every target, not just wasm. Android would have wanted it. This was a quick fix to stop the wasm build from panicking on thread::spawn. The clean version is a cfg(target_arch = "wasm32") split, false on wasm and true elsewhere.

the browser db is a wasm build artifact committed into the repo. It lives in z_db/db_wrapper, built with wasm-pack, then copied into web_interface/zdb_web_output by update_hard_copied_z_db_web_output.sh. The copy overwrites worker_wrapper.js, which holds a hardcoded connection name. That name has to be changed back to "cqrs" after every copy. If two projects use the same name they share the same OPFS database.

three files have to agree on every db command name by hand: macro_core.rs in db_wrapper defines the method, worker.js maps the command string to it, and db_helper.rs in web_internal_db declares the async fn. Nothing enforces that the three match. If a name in db_helper doesn't exist in worker.js, the worker returns an error at runtime and you get a confusing failure in the browser console.

the wire format is split by return type. Commands that return nothing send Ok::<(), DbError>(()) on success and Err::<(), DbError>(e) on error, and db_helper decodes them as Result<(), DbError>, so both cases parse. Commands that return data (get_data, list_tables, check_table and so on) send the bare output struct on success through finish_output!, and db_helper decodes them as the bare struct. On error, unwrap_or_bail! still sends Err::<(), DbError>(e), which the bare-struct decode can't read, so a failing data command shows up in the browser as a bincode decode error (CureFail) instead of the real DbError. If a get_data call fails with a strange decode message, the real failure is on the sql side and the message is hiding it.

every dep in this workspace points at git, deliberately. The cost is that a local edit to yrs, client_table_blueprints, love_letter, protocol, or web_internal_db is invisible until pushed. On top of that, Cargo.lock pins the exact commit, so pushing alone doesn't move it — after pushing you have to run cargo update -p <package> from the workspace root or the new commit won't be used.

cargo prints "skipping duplicate package" warnings on every invocation. That's because /home/zakke/ProgStuff/ZakkeN is itself a clone of the Zakarias-Viinikka/ZakkeN repo, so cargo sees the same git URL at a local path and in its checkout cache. It picks one and moves on. Cosmetic, not a problem.

the builder and executor split isn't by purpose. It's by target. The builder prepares all the data and touches nothing outside itself, so it can run anywhere. The executor is the only one that reaches into web_internal_db, which is wasm-only. Same job, different constraints on where each half can run.

client_table_blueprints has consts nowhere — every table name and column name is a hand-typed string, in both the *_columns() functions and the new_*_row helpers. Nothing links the two. If someone renames a column in one place and not the other, the insert silently targets a column that doesn't exist. The Android side had a SafeRowMapper that checked names against the table def before mapping. The Rust side doesn't have an equivalent.

the yrs doc doesn't know which block is the title. create_page inserts two blocks with empty text and empty meta, the second after the first, so the title is only the block at index 0 of the yrs array plus is_title = "true" in every_block_in_existence. Booleans in these tables are stored as Text "true"/"false", so queries compare against the string.

position in every_block_in_existence mirrors the yrs array order, it doesn't define it. create_page hardcodes 0.0 for the title and 1.0 for the first block. The android app computes position with a gap scheme (average of neighbours, max+1 at the end, min-1 at the top). The first uncommitted_diffs row for a new page holds the full doc snapshot, the same bytes as pages.blobbed_page, and its target_id is the page id.

page_status in the pages row is built inside client_table_blueprints::new_page_row, which the executor calls. The builder never touches YrsActivePages, even though CURRENT_STATE.md says it snapshots one.

create_boss in the builder passes an empty FAKE_TIME and user id "1", so ids look like "-0-<random>-1". Uniqueness comes from the counter plus the random part. The counter is a process-wide static that restarts at 0 on every page load, so the random part is what stops ids colliding across reloads.

the android code that CONTEXT.md points to is a reference for the order of operations and the position logic, not for api shapes. On the 24th the rust helpers changed: new_row_helper functions return Vec<ColumnValue> and pick the column names themselves, and new_every_block_in_existence_row takes is_part_of_main_menu_page. The android build was refreshed but the kotlin callers still use the old signatures, so they don't match. AI_READ_THIS.md on the android side is behind too.

new_log_row and new_incoming_love_letter_row in client_table_blueprints call SystemTime::now(), which panics in browser wasm. They can't be called from web_interface as written.

web_interface/zdb_web_output/db_helper.rs is not compiled. It's only there because the copy script takes the whole web_output folder. The db_helper that actually builds is web_internal_db::db_helper from the git dep. The executor crate's Cargo.toml also uses absolute paths for error_stuff and the builder, while the rest of the workspace uses relative ones.

menu.rs reads the columns "title" and "yrs_id" from every_block_in_existence. The real column names are content and my_id_as_given_by_yrs, so that read fails until they're changed, and because it's a data command the failure surfaces as the decode error described above.
