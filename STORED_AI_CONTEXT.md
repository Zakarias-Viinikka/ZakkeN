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

LoveLetterResponse is a struct wrapping a LoveLetter plus a letter_id.
It's what the server pushes to a client: the letter_id is what the
client acks and what it uses to dedup if the same letter arrives twice.
serialize/deserialize are love_letter_response_to_bytes and
deserialize_love_letter_response.

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

## zakken_server — 2026-10-15

A workspace in three folders, shaped like rust_practice/server_template_3:

  shared/        error_stuff, server_protocol
  server_side/   db (package db_core), server_core, love_letter_parser,
                 queue, connected_clients
  mock_client/   mock_client_core, cli_gui

db_core — schemas and db operations, nothing else. Schemas in
db/src/db/schema/, one file per table, tbl_ prefix:
tbl_registered_users, tbl_love_letters_from_clients,
tbl_unsent_love_letters, tbl_socket_testing_table. There is no
tbl_pages here; db_core depends on client_table_blueprints and uses
its tbl_pages, so both sides share one definition. The logs and
key_value_storage tables also come from client_table_blueprints.
schema/migration/ is empty on purpose. Operations: user_registration,
operations_love_letters, operations_pages, operations_logs,
operations_key_value, apply_letter.

apply_letter.rs is the server's equivalent of cqrs's executor. It's
small because the server's job is small: it receives a snapshot, not a
character diff. CreateNewPage inserts a row; everything else loads the
page's blob, merges the snapshot in via BossOfYrs::merge_with_snapshot,
and saves. DisablePage does the same against the page_status blob via
YrsActivePages. It deliberately doesn't call cqrs's builder.

server_protocol — the server's own Message, Response, Request and
payload structs. Request variants: Register, Login, SendLoveLetter,
GetAllYrsPages, Ack. Every response payload is bincode
Result<T, DbError>, so the client can tell success from failure.

queue — Job, JobReply, Priority, QueueManager. No db knowledge. Every
client request goes through it. The executor (in server_core) drains
it.

connected_clients — owns the user_name -> UnboundedSender<i64> map.
No db knowledge, so it can never take the db lock; that fixes the
lock order at db-then-clients forever. Every method talks to one task
that owns the map.

server_core — the server. ServerState holds only the db connection.
do_request is async and returns HandlerOutcome { bytes,
just_identified, incoming_letters }. main.rs is wiring; setup.rs opens
the db; socket.rs holds handle_socket; queue_executor.rs holds the
executor loop, which also drains the log channel.

Delivery, event-driven, at-least-once. On a new letter: apply to
pages, store the letter, then for every registered user except the
sender — queue an unsent row, online or not. Then fan_out the letter
id to every connected user except the sender. On Ack: delete that
user's unsent row. On Login: register the socket's channel, then push
that user's unsent rows, oldest first. The rows stay until acked, so a
dropped connection gets the letter again on reconnect and the client
dedups by letter_id.

mock_client/ is a workspace inside a workspace. mock_client_core owns
the connection and the operations, no printing. cli_gui is the REPL.
mock_client_core decodes every response as Result<T, DbError>.

plan.md at ZakkeN/plan.md describes this server's current state.

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

## server_template_3 — 2026-10-15

~/ProgStuff/rust_practice/server_template_3 — the reference for how
this server is shaped, and the source of the reshape that landed in
zakken_server on 2026-10-15. server_template_2 is where the axum
websocket started; server_template_3 separates "receiving a request"
from "doing it" using the actor-queue shape from
rust_practice/actor_audition. The queue there ended up as two tokio
channels, one important and one normal, drained with a biased select
so important jobs go first.

zakken_server now has the same shape: shared/ and server_side/
folders, the queue between the socket and the executor, and
ConnectedClients in its own crate. The db and schema work did not come
from server_3 — server_3's is deliberately lazy and is not an example.

What zakken_server borrowed from server_template_3: the three-folder
workspace layout, the queue crate, the protocol crate shape (lib.rs as
pure re-exports over message.rs / request.rs / builders.rs), the
error_stuff shape (ServErr enum with DbErrorContainer / ServerError
variants, ErrorDetails with err_msg/file/method), and the mock-client
split (core crate with the logic, thin crate with the UI).

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

## E2E Android testing — 2026-10-17

e2e_android_tests/ — an external Python/Appium UI test harness for the
Android app. Not part of any Cargo workspace; not part of the Android
build. It drives the already-installed app on a physical phone over
adb, from the terminal on the PC. The phone is connected via wireless
debugging; `ip.env` holds the current `PHONE_IP:PHONE_PORT`.

Run it with `~/ProgStuff/ZakkeN/e2e_android_tests/run-ui-tests.sh`.
Rebuild the app first if app code changed (`android_app/build_and_install.sh`).
It writes artifacts to `e2e_android_tests/artifacts/<timestamp>/` — those
are diagnostic detail for the human, not for the Android AI.

The app provides two debug-only receivers (`android_app/app/src/debug/`)
for the harness. `TestTransactionReceiver` opens a DB transaction that
spans the whole test run and rolls it back at teardown, so a test cannot
leave user data modified. `TestNotificationReceiver` posts a silent
"Tests done" notification on the phone. See `android_app/docs/AI_READ_THIS.md`
for the actions, message shapes, and the deliberate 2-block minimum
that tests must account for.

The Android AI runs the tests itself, reads the terminal output, and
fixes what it reports. It never edits anything under `e2e_android_tests/`.

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

messing_around — UI style explorations and small Leptos playgrounds.
Not dead; the user looks at them for design ideas. Not part of the
build. feel_labs is a Leptos csr playground for testing how typing and
editing effects should feel; one lab per idea, a picker at the top,
no persistence. Its README.md and STORED_AI_CONTEXT.md live in
messing_around/feel_labs/.

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

## Build and environment gotchas — 2026-10-15

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

Cargo needs the package name on the left of a dependency line, not
the lib name. yrs's package name is "yrs" but its lib name is
"my_yrs_lib", so Cargo.toml says yrs.path = "..." and the code says
use my_yrs_lib::. Getting this backwards gives "no matching package
named my_yrs_lib".

Workspace crates can't share a name with a path or git dependency.
zakken_server's db crate was renamed to db_core for exactly this
reason — z_db also has a crate named db, and the lockfile can't hold
two.

local_cargo.sh is a per-directory wrapper around cargo. It sets
RUSTFLAGS for mold and then forwards to cargo. Each package that needs
a default binary puts one at its own root: zakken_server/local_cargo.sh
runs zakken_server (and `c t` there runs `cargo test --workspace`),
mock_client/local_cargo.sh runs cli_gui. A lookup walks up from the
current directory, so a directory with no local_cargo.sh of its own
finds the nearest one above it.

## E2E Android testing — 2026-10-17

e2e_android_tests/ — an external Python/Appium UI test harness for the
Android app. Not part of any Cargo workspace; not part of the Android
build. Drives the already-installed app on a physical phone over adb,
from the terminal on the PC. The phone is connected via wireless
debugging; `ip.env` holds the current `PHONE_IP:PHONE_PORT`.

Run with `~/ProgStuff/ZakkeN/e2e_android_tests/run-ui-tests.sh`. Rebuild
the app first if app code changed (`android_app/build_and_install.sh`).
Artifacts go to `e2e_android_tests/artifacts/<timestamp>/` — diagnostic
detail for the human, not for the Android AI.

App-side debug hooks (`android_app/app/src/debug/`):
`TestTransactionReceiver` opens a DB transaction that spans the whole
test run and rolls it back at teardown, so tests cannot leave user data
modified. `TestNotificationReceiver` posts a silent "Tests done"
notification on the phone. Actions and details in
`android_app/docs/AI_READ_THIS.md`.

The Android AI runs the suite itself, reads the terminal output, fixes
what it reports. Never edits anything under `e2e_android_tests/`.
