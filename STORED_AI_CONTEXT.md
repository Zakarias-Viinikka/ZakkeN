# ZakkeN — AI stored context

Written by AI for AI. Meant to be the first file a fresh AI reads when
starting work on this project. Update a section when its content
changes and bump that section's date. If code and this file disagree,
the code wins and this file should be corrected.

## What this is — 2026-10-13

ZakkeN is an offline-first Notion-style notes app for personal use, on
Android and the web. The goal is to store, edit, and find old notes
easily, sync seamlessly, need almost no config, and have a better UX
than what exists.

The domain is pages and blocks. A page is a list of blocks. A block is
text plus metadata. Every page's state lives in a yrs CRDT doc; the
sqlite mirror is queryable but not authoritative. Edits are described
as "love letters" — a serialized intent the client sends to the server
so both sides agree on what changed. The client stores its own copy in
uncommitted_diffs while the server, when it exists, tracks which
registered clients have received which letters.

The server side is not built out. cqrs_state_synchronization is where
the client-side state machinery is being developed.

## Crate map — 2026-10-13

yrs (ZakkeN/yrs) — CRDT wrappers. BossOfYrs owns one page's doc,
YrsActivePages owns the "which pages are disabled" doc, YrsBacklinks
owns backlinks. Wasm has no threads and no clock, so BossOfYrs::new
takes a unix_time string; IDs are built from that string, an atomic
counter, and a random part. anti_deadlock.rs wraps every public method
with a timeout; DEBUG_MODE is currently false on all targets (wasm
would panic on thread::spawn), which is a stopgap.

client_table_blueprints (ZakkeN/client_table_blueprints) — the schema.
tbl_pages, tbl_every_block_in_existence, tbl_uncommitted_diffs,
tbl_incoming_love_letters, tbl_backlinks, tbl_key_value_storage,
tbl_logs. Each tbl_x.rs declares the columns as ColumnDefs, exports
SchemaColumn consts, and has a new_x_row() helper returning the
Vec<ColumnValue> for an insert. Table names are TABLE_NAME consts.

love_letter (ZakkeN/love_letter) — the contract. LoveLetterSketch is
the intent (EditBlock, CreateNewBlock, RemoveBlock, DisablePage,
CreateNewPage). LoveLetter is the same plus a snapshot_of_edit blob.
build_a_love_letter combines sketch + snapshot into a letter.
bincode serialization. No dependency on yrs — TextEdit and
PositionToInsert live in yrs and are re-exported for use here.

text_diff (ZakkeN/text_diff) — character-level diff and squashing.
diff_logic::get_diff(old, new) -> DiffResult. diff_batching has the
combine logic. Clients batch edits before persisting; the batching
library was deliberately made dumb (caller owns the list of diff
results and calls combine when ready) because sharing batch state
across the FFI boundary was too much complexity.

cqrs_state_synchronization (ZakkeN/cqrs_state_synchronization) —
where the current work is. Layout:

- data_builder_for_operations_that_need_to_be_correct — the builder.
  Pure, runs anywhere. create_page, edit_block, disable_page each
  return a ctx struct with everything the executor needs.
- executor_of_what_the_builder_built_because_the_builder_shouldnt_touch_the_db —
  the executor. Reaches into web_internal_db, so wasm-only.
  insert_page_requires_three_db_inserts, edit_block_requires_three_db_inserts,
  disabled_page_requires_three_db_actions. Each wraps in a transaction,
  or joins an outer one if already open (start_transaction_if_not_in_one!).
- error_stuff — shared CqrsErr for the workspace.
- web_interface — the leptos csr web client. Routes: / and /Menu (menu),
  /DbGui (the orm, from web_internal_db). LocalPages is the menu's
  in-memory page list, read from db on load. Each row in the menu has
  an edit-title button and a disable-page button; each button owns
  its own builder -> executor -> db_helper chain.

The builder/executor split isn't by purpose — it's by target. The
builder prepares data and touches nothing, so it can run anywhere.
The executor is the only piece that reaches into web_internal_db,
which is wasm-only.

Docs in cqrs: CONTEXT.md (what to read), CURRENT_PLAN.md (the testing
plan, deferred), CURRENT_STATE.md (where things are now),
GENERAL_TODO.md, AI_UPDATES_THIS_DOC_EVERY_SESSION.md (reverse-
chronological session log — the best place to see recent work).

zakken_server (ZakkeN/zakken_server) — the axum websocket server for
the actual project. Currently has /ws, LiveForever behind a Mutex,
allowed_requests::do_request matching on Request, and reply::craft_response.
Not much there yet. The rust_practice/server_template_* family is the
reference lineage; server_template_3 is the intended future shape of
this server (see "server_template_3" below).

client_table_blueprints/android_output, yrs/android_output_folder,
love_letter/android_output_folder, text_diff/android_output_folder —
build artifacts the Android app downloads via update.sh. Regenerated
by each crate's build script; committed to the repo.

## External: z_db — 2026-10-13

~/ProgStuff/z_db — separate repo, git dependency in the crates that
need it. Provides sqlite access, a protocol crate (payloads, Row/Col,
errors, ColumnDef, schema_helper), and per-target wrappers ("mascots").

Crates:
- db — rusqlite wrapper. black_magic.rs does the actual sqlite calls,
  black_magic_extension.rs is "extension" crud (count, etc), fts5_magic.rs
  is search, migration.rs is migration, transactions.rs is transaction
  control.
- protocol — shared by everyone. payload has the in/out structs;
  row_col has Col/Row; new_table has ColumnDef and the column helpers
  (id_column, not_null_col, not_null_unique_col, ForeignKeyDef);
  schema_helper has SchemaColumn<T> and TypeOfCol, plus
  destruct_db_col / try_destruct_db_col. The three-field error
  convention (err_msg, method, filepath) lives here.
- sql_builder — takes parameters, produces SQL.
- db_wrapper — the facade. macro_core.rs holds the methods once;
  per-target mascots (plain, serializer, android, web, testing) are
  generated from it. A new command has to be added to macro_core.rs,
  worker.js, and both db_helper.rs files (web_internal_db and
  db_wrapper/src/web_output) by hand. Nothing enforces agreement.
  add_new_method_to_wrapper is a GUI for doing it.
- web_internal_db — browser-facing. Runs inside a worker, talks to
  web_mascot, and exposes db_helper::{create_table, get_data,
  insert_data, get_single_col, count_all_rows, delete_all_rows,
  edit_col_in_row_where, edit_col_in_row, are_we_in_middle_of_transaction,
  begin_all_or_nothing, everything_went_perfectly, regret_everything,
  ...}. Also has db_gui (the ORM page). Adding a command means
  updating macro_core.rs and both web db_helper.rs files.
- migration_helper / migration_template — how migrations are done.
  See migration_template/simple_real_usage_example.
- msg_build_n_debuild_helper — helpers for building/unbuilding messages
  across the wire.

ZakkeN apps import z_db's protocol and web_internal_db; the app never
touches rusqlite directly.

## server_template_3 — 2026-10-13

~/ProgStuff/rust_practice/server_template_3 — the intended future of
zakken_server. server_template_2 is the current state: axum websocket,
sqlite next to the crate, tables from server_db_schema, all DB access
through a Mutex<LiveForever>. server_template_3 separates "receiving a
request" from "doing it" so they can run concurrently, using the
actor-queue shape from rust_practice/actor_audition: a priority list
and a normal list, oldest of the highest priority first, a wake channel
so the worker doesn't spin when idle, and an add_to_queue method that
pushes and wakes in one call. Refactor comes after.

server_template_1 has the client side of the same pattern: server_talker
(browser websocket holder with tx/rx/state channels) and
router_for_responses_coming_to_client (matches message_ids to waiting
callers). server_template_2 carries those over.

## browser_ui_testing/drag_reorder — 2026-10-13

Experiments for a block-based editor's drag-reorder UI. The most
relevant crate is rapier_tests_combined — it merges the two-page
drag-reorder UI from rapier_test_2 with the cascade motion from
rapier_collision_test, both siblings. Rapier is a query engine only
(the world is never stepped); positions live in a leptos signal and
motion is DOM-driven. Blocks cascade downward from the drop point with
a velocity ramp, targets computed as bottom_of_block_above + GAP.
Vacuum on the left edge stores a block; visual confirmation draws four
guide lines per changed rect. The crate is a UI prototype, not part of
the sync path.

The current work on this crate (in progress, see STORED_AI_CONTEXT.md
inside it) is to isolate the drag UI behind a component so it can drop
into a real project. The component takes pages, blocks (block_id,
page_id, content), and an on_move callback
(moved, from_page, to_page, above, below). Blocks become editable
text; drag moves to a handle. Vacuum gains a ghost block occupying
the stored block's slot. The db_test sibling has the sqlite schema
(migrations, schema_version, word table) that the merged crate will
use for a minimal pages + every_block demo.

## Android and web targets — 2026-10-13

android_app — the Android build of the real app. Kotlin + Compose
UI on top of the same Rust crates via UniFFI. It's the second target
every crate has to keep building for. It is very stale: its docs
(SHARED_MEMORY.md, AI_READ_THIS.md, UX_INTENT.md, AI_LAB_RULE.md)
are out of date, and lots of its code was AI-driven. The docs still
describe the intent (block-based editing, mobile-first, buffered
edits, Lab for UX experiments) and are worth reading for that, but
don't trust them as a picture of current state.

web_client — superseded by cqrs_state_synchronization/web_interface.
The current web client is the cqrs one.

## Dead or parked — 2026-10-13

These exist but aren't part of the working set. Worth knowing about
because they sometimes have the only example of something.

csr_testing — drag-reorder with Sortable.js from a leptos csr app.
Superseded by the rapier drag_reorder experiments.

wasm/* — many small experimental crates (wasm_rusqlite_2, serde_sql_template,
yrs_integration, leptos_integration_1/2/3, and more). Everything here
is historical. Some of it (serde_sql_template, leptos_integration_3)
documents how the current z_db web stuff got its shape; if the user
asks "where was I doing X", the answer is often in here.

old_stuff — text_diff_old and an old zakken_server. Ignore.

messing_around — UI style explorations (themes). Not dead; the user
looks at it for design ideas. Not part of the build.

fts5 — an fts5 search demo. Parked but not deleted; may become the
search story later.

client_table_blueprints/android_output is a build artifact, not
code — it's not dead, just regenerate with the build script when the
blueprints change.

browser_ui_testing/drag_reorder/rapier_test, rapier_test_2,
rapier_collision_test, nav_page, nav_page_without_wizardry — siblings
that rapier_tests_combined was built from. Keep as reference; don't
edit.

## Build and environment gotchas — 2026-10-13

The z_db web build is a wasm artifact committed into the repo. It's
built with wasm-pack and copied into cqrs/web_interface/zdb_web_output
by update_hard_copied_z_db_web_output.sh. The copy overwrites
worker_wrapper.js, which holds a hardcoded connection name. That name
must be changed back to "cqrs" after every copy. Two projects with
the same name share the same OPFS database.

cargo prints "skipping duplicate package" warnings on every invocation
because /home/zakke/ProgStuff/ZakkeN is itself a clone of
Zakarias-Viinikka/ZakkeN, so cargo sees the same git URL at a local
path and in its checkout cache. Cosmetic.

nightly rustc hashes change daily. Every workspace with a target dir
built on the old hash fails every dep with "found crate X compiled by
an incompatible version of rustc". cargo clean in each, rebuild.

zap (the file-sharing tool) refuses mv on directories — use /bin/mv
in a real shell. chmod requires a real shell too.