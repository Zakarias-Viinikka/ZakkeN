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
uncommitted_diffs while the server tracks which registered clients have
received which letters.

The server now works end to end for the shapes that exist. See the
zakken_server section below.

## Crate map — 2026-10-14

yrs (ZakkeN/yrs) — CRDT wrappers. BossOfYrs owns one page's doc,
YrsActivePages owns the "which pages are disabled" doc, YrsBacklinks
owns backlinks. Wasm has no threads and no clock, so BossOfYrs::new
takes a unix_time string; IDs are built from that string, an atomic
counter, and a random part. anti_deadlock.rs wraps every public method
with a timeout; DEBUG_MODE is currently false on all targets (wasm
would panic on thread::spawn), which is a stopgap. The crate's lib name
is my_yrs_lib even though the package name is yrs — imports say
my_yrs_lib, Cargo.toml lines say yrs.path.

client_table_blueprints (ZakkeN/client_table_blueprints) — the schema.
tbl_pages, tbl_every_block_in_existence, tbl_uncommitted_diffs,
tbl_incoming_love_letters, tbl_backlinks, tbl_key_value_storage,
tbl_logs. Each tbl_x.rs declares the columns as ColumnDefs, exports
SchemaColumn consts, and has a new_x_row() helper returning the
Vec<ColumnValue> for an insert. Table names are TABLE_NAME consts.
The server depends on this crate for tbl_pages so the page schema
can't drift between the two sides.

love_letter (ZakkeN/love_letter) — the contract. LoveLetterSketch is
the intent (EditBlock, CreateNewBlock, RemoveBlock, DisablePage,
CreateNewPage). LoveLetter is the same plus a snapshot_of_edit blob.
build_a_love_letter combines sketch + snapshot into a letter.
CreateNewPage carries is_main_menu_page in both shapes — the server
needs it to write the pages row and the letter is the only place it
can come from. bincode serialization. No dependency on yrs — TextEdit
and PositionToInsert live in yrs and are re-exported for use here.

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

zakken_server (ZakkeN/zakken_server) — the server, and the thing this
project has been building lately. It is a workspace, not a single
crate. See the next section.

client_table_blueprints/android_output, yrs/android_output_folder,
love_letter/android_output_folder, text_diff/android_output_folder —
build artifacts the Android app downloads via update.sh. Regenerated
by each crate's build script; committed to the repo.

## zakken_server — 2026-10-14

A workspace. Members: db (package name db_core), server_core,
error_stuff, server_protocol, love_letter_parser,
mock_client/mock_client_core, mock_client/cli_gui.

db_core (directory db/) — schemas and db operations, nothing else.
Schemas in db/src/db/schema/, one file per table, tbl_ prefix:
tbl_registered_users, tbl_love_letters_from_clients,
tbl_unsent_love_letters, tbl_socket_testing_table. There is no
tbl_pages here; db_core depends on client_table_blueprints and uses
its tbl_pages, so both sides share one definition. schema/migration/
exists and is empty on purpose — every table is currently created by
a create_table_if_not_exist function at server startup, which is fine
while the schema is frozen and won't be the first time a column
changes. Operations live beside schema: user_registration.rs,
operations_love_letters.rs, operations_pages.rs, apply_letter.rs.

apply_letter.rs is the server's equivalent of cqrs's executor, and it
is small because the server's job is small: it receives a snapshot,
not a character diff. CreateNewPage inserts a row; everything else
loads the page's blob, merges the snapshot in via
BossOfYrs::merge_with_snapshot, and saves. DisablePage does the same
against the page_status blob via YrsActivePages. It deliberately
doesn't call cqrs's builder — create_page builds its own doc and
edit_block wants a DiffResult the server never has, so neither is
reachable for the incoming-letter path. The builder is reference, not
a dependency.

server_protocol — the server's own Message, Response, Request and
payload structs. Deliberately not z_db's protocol, which belongs to
the browser. lib.rs is just re-exports; message.rs, request.rs and
builders.rs hold the definitions, same shape as
rust_practice/server_template_3. Request variants: Register, Login,
SendLoveLetter, GetAllYrsPages. Every response payload is bincode
Result<T, DbError>, so the client can tell success from failure —
this matters, and an earlier version that used a bare
ok_serialized()/error and checked "is the payload empty" was wrong
because both are non-empty.

love_letter_parser — letter bytes -> target_page_id.

server_core — the server. ServerState holds the db connection and
ConnectedClients together behind one Mutex, because every handler
that touches the map also touches the db. ConnectedClients owns the
user_name -> UnboundedSender<i64> map; the i64 is a letter id. The
struct is the only thing that touches the map. allowed_requests/ has
one file per request. logging.rs holds the log helpers, and its style
is deliberately plain English ("updated its copy of page X") rather
than dumping JSON, because the raw dumps were unreadable.

The socket loop in main.rs selects on two things: the websocket and
the socket's own push channel. A push carries a letter id; the loop
loads the letter and writes it. Register and Login are different
messages on purpose: Register creates the account and fails if the
name is taken, Login identifies the socket and fails if the name
doesn't exist. Only Login makes a socket receive pushes.

Delivery is event-driven, no polling. On a new letter: apply to pages,
store the letter, then for every registered user except the sender —
online, push the id; offline, insert an unsent row. On Login: read
that user's unsent rows, push them, delete them.

mock_client/ is a workspace inside a workspace, so the CLI and the
logic don't live in one file. mock_client_core owns the connection,
the local page map, and the operations (login-or-register, load pages,
create page, edit, insert block, delete block, disable page). It has
no print statements. cli_gui is the REPL — a header, a "> " prompt,
and one short paragraph of output per action. mock_client_core
decodes every response as Result<T, DbError> and turns Err into a
plain String, so the CLI never sees wire types.

plan.md at ZakkeN/plan.md describes this server's current state and
what's left. Read it after this file.

## External: z_db — 2026-10-13

~/ProgStuff/z_db — separate repo. Crates the ZakkeN projects use:
db, protocol, sql_builder, db_wrapper, web_internal_db,
migration_helper, migration_template, msg_build_n_debuild_helper.

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
  convention (err_msg, method, filepath) lives here. serialization.rs
  has Convert (bincode to/from Vec<u8>) and Base64Bytes.
- sql_builder — takes parameters, produces SQL.
- db_wrapper — the facade. macro_core.rs holds the methods once;
  per-target mascots (plain, serializer, android, web, testing) are
  generated from it. The server uses android_mascot::LiveForever.
  LiveForever's db_conn is private and every operation goes through a
  generated method (create_table, get_data, insert_data,
  insert_data_and_get_col, edit_col_in_row_where, delete_row_where,
  count_rows, list_tables, ...). Callers build the payload structs
  from protocol::payload. A new command has to be added to
  macro_core.rs, worker.js, and both db_helper.rs files by hand;
  nothing enforces agreement.
- web_internal_db — browser-facing. Runs inside a worker, talks to
  web_mascot, and exposes db_helper::{create_table, get_data,
  insert_data, get_single_col, count_all_rows, delete_all_rows,
  edit_col_in_row_where, edit_col_in_row,
  are_we_in_middle_of_transaction, begin_all_or_nothing,
  everything_went_perfectly, regret_everything, ...}. Also has db_gui.
- migration_helper / migration_template — how migrations are done.
  See migration_template/simple_real_usage_example.
- msg_build_n_debuild_helper — helpers for building/unbuilding
  messages across the wire.

ZakkeN apps import z_db's protocol and web_internal_db; the app never
touches rusqlite directly. The server imports protocol and
db_wrapper's android_mascot.

## server_template_3 — 2026-10-13

~/ProgStuff/rust_practice/server_template_3 — the reference for how
this server is shaped. server_template_2 is where the axum websocket
started; server_template_3 separates "receiving a request" from
"doing it" using the actor-queue shape from
rust_practice/actor_audition (a priority list, a wake channel so the
worker doesn't spin when idle). That refactor hasn't landed in
zakken_server — zakken_server still locks a Mutex around one
ServerState, which is fine at this scale.

What zakken_server borrowed from server_template_3: the protocol crate
shape (lib.rs as pure re-exports over message.rs / request.rs /
response.rs / builders.rs), the error_stuff shape (ServErr enum with
DbErrorContainer / ServerError variants, ErrorDetails with
err_msg/file/method), and the mock-client split (core crate with the
logic, thin crate with the UI).

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
looks at them for design ideas. Not part of the build.

fts5 — an fts5 search demo. Parked but not deleted; may become the
search story later.

client_table_blueprints/android_output is a build artifact, not
code — it's not dead, just regenerate with the build script when the
blueprints change.

browser_ui_testing/drag_reorder/rapier_test, rapier_test_2,
rapier_collision_test, nav_page, nav_page_without_wizardry — siblings
that rapier_tests_combined was built from. Keep as reference; don't
edit.

rust_practice/server_template_0 through _3, server_ffi_recv,
actor_audition — reference implementations for websocket patterns.
Not part of ZakkeN, but worth reading when working on zakken_server.

## Build and environment gotchas — 2026-10-14

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

Cargo needs the package name on the left of a dependency line, not
the lib name. yrs's package name is "yrs" but its lib name is
"my_yrs_lib", so Cargo.toml says yrs.path = "..." and the code says
use my_yrs_lib::. Getting this backwards gives "no matching package
named my_yrs_lib".

Workspace crates can't share a name with a path or git dependency.
zakken_server's db crate was renamed to db_core for exactly this
reason — z_db also has a crate named db, and the lockfile can't hold
two.

local_cargo.sh is a per-directory script the user's shell function
`c` looks for by walking up from cwd. It sets RUSTFLAGS for mold and
then forwards to cargo. Each package that needs a default binary puts
one at its own root: zakken_server/local_cargo.sh runs zakken_server,
mock_client/local_cargo.sh runs cli_gui. If a directory has no
local_cargo.sh, `c r` finds the nearest one above it, which is often
the wrong thing.