context that's hard to figure out from the code alone.

the yrs crate is compiled for wasm, and wasm has no threads and no SystemTime::now(). That's why BossOfYrs::new takes a unix_time string — it can't ask the clock itself. Ids are built from that string plus an AtomicU64 counter plus a random part, so a boss only needs the time once at construction and every id after that derives from what it already holds.

anti_deadlock.rs has DEBUG_MODE = false. That's a global const, so it turns off deadlock detection on every target, not just wasm. Android would have wanted it. This was a quick fix to stop the wasm build from panicking on thread::spawn. The clean version is a cfg(target_arch = "wasm32") split, false on wasm and true elsewhere.

the browser db is a wasm build artifact committed into the repo. It lives in z_db/db_wrapper, built with wasm-pack, then copied into web_interface/zdb_web_output by update_hard_copied_z_db_web_output.sh. The copy overwrites worker_wrapper.js, which holds a hardcoded connection name. That name has to be changed back to "cqrs" after every copy. If two projects use the same name they share the same OPFS database.

three files have to agree on every db command name by hand: macro_core.rs in db_wrapper defines the method, worker.js maps the command string to it, and db_helper.rs in web_internal_db declares the async fn. Nothing enforces that the three match. If a name in db_helper doesn't exist in worker.js, the worker returns an error at runtime and you get a confusing failure in the browser console.

the wire format has a mismatch. On success the wasm sends Ok::<(), DbError>(()). On error it sends the bare DbError with no Ok/Err wrapper. The browser always decodes as Result<T, DbError>, so successes parse and errors don't. The error shows up as a bincode complaint like "invalid value: integer 2, expected Ok or Err". The integer is the DbError variant index, so integer 2 means IllegalInput. The real error is buried under a deserialization failure.

The error arm of unwrap_or_bail! in web_mascot.rs now wraps the error before serializing, so that one's handled. There's a separate spot that might still have the same shape of problem: finish_output! sends the bare output value instead of an Ok-wrapped one, and the browser decodes as Result<T, DbError>. If a data-returning method like get_data or list_tables ever fails to decode, that's a place to look. This one can't be fixed inside the macro the way the error arm could, because the Ok variant's bytes include T, so the macro would need T to build them.

every dep in this workspace points at git, deliberately. The cost is that a local edit to yrs, client_table_blueprints, love_letter, protocol, or web_internal_db is invisible until pushed. On top of that, Cargo.lock pins the exact commit, so pushing alone doesn't move it — after pushing you have to run cargo update -p <package> from the workspace root or the new commit won't be used.

cargo prints "skipping duplicate package" warnings on every invocation. That's because /home/zakke/ProgStuff/ZakkeN is itself a clone of the Zakarias-Viinikka/ZakkeN repo, so cargo sees the same git URL at a local path and in its checkout cache. It picks one and moves on. Cosmetic, not a problem.

the builder and executor split isn't by purpose. It's by target. The builder prepares all the data and touches nothing outside itself, so it can run anywhere. The executor is the only one that reaches into web_internal_db, which is wasm-only. Same job, different constraints on where each half can run.

client_table_blueprints has consts nowhere — every table name and column name is a hand-typed string, in both the *_columns() functions and the new_*_row helpers. Nothing links the two. If someone renames a column in one place and not the other, the insert silently targets a column that doesn't exist. The Android side had a SafeRowMapper that checked names against the table def before mapping. The Rust side doesn't have an equivalent.
