# ZakkeN - AI stored context

Written by AI for AI. Meant to be the first file a fresh AI reads when
starting work on this project. Update a section when its content
changes and bump that section's date. If code and this file disagree,
the code wins and this file should be corrected.

## What this is - 2026-10-13

ZakkeN is an offline-first Notion-style notes app for personal use, on
Android and the web. The goal is to store, edit, and find old notes
easily, sync seamlessly, need almost no config, and have a better UX
than what exists.

The domain is pages and blocks. A page is a list of blocks. A block is
text plus metadata. Every page's state lives in a yrs CRDT doc; the
sqlite mirror is queryable but not authoritative. Edits are described
as "love letters" - a serialized intent the client sends to the server
so both sides agree on what changed. The client stores its own copy in
client_love_letter while the server tracks which registered clients have
received which letters.

The server now works end to end for the shapes that exist. See the
zakken_server section below.

## Crate map - 2026-10-14

yrs (ZakkeN/yrs) - CRDT wrappers. BossOfYrs owns one page doc,
YrsActivePages owns the which-pages-are-disabled doc, YrsBacklinks owns
backlinks. Every block carries a frac_position (hex string from
smart_frac_index) that decides order; reads sort by (frac_position,
block_id). move_block(block_id, above, below) writes a new position, it
does not delete+insert. Wasm has no threads and no clock, so BossOfYrs::new
takes a unix_time string; IDs are built from that string, an atomic
counter, and a random part. anti_deadlock.rs wraps every public method
with a timeout; DEBUG_MODE is currently false on all targets (wasm
would panic on thread::spawn), which is a stopgap. The crate's lib name
is my_yrs_lib even though the package name is yrs - imports say
my_yrs_lib, Cargo.toml lines say yrs.path.

client_table_blueprints (ZakkeN/client_table_blueprints) - the schema.
tbl_pages, tbl_every_block_in_existence, tbl_client_love_letter,
tbl_incoming_love_letters, tbl_backlinks, tbl_key_value_storage,
tbl_logs, tbl_seen_server_letters. Each tbl_x.rs declares the columns as ColumnDefs, exports
SchemaColumn consts, and has a new_x_row() helper returning the
Vec<ColumnValue> for an insert. Table names are TABLE_NAME consts.
The server depends on this crate for tbl_pages so the page schema
can't drift between the two sides.

love_letter (ZakkeN/love_letter) - the contract. LoveLetterSketch is
the intent (EditBlock, CreateNewBlock, RemoveBlock, DisablePage,
CreateNewPage). LoveLetter is the same plus a snapshot_of_edit blob.
build_a_love_letter combines sketch + snapshot into a letter.
CreateNewPage carries is_main_menu_page in both shapes - the server
needs it to write the pages row and the letter is the only place it
can come from. bincode serialization. No dependency on yrs - TextEdit
and PositionToInsert live in yrs and are re-exported for use here.

LoveLetterResponse is a struct wrapping a LoveLetter plus a letter_id.
It's what the server pushes to a client: the letter_id is what the
client acks and what it uses to dedup if the same letter arrives twice.
serialize/deserialize are love_letter_response_to_bytes and
deserialize_love_letter_response.

text_diff (ZakkeN/text_diff) - character-level diff and squashing.
diff_logic::get_diff(old, new) -> DiffResult. diff_batching has the
combine logic. Clients batch edits before persisting; the batching
library was deliberately made dumb (caller owns the list of diff
results and calls combine when ready) because sharing batch state
across the FFI boundary was too much complexity.

cqrs_state_synchronization (ZakkeN/cqrs_state_synchronization) -
where the current work is. Layout:

- data_builder_for_operations_that_need_to_be_correct - the builder.
  Pure, runs anywhere. create_page, edit_block, disable_page each
  return a ctx struct with everything the executor needs.
- executor_of_what_the_builder_built_because_the_builder_shouldnt_touch_the_db -
  the executor. Reaches into web_internal_db, so wasm-only.
  insert_page_requires_three_db_inserts, edit_block_requires_three_db_inserts,
  disabled_page_requires_three_db_actions. Each wraps in a transaction,
  or joins an outer one if already open (start_transaction_if_not_in_one!).
- error_stuff - shared CqrsErr for the workspace.
- web_interface - the leptos csr web client. Routes: / and /Menu (menu),
  /DbGui (the orm, from web_internal_db). LocalPages is the menu's
  in-memory page list, read from db on load. Each row in the menu has
  an edit-title button and a disable-page button; each button owns
  its own builder -> executor -> db_helper chain.

The builder/executor split isn't by purpose - it's by target. The
builder prepares data and touches nothing, so it can run anywhere.
The executor is the only piece that reaches into web_internal_db,
which is wasm-only.

Docs in cqrs: CONTEXT.md (what to read), CURRENT_PLAN.md (the testing
plan, deferred), CURRENT_STATE.md (where things are now),
GENERAL_TODO.md, AI_UPDATES_THIS_DOC_EVERY_SESSION.md (reverse-
chronological session log - the best place to see recent work).

zakken_server (ZakkeN/zakken_server) - the server, and the thing this
project has been building lately. It is a workspace, not a single
crate. See the next section.

client_table_blueprints/android_output, yrs/android_output_folder,
love_letter/android_output_folder, text_diff/android_output_folder -
build artifacts the Android app downloads via update.sh. Regenerated
by each crate's build script; committed to the repo.

## cqrs work status - 2026-10-27 (web<-server link landed)

Most of the earlier near-term list is built now. `add_block`,
`edit_block`, `delete_block`, `move_block`, and
`move_block_to_another_page` all have a builder, an executor, and a
`ui_actions` entry point. `text_diff` is a dependency. The batcher is
live in `complicated_signal_flow/core.rs`, owned by `main.rs` and put
in context as `BatcherHandle`. Every write entry point flushes first.
Page-inside-page and the special menu (z+space opens, esc or space
again closes, down arrow selects) are designed but not started. The
loading system - preload, then offline, then server - is described in
ZakkeN/cache_and_preload.md. The general data management operations -
export, import, pause, wipe server, adopt, reconcile - are planned in
ZakkeN/plan_for_general_data_management_ops.md.

The Enter-split rule is "split at the cursor" now, not three newlines;
blocks appear in `page.blocks` before the DB has them (the batcher's
queue is an enum of Edit and AddBlock, run in order), and arrow keys
navigate between blocks. See the cqrs CURRENT_STATE.md for the shape.

The web client now talks to `zakken_server`. `client_internals/`
holds a shared crate that owns both the socket connection and the
response router (`Router::new` opens the socket, spawns the dispatch
and retry loops, returns a `RouterHandle`; per-target socket code lives
in `socket_web.rs` / `socket_android.rs`).
`web_interface/src/owner_of_interacting_with_server/` owns the
connection lifecycle (main_for_login_and_registration,
start_over_fresh, receive worker, sender, write lock). Sync
has been observed end to end. The receive worker re-runs login on
every reconnect, not just the first connect, because each new
server-side socket starts with no user attached.

The drag-reorder UI lives inside `web_interface`, in
`drag_reorder_container/`. The folder has three layers:
`drag_reorder_container.rs` is the wrapper the app calls,
`my_block_content.rs` is the block UI the human owns and edits, and
`drag_reorder_black_box/` is the drag machinery with a
renderer-for-blocks prop so the human never touches it. There is no
separate `pure_web_ui_components` crate; that was tried and folded
back in.

`leptos_components/main_content/` holds the page view.
`page_state.rs` at the crate root holds `Page` (one page's whole live
state - id, yrs doc, blocks, load state, preload request/response),
`PageLoadState`, and `OpenPages`. `shared_owners/` at the crate root
holds `BlockRow`, `LocalPages`, `BlockInputEvent`, `APP_LOCKED`,
`FOCUS_BLOCK_ID`, and `FOCUS_BLOCK_AT_START`. `Batcher` and
`BatcherHandle` live in `db/batcher/`. The load sequence is two effects
per page: one matching on load state and advancing one step, one
watching the preload response.

Moving a block within a page works end to end on web, including after
the frac rework. Cross-page move works on web. Back button still does
not re-render without a manual refresh; cause not diagnosed.

Focus-on-split was fixed on 2026-10-24: `.ps-box.ps-unmeasured` used
`visibility: hidden`, which leaves `document.activeElement` set on the
new textarea but does not route input to it. Replaced with
`opacity: 0; pointer-events: none`. `ZakkeN/tmp_focus_test` reproduces
the mechanic and can be deleted, or kept as a regression test.

### Update - 2026-10-30 (naming pass and batcher refactor)

A naming pass landed in cqrs, plus a batcher rewrite to match the new owner+handle convention. Guide: `ZakkeN/coding_style.md`.

Renames in `web_interface/src/`:
- `server_link/` → `owner_of_interacting_with_server/`. The folder owns all server-interaction logic.
- `receive.rs` → `main_for_receiving_spontaneous_love_letters_from_server.rs`.
- `identity.rs` → `main_for_login_and_registration.rs`.
- `merge.rs` → `main_for_applying_love_letter_from_server_to_app.rs`.
- `send.rs` → `send_unsent_love_letters_to_server_if_there_are_any.rs`.
- `two_tab.rs` → `two_tab_guard.rs`.
- `core.rs` stays `core.rs` (core to a folder is what main is to a crate).
- `db/kick_send.rs` → `db/main_send_unsent_love_letters.rs`, and function `kick_send()` → `main_send_unsent_love_letters()`. Behaviour unchanged; the name follows caller pov.

Batcher now matches owner+handle:
- `Batcher::new()` and `Batcher::run()` gone. `BatcherHandle::spawn_task_and_get_handle()` creates the channel, spawns the run loop on `spawn_local`, returns the handle.
- The pending list moved from a shared `RwSignal` into the run loop's own `Vec<PendingWrite>`.
- `BatcherMsg` is `Push(PendingWrite)` / `Flush { reply }`. The old `Poke` is gone; `Push` is the poke.
- `push_edit` / `push_add_block` send `BatcherMsg::Push`. `flush_now()` unchanged (sends `Flush { reply }` and awaits the oneshot).
- `batcher/mod.rs` no longer re-exports `Batcher`. Only `BatcherHandle`, `PendingWrite`, `run_exclusive`.
- `main.rs` calls `BatcherHandle::spawn_task_and_get_handle()`; the `spawn_local(batcher.run())` block is gone.

## zakken_server - 2026-10-28

A workspace in four folders, shaped like rust_practice/server_template_3:

  shared/        error_stuff, server_protocol
  server_side/   db (package db_core), server_core, love_letter_parser,
                 queue
  mock_client/   mock_client_core, cli_gui
  client_internals/ shared_internals_for_server

db_core - schemas and db operations, nothing else. Schemas in
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

server_protocol - the server's own Message, Response, Request and
payload structs. Request variants: Register, Login, SendLoveLetter,
GetAllYrsPages, Ack, GetMyUnsentLetters. Request-only payloads
(RegisterPayload, LoginPayload, SendLoveLetterPayload, AckPayload)
live in request.rs; reply-only payloads (PageRow,
GetAllYrsPagesPayload, UnsentLetter, GetMyUnsentLettersPayload) live
in responses.rs. lib.rs is pure re-exports, so no call site cares
which file a struct is in. Every reply payload is bincode
Result<T, DbError>, so the client can tell success from failure.

queue - Job, Priority, QueueManager. No db knowledge. Every client
request goes through it, carrying the socket id. The executor (in
server_core) drains it.

telephone - a module inside server_core now, not a crate. Owns the
socket id -> { notify sender, user name } map. The channel carries ()
and means "go pull"; the letters stay in the db. No db knowledge, so
it can never take the db lock; that fixes the lock order at
db-then-telephone forever. Every method talks to one task that owns
the map.

server_core - the server. The shared state is Arc<Mutex<LiveForever>>,
no wrapper struct. do_request is async and returns Vec<u8>; the
telephone is a module in this crate. main.rs is wiring; setup.rs opens
the db; socket.rs holds socket_main; queue_executor.rs holds the
executor loop, which also drains the log channel.

Delivery, event-driven, at-least-once. On a new letter: apply to
pages, store the letter, then for every registered user except the
sender - queue an unsent row, online or not. Then notify every
connected user except the sender. A notified socket submits
GetMyUnsentLetters for its own user and pushes each letter to the
client as its own LoveLetterFromServer message, so each is acked on its
own. On Ack: delete that user's unsent row. On Login: the socket
registered its channel at birth, so login attaches the name to the
socket id and notifies that socket once. The rows stay until acked, so
a dropped connection gets the letter again on reconnect and the client
dedups by letter_id.

mock_client/ is a workspace inside a workspace. mock_client_core owns
the connection and the operations, no printing. cli_gui is the REPL.
mock_client_core decodes every response as Result<T, DbError>.

plan.md at ZakkeN/plan.md describes this server's current state.

## connecting_to_server_template - 2026-10-29

ZakkeN/connecting_to_server_template - a self-contained Leptos CSR demo. Two halves ("client" and "server") in one app, talking through two futures::mpsc channels in client_server_coms.rs. No socket, no DB, not part of any Cargo workspace. It exists to be a reference for how connection status and the general data management operations (pause, resume, wipe, adopt, reconcile, dataset generation) should be shaped, so the real web client (cqrs_state_synchronization/web_interface) and the real server (zakken_server) can be modeled against it. Behaviour behind the enums and handle methods is faked: pretend-edit buttons, a pretend-reconcile modal, a mocked generation check. Names don't have to match the real thing exactly; the shape should. The plan in plan_for_general_data_management_ops.md is the reference for what shapes to mirror.

As of 2026-10-29 the work order has been implemented. Files:

- protocol.rs - Payload types and LoveLetter, small copy of the real server protocol. Canned page ids.
- sync_mode.rs - SyncMode enum (Connected / Paused / NeedsReconcile), static SYNC_MODE, is_gated, set_sync_mode (only writer). Persistence is a placeholder comment.
- owner_of_interacting_with_server.rs - ConnState (Offline / Connecting / FailedToConnect / Connected), generation and count signals, restart_sync_stack (soft restart, no page reload), receive_letter (gated), pending queue with main_send_unsent_love_letters (gated, drain re-checks per send), pull_unsent, start_over_fresh, check_login_generation (canned), clear_seen_everywhere (no-op).
- ui_ops.rs - pause (refused in NeedsReconcile), resume, wipe_server, adopt_local, DATASET_OP_BUSY, with_busy, read_local_pages_placeholder.
- reconcile.rs - at the time this was written: ConflictChoice, ReconcileDecisions, PageDiffSummary, ReconcilePlan, RECONCILE_PLAN/RECONCILE_ERROR signals, a session holding previous_mode + apply_attempted + plan, and the three verbs (plan / apply / cancel). classify_pages_placeholder branched on whether the server returned any pages. Superseded 2026-10-31 — see the "reconcile rework" update.
- overlays.rs - FullScreenBlocker, BusyOverlay, ReconcileModal. At this point ReconcileModal had an inner ReconcileModalBody that read the plan once untracked. Now it reads `local_pages` and mocks the process — see the 2026-10-31 update.
- cool_popup.rs - Toast stack with click-to-modal, hover-freeze, per-kind lifetime. show_popup(String) and show_popup_as(PopupKind, String). Replaces a separate popup.rs.
- server.rs - fake server with dataset_generation (starting at 1) and seed_pages, implements every request. Wipe bumps generation and clears pages. Adopt bumps generation and replaces pages. SendLetter logs and replies Ok. GetMyUnsentLetters returns empty. The "kick other sockets / stamp generation / guard SendLoveLetter on stale stamp" is not representable with one client and is a comment in Login. ServerPanel readout and ServerReadout context.
- client.rs - BorderComponent now takes BorderState (Offline / Paused / Connected), not a bool. ClientPanel shows mode, conn, pending, applied, both generations, and two pretend-edit buttons. ServerMessageReceiver passes pushed letters to receive_letter.
- controls.rs - coms + server_coms props. spawn_verb popups on Err. Connect, Pause sync, Resume sync, Adopt my data, Reconcile, Wipe server, Push pretend letter, and a Popups section (Show popup / Show error).
- app_main.rs - fragment root. .sides, ReconcileModal, BusyOverlay, CoolPopup as siblings. Server takes a started prop so its border stays grey until "Start server" is pressed.

Four gating points read is_gated(): receive_letter, kick_send, the per-send re-check inside kick_send's drain loop, and restart_sync_stack's tail (login runs, bootstrap and pull are skipped).

Deviations from the plan:
- cool_popup replaces popup.rs. Same show_popup(String) signature, plus show_popup_as(kind, msg). The plan's simple PopupBar was not created.
- Step 26 (inert on .sides) skipped: attr:inert is not accepted by the Leptos 0.8 view! macro and no fallback was added.
- Overlay CSS: one file per component under css/, loaded with include_str! inside the component. No shared stylesheet. Z-index order: blocker 100, busy 200, modal 300, popup 400.
- Sidebar border: paused uses #4a90e2 (blue), online uses #4caf50 (green), offline uses the default grey. The server border stays grey until "Start server" is pressed.

Expected non-bugs: server pages don't grow after a reconcile push, letters dropped while paused aren't re-delivered, and edits made before Connect may drain anyway (kick_send gates on mode only, not on connection state).

### Update - 2026-10-29 (state machine and feedback layer)

A second plan ("state machine and feedback layer") is implemented. Behaviour now follows R1–R9 (one popup per verb, refusals first, busy overlay on every server-awaiting verb, single-flight restart, ConnState meanings, two gates on sending, failure semantics, discards announced, mode reason). Fake-server stand-ins F1–F7 are in.

What changed since the shape-only build:

- Server power is real. It boots off. The fallback "Start server" reveals the panel AND fires `turn_on_server()`. Controls "Turn off server" / "Start server" toggle it after that. Power off keeps data; the socket-lost message fires.
- Fake latency (500ms light, 900ms heavy) via `leptos::set_timeout` + oneshot. `AreYouOnline`, Login, GetAllPages, SendLetter, GetMyUnsentLetters, Wipe, Adopt all defer their reply. Mutation happens at arrival; reply re-checks power and drops if off.
- Server backlog, socket-live flag, Push pretend letter goes through the server. GetMyUnsentLetters returns the backlog; Ack removes.
- Server page rows: CreateNewPage inserts a row if absent. Server panel shows power, generation, pages, unsent backlog, letters received, fault armed.
- "Fail next request" arms a one-shot that fails the next mutating request (Login, SendLetter, Wipe, Adopt). "Other client wipes" mints a new generation and fires socket-lost — the only way to reach the generation-changed branch.
- Client receive has a seen set. `clear_seen_everywhere` actually clears it now. Receive order: gated (bump ignored counter, no ack), already-seen (ack only), otherwise apply/mark/bump/ack.
- `restart_sync_stack` returns `RestartOutcome` (Synced / AlreadyStopped / JustChanged / Failed), never pops. `announce_connect_outcome` is the popup layer. Single-flight via `RESTART_IN_FLIGHT`.
- Drain loop gates on ConnState::Connected AND on `is_gated()`, and surfaces send failures as Error "Sending failed: ... N edits still queued" (server errors only; transport failures are silent because ConnectionLost already warned).
- Status line at the top of ClientPanel. Single derivation, nine rows in the plan, first matching wins.
- Border states: Offline / Connecting / Failed / Connected / NeedsReconcile / Lonewolf. `border-needs-reconcile` amber `#e8b060` (gated + not connected), `border-lonewolf` violet `#a78bfa` (gated + connected — the "connected but disconnected" case), `border-failed` red, `border-pulse` animation over any state while work is in flight, dashed-border fallback under reduced motion. "Lonewolf" is the name for the gated-but-connected state; `owner_of_interacting_with_server::is_lonewolf()` is the predicate. It is not a `SyncMode` variant.
- `CustomButton` gained `#[prop(optional, into)] disabled: Signal<bool>`. Controls button policy: client-section + Wipe disabled until client started; server power controls disabled when the appropriate polarity; everything else stays enabled and refuses with a popup (test bench).
- `VerbError` enum in `ui_ops` (Refused → Warning popup, Failed → Error popup). `spawn_verb` splits. `reconcile_plan` returns `Result<Option<ReconcilePlan>, VerbError>`; "already in progress" is an Info no-op not an error.
- Popup coalescing: `show_popup_as` resets the timer of a live toast with same kind and message instead of stacking.
- Client panel gained reason suffix, ignored-while-stopped counter, and amber coloring when login ≠ stored generation.

Behaviour changes since the earlier update:

- Paused was merged into NeedsReconcile. `SyncMode` is now just `Connected` / `NeedsReconcile`. The Pause button sets NeedsReconcile with reason `YouPaused`. There is no Resume button — the only exits are Reconcile, Adopt, or (soon) a dedicated "Wipe server & start from mine" that currently calls adopt.
- Mode and reason persist to localStorage and load on boot. Reload into paused stays paused. Connected resets to Connected (no persistence). Auto-connect on "Start client" fires unless mode is already NeedsReconcile.
- `ModeReason` extended with `YouPaused`.
- `CustomButton`, status line and border states all live in the client half; nothing on the server side invents behaviour.
- The Reconcile-actions group has four buttons: Reconcile, Adopt my data, Wipe server & start from mine, Wipe mine & start from server. The last one is the new "client becomes the server's mirror" verb (`ui_ops::wipe_local_and_pull_from_server`): fetches all pages, clears queue and seen, applies every server page locally (PLACEHOLDER — logs the count), records generation, sets Connected, restarts. `"Wipe server & start from mine"` still calls `adopt_local` — same behaviour as Adopt my data; if the human wants them distinct that is a plan question.

Known deviations still in place:

- Verb popups fire from inside `with_busy`, not strictly after the overlay clears. Both happen in the same tick and the popup sits above the overlay, so nothing is hidden.
- "Wipe server & start from mine" duplicates Adopt.

main.rs uses `connecting_to_server_template::app_main::AppMain`, matching the Cargo.toml [package] name. The old `connecting_to_server_example::` prefix was corrected at some point.

### Update - 2026-10-31 (reconcile rework: process, not plan)

Reconcile no longer computes a plan object. The old `reconcile/{plan, session, classify, modal}.rs` files are gone. The folder `reconcile/` is empty; the module is `reconcile.rs`.

The real model (written down in `plan_for_general_data_management_ops.md` section 4): the client passes `local_pages` only. The server checks each page — "do you have this exactly as I do?" — and sends its full version of any page that doesn't match. The comparison happens live inside that exchange. Nothing is stored between layers.

The template mocks this. `reconcile.rs` owns three verbs: `reconcile_plan` (sets `NeedsReconcile + Reconciling`, stores the previous mode/reason in a `Mutex`, opens the modal), `reconcile_apply` (mock — flips back to `Connected`, pops a success, restarts), `reconcile_cancel` (restores the stored mode/reason, closes the modal). No plan, no decisions map, no classify step.

State lives in `reconcile.rs` as two `ArcRwSignal`s (`MODAL_OPEN: bool`, `MODAL_ERROR: Option<String>`) exposed as `open_signal()` / `error_signal()`, plus a `Mutex<Option<(SyncMode, Option<ModeReason>)>>` for what cancel restores.

`overlays/reconcile_modal.rs` reads `local_pages` directly at render. Each page is a row; rows alternate between "server would send its version" and "matches, nothing to send", so both display shapes are visible. Confirm and Cancel take no arguments. No plan prop.

`client/status_line.rs` reads `reconcile::open_signal().get()` (bool), not a plan.
`client/components/controls.rs` reconcile button awaits `reconcile_plan`, no plan branch.
`app_main.rs` derives two signals (`open_sig`, `error_sig` from the two getters) and passes them plus two no-arg callbacks. The `ReconcileDecisions` prop and the plan prop are gone.

`css/reconcile_modal.css` dropped the summary grid, the conflict row styles, and the decisions styles. Kept card / head / body / note / error / page-row / page-state / foot / btn.

`behaviour_decisions.md` "Not decisions" section rewritten to state the process model and that the template's reconcile is a mock, not an implementation.

## External: z_db - 2026-10-13

~/ProgStuff/z_db - separate repo. Crates the ZakkeN projects use:
db, protocol, sql_builder, db_wrapper, web_internal_db,
migration_helper, migration_template, msg_build_n_debuild_helper.

Crates:
- db - rusqlite wrapper. black_magic.rs does the actual sqlite calls,
  black_magic_extension.rs is "extension" crud (count, etc), fts5_magic.rs
  is search, migration.rs is migration, transactions.rs is transaction
  control.
- protocol - shared by everyone. payload has the in/out structs;
  row_col has Col/Row; new_table has ColumnDef and the column helpers
  (id_column, not_null_col, not_null_unique_col, ForeignKeyDef);
  schema_helper has SchemaColumn<T> and TypeOfCol, plus
  destruct_db_col / try_destruct_db_col. The three-field error
  convention (err_msg, method, filepath) lives here. serialization.rs
  has Convert (bincode to/from Vec<u8>) and Base64Bytes.
- sql_builder - takes parameters, produces SQL.
- db_wrapper - the facade. macro_core.rs holds the methods once;
  per-target mascots (plain, serializer, android, web, testing) are
  generated from it. The server uses android_mascot::LiveForever.
  LiveForever's db_conn is private and every operation goes through a
  generated method (create_table, get_data, insert_data,
  insert_data_and_get_col, edit_col_in_row_where, delete_row_where,
  count_rows, list_tables, ...). Callers build the payload structs
  from protocol::payload. A new command has to be added to
  macro_core.rs, worker.js, and both db_helper.rs files by hand;
  nothing enforces agreement.
- web_internal_db - browser-facing. Runs inside a worker, talks to
  web_mascot, and exposes db_helper::{create_table, get_data,
  insert_data, get_single_col, count_all_rows, delete_all_rows,
  edit_col_in_row_where, edit_col_in_row,
  are_we_in_middle_of_transaction, begin_all_or_nothing,
  everything_went_perfectly, regret_everything, ...}. Also has db_gui.
- migration_helper / migration_template - how migrations are done.
  See migration_template/simple_real_usage_example.
- msg_build_n_debuild_helper - helpers for building/unbuilding
  messages across the wire.

ZakkeN apps import z_db's protocol and web_internal_db; the app never
touches rusqlite directly. The server imports protocol and
db_wrapper's android_mascot.

## server_template_3 - 2026-10-15

~/ProgStuff/rust_practice/server_template_3 - the reference for how
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
from server_3 - server_3's is deliberately lazy and is not an example.

What zakken_server borrowed from server_template_3: the three-folder
workspace layout, the queue crate, the protocol crate shape (lib.rs as
pure re-exports over message.rs / request.rs / builders.rs), the
error_stuff shape (ServErr enum with DbErrorContainer / ServerError
variants, ErrorDetails with err_msg/file/method), and the mock-client
split (core crate with the logic, thin crate with the UI).

## browser_ui_testing/drag_reorder - 2026-10-13

Experiments for a block-based editor's drag-reorder UI. The most
relevant crate is rapier_tests_combined - it merges the two-page
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

## Android and web targets - 2026-10-13

android_app - the Android build of the real app. Kotlin + Compose
UI on top of the same Rust crates via UniFFI. It's the second target
every crate has to keep building for. It is very stale: its docs
(SHARED_MEMORY.md, AI_READ_THIS.md, UX_INTENT.md, AI_LAB_RULE.md)
are out of date, and lots of its code was AI-driven. The docs still
describe the intent (block-based editing, mobile-first, buffered
edits, Lab for UX experiments) and are worth reading for that, but
don't trust them as a picture of current state.

web_client - superseded by cqrs_state_synchronization/web_interface.
The current web client is the cqrs one.

## E2E Android testing - 2026-10-17

e2e_android_tests/ - an external Python/Appium UI test harness for the
Android app. Not part of any Cargo workspace; not part of the Android
build. It drives the already-installed app on a physical phone over
adb, from the terminal on the PC. The phone is connected via wireless
debugging; `ip.env` holds the current `PHONE_IP:PHONE_PORT`.

Run it with `~/ProgStuff/ZakkeN/e2e_android_tests/run-ui-tests.sh`.
Rebuild the app first if app code changed (`android_app/build_and_install.sh`).
It writes artifacts to `e2e_android_tests/artifacts/<timestamp>/` - those
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

## Dead or parked - 2026-10-13

These exist but aren't part of the working set. Worth knowing about
because they sometimes have the only example of something.

csr_testing - drag-reorder with Sortable.js from a leptos csr app.
Superseded by the rapier drag_reorder experiments.

wasm/* - many small experimental crates (wasm_rusqlite_2, serde_sql_template,
yrs_integration, leptos_integration_1/2/3, and more). Everything here
is historical. Some of it (serde_sql_template, leptos_integration_3)
documents how the current z_db web stuff got its shape; if the user
asks "where was I doing X", the answer is often in here.

old_stuff - text_diff_old and an old zakken_server. Ignore.

messing_around - UI style explorations and small Leptos playgrounds.
Not dead; the user looks at them for design ideas. Not part of the
build. feel_labs is a Leptos csr playground for testing how typing and
editing effects should feel; one lab per idea, a picker at the top,
no persistence. Its README.md and STORED_AI_CONTEXT.md live in
messing_around/feel_labs/.

fts5 - an fts5 search demo. Parked but not deleted; may become the
search story later.

client_table_blueprints/android_output is a build artifact, not
code - it's not dead, just regenerate with the build script when the
blueprints change.

browser_ui_testing/drag_reorder/rapier_test, rapier_test_2,
rapier_collision_test, nav_page, nav_page_without_wizardry - siblings
that rapier_tests_combined was built from. Keep as reference; don't
edit.

rust_practice/server_template_0 through _3, server_ffi_recv,
actor_audition - reference implementations for websocket patterns.
Not part of ZakkeN, but worth reading when working on zakken_server.

## Build and environment gotchas - 2026-10-15

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
reason - z_db also has a crate named db, and the lockfile can't hold
two.

local_cargo.sh is a per-directory wrapper around cargo. It sets
RUSTFLAGS for mold and then forwards to cargo. Each package that needs
a default binary puts one at its own root: zakken_server/local_cargo.sh
runs zakken_server (and `c t` there runs `cargo test --workspace`),
mock_client/local_cargo.sh runs cli_gui. A lookup walks up from the
current directory, so a directory with no local_cargo.sh of its own
finds the nearest one above it.

## E2E Android testing - 2026-10-17

e2e_android_tests/ - an external Python/Appium UI test harness for the
Android app. Not part of any Cargo workspace; not part of the Android
build. Drives the already-installed app on a physical phone over adb,
from the terminal on the PC. The phone is connected via wireless
debugging; `ip.env` holds the current `PHONE_IP:PHONE_PORT`.

Run with `~/ProgStuff/ZakkeN/e2e_android_tests/run-ui-tests.sh`. Rebuild
the app first if app code changed (`android_app/build_and_install.sh`).
Artifacts go to `e2e_android_tests/artifacts/<timestamp>/` - diagnostic
detail for the human, not for the Android AI.

App-side debug hooks (`android_app/app/src/debug/`):
`TestTransactionReceiver` opens a DB transaction that spans the whole
test run and rolls it back at teardown, so tests cannot leave user data
modified. `TestNotificationReceiver` posts a silent "Tests done"
notification on the phone. Actions and details in
`android_app/docs/AI_READ_THIS.md`.

The Android AI runs the suite itself, reads the terminal output, fixes
what it reports. Never edits anything under `e2e_android_tests/`.

### Update - 2026-10-30 (structural refactor)

A structural refactor pass landed. Everything still works the same; the layout is now file+subfolder instead of one file per concern. Full plan archived at ZakkeN/artifacts/refactor_plan.md.

src/ layout now:

- lib.rs  main.rs  app_main.rs  protocol.rs  client_server_coms.rs  custom_button.rs  local_pages.rs
- sync_mode.rs + sync_mode/persistence.rs
- owner_of_interacting_with_server.rs + owner_of_interacting_with_server/{conn_state, dataset_generation, pending_queue, sending, receiving, restart}.rs
- ui_ops.rs + ui_ops/{verb_error, busy}.rs
- reconcile.rs + reconcile/{plan, session, classify, modal}.rs — superseded 2026-10-31; the folder is now empty and reconcile.rs owns the verbs and state directly.
- client.rs + client/{border, server_messages, status_line, panel}.rs
- server.rs + server/{power, latency, panel, border}.rs
- controls.rs (one file, five components: ControlsSection, ClientControls, StoppedSyncControls, PopupControls, ServerControls)
- overlays.rs + overlays/{full_screen_blocker, busy_overlay, reconcile_modal}.rs
- cool_popup.rs (deliberately NOT split — it is a black box, only show_popup / show_popup_as / CoolPopup / PopupKind are public)

Conventions:
- Parent file is `pub mod <child>;` lines only, plus its namesake item when it has one.
- Call sites use full submodule paths (`owner_of_interacting_with_server::sending::main_send_unsent_love_letters`, not re-exported).
- Only cool_popup keeps private submodules. (At 2026-10-30 reconcile also had a `pub use plan::{...}` shim; removed 2026-10-31 with the reconcile rework.)
- Statics stay private to their owner file, accessed through function getters.

Behaviour changes: none. Bugs and stale comments were preserved, not fixed.

Deviations from the plan:
- cool_popup stayed one file, not a folder.
- `.border-paused` deleted from sides.scss (dead after SyncMode collapse).
- `.btn:disabled` moved from sides.scss to button.scss.
- reconcile.rs had one re-export shim (`pub use plan::{...}`). Gone as of 2026-10-31 — there is no plan module.

## Plans at the repo root - 2026-10-30

`e2e_webapp_server_mock.md` — the implementation plan for the web E2E harness (`e2e_web_tests/`). Covers server `--test-mode` / `--db-path` / `--port` / `--ping-timeout` flags, mock client `--plain` mode, web client `test_config.rs` with `?test_mode=1&server_port=N` pinned in sessionStorage, the Python + Playwright harness layout, and one happy-path test. Not started.

### Update - 2026-10-30 (register flow, lonewolf message, retries, test-bench restructure)

The template grew the pieces needed to reason about re-registration and dataset replacement, so that behaviour could be exercised against the fake server without touching the real one.

**Register + login are now distinct.** `MessageToServer::Register` was added; the client's `log_in` tries `Register` first, falls through to `Login` on `ERR_USER_ALREADY_EXISTS`. First connect registers; every later connect logs in. Server-side `registered_users: RwSignal<Vec<String>>` tracks names, refuses a duplicate register with `ERR_USER_ALREADY_EXISTS`, and refuses a login for an unknown name with `ERR_NO_SUCH_USER`. Those two strings live in `protocol.rs` because both sides match on them.

**Test bench restructured.** The "Another client did this" group has three buttons: wiped / reconciled / started over from its own version. The old "Other client wipes" was removed from the Server section, and the two Popup buttons were removed entirely. Reconciled no longer bumps generation or kills the socket — a reconcile is a batch of letters, so it queues three `pretend_edit` letters and fans them out. The other two both replace the dataset and clear users.

**Server no longer disconnects on dataset replacement.** Both the wipe and the start-over-from-its-version simulations bump generation, clear pages and users and backlog, and then — only if the client was live — send `MessageToClient::ServerSaysYouAreNowLonewolf { what_happened }`. The socket stays up. If the client was offline when it happened, the state change is silent and it learns about it on next reconnect via `log_in` seeing a new generation.

**Client reacts by going lonewolf, not offline.** `handle_server_says_you_are_now_lonewolf` sets `NeedsReconcile` + `ServerChanged`, clears seen letters and pending edits, pops one warning naming what happened, then calls `owner_of_interacting_with_server::restart::kick_lonewolf_retry`. That runs `restart_sync_stack` up to 3 times at 2s intervals to re-register. Silent on success. The socket is never lost, so `force_offline` is no longer called from this path.

**Offline retry loop.** Separately, when the client is on and `ConnState` is `Offline` or `FailedToConnect`, `retry_reconnect_when_offline` (in `app_main.rs`) runs a 5s loop that pops "Trying to reconnect…" and calls `restart_sync_stack`. Exits on success (announcing via `announce_connect_outcome`, which correctly says "Server dataset changed…" if the reconnect discovers a wipe) or when the client is no longer offline.

**Title + visibility changes.** The Client box title now reflects state: `Client — Offline / Connecting / Failed to connect / Connected / Lonewolf`. "Lonewolf" is the `Connected + NeedsReconcile` combination. The stopped-sync buttons are wrapped in `<Show when=move || show_stopped.get()>` and only visible in that state — before this they were always rendered.

**Behaviour to expect when walking through it.** Connect → Connected. Click "Wiped the server" → one popup naming what happened, client stays Connected, title goes "Client — Lonewolf", stopped-sync buttons appear, ~2s later the retry re-registers silently. Then Reconcile / Adopt / Wipe-local-and-pull exit lonewolf back to Connected.
