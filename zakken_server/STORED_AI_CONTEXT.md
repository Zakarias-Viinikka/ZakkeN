# zakken_server - AI stored context

Written by AI for AI. Read this before touching the server. Update the
relevant section when its content changes and bump that section's date. If
code and this file disagree, the code wins and this file should be
corrected.

The project-wide picture lives one level up, in
ZakkeN/STORED_AI_CONTEXT.md. This file is about the server only.

## What this is - 2026-10-22

The axum websocket server for ZakkeN. Clients describe edits as "love
letters" - a serialized intent. The server applies each letter to its own
copy of the pages, stores the letter, then nudges the other registered
clients so each one can pull whatever it hasn't received. It's the only
place that knows which clients are online and which clients still need
which letters.

## Workspace layout - 2026-10-27

A cargo workspace in three folders. The split mirrors
rust_practice/server_template_3.

shared/ holds what more than one side needs:
- server_errors - ServErr, ErrorDetails (err_msg/file/method/probable_cause),
  and the log types LogCategory, LogLevel, ServerLogEntry. Package name
  is `server_errors`; the folder is `shared/server_errors`.
- server_protocol - the server's own Message/Response/Request and payload
  structs. Not z_db's protocol; that one belongs to the browser.

server_side/ holds the server's own crates:
- db (package name db_core) - schemas and db operations, nothing else.
- server_core - the server binary, the request handlers, the queue
  executor, and the telephone module.
- love_letter_parser - letter bytes -> target_page_id.
- queue - Job, Priority, QueueManager. No database knowledge.

client_internals/ holds how a client talks to the server:
- shared_internals - owns the socket connection AND the
  response router. `Router::new(url, on_love_letters, on_conn_status)`
  opens the socket, spawns the dispatch loop and the retry loop, and
  returns a `RouterHandle`. The `Router` struct never escapes; callers
  only see `RouterHandle`. Per-target socket code lives in
  `src/socket_web.rs` (web_sys::WebSocket,
  wasm_bindgen_futures::spawn_local) and `src/socket_android.rs`
  (stub). Both expose `spawn(url, outgoing_rx, incoming_tx,
  on_conn_status)` and `spawn_dispatch(router)`, so `Router::new` is
  identical for both targets. `ConnStatus` (Connecting / Connected /
  Failed) is passed to `on_conn_status` on every change.
- web - stub. The browser socket moved into shared_internals.
- android - stub.

The prose version of the client's side is guide_to_interacting_with_server.md
at the root of zakken_server.

## db_core - 2026-10-15

Schemas in db/src/db/schema/, one file per table, tbl_ prefix:
tbl_registered_users, tbl_love_letters_from_clients,
tbl_unsent_love_letters. There is no tbl_pages
here - db_core depends on client_table_blueprints and uses its tbl_pages,
so the client and server share one definition of that table. The logs and
key_value_storage tables also come from client_table_blueprints.

schema/migration/ exists and is empty on purpose. Every table is created by
a create_table_if_not_exist function at server startup, which is fine while
the schema is frozen and won't be the first time a column changes.

Operations live beside schema:

- user_registration.rs - create_table_if_not_exist, user_exists,
  create_user. create_user errors if the name is already taken.
- operations_love_letters.rs - the two letter tables.
  create_tables_if_not_exist, insert_letter, get_letter,
  get_all_registered_user_names, get_unsent_letter_ids_for_user,
  user_has_unsent_row, queue_unsent_for_user, delete_unsent_row.
- operations_pages.rs - the pages table. create_table_if_not_exist,
  page_exists, get_page, get_all_pages, insert_page, update_page_blob,
  update_page_status.
- operations_logs.rs - create_table_if_not_exist, insert_log. Writes rows
  to the logs table.
- operations_key_value.rs - create_table_if_not_exist, get_value,
  bump_session_id. bump_session_id reads the current id, increments it,
  writes it back, and returns the new one, so every server run has its own
  id.
- apply_letter.rs - the server's executor. Takes a LoveLetter, writes the
  pages table.

apply_letter.rs is small because the server's job is small. The client
sends a snapshot (a full CRDT state, or a diff against a known state), not
a character diff. So CreateNewPage inserts a row, and everything else loads
the page's blob, merges the snapshot in via BossOfYrs::merge_with_snapshot,
and saves. DisablePage does the same against the page_status blob via
YrsActivePages. It deliberately does not call cqrs's builder: create_page
builds its own doc, edit_block wants a DiffResult the server never has, so
neither is reachable for the incoming-letter path. The builder is
reference, not a dependency.

## server_protocol - 2026-11-10

The server's own Message, Response, Request and payload structs. Its
lib.rs is pure re-exports; message.rs, request.rs and builders.rs hold the
definitions.

Message is { message_id, request, payload, drift_version }. The drift
version is the client's claim about what the server's is; the executor
compares it against the current row and refuses mismatches. Register,
Login and GetCurrentDrift are exempt, and so are the socket's own
internal pulls (which travel the queue with no client on either end).

Request variants: Register, Login, SendLoveLetter, GetAllYrsPages, Ack,
GetMyUnsentLetters, WipeDataset, GetCurrentDrift. Ack carries an
AckPayload { letter_id: i64 } - the client saying it has applied that
letter, so the server can delete that user's unsent row.
GetMyUnsentLetters carries no payload and returns a
GetMyUnsentLettersPayload { letters: Vec<UnsentLetter>, drift_version:
String }, where each UnsentLetter is { letter_id: i64, love_letter:
Vec<u8> } and drift_version is the server's current one, stamped onto
every push the socket makes from this pull. WipeDataset carries a
WipeDatasetPayload { drift_solution: Option<DriftSolution> }, wipes the
dataset, mints a new drift version carrying that solution, notifies
every other connected socket to close after telling its client, and
returns the new version. GetCurrentDrift carries no payload and returns
the current drift version + reason + solution; it's exempt from the
drift check because asking is how you find out you were stale.

Response is its own enum now, not a reuse of Request. `Response::Reply {
message_id, request, payload }` is a reply to a specific client request;
`message_id` matches the request the client sent.
`Response::LoveLetterFromServer { letter, drift_version }` is a server
push: one letter plus the drift version the server is on, no request
id, applied and acked by the client on a matching version, dropped
otherwise. `Response::TheDriftVersionChanged { drift_version,
drift_reason }` is the message the server sends a connected client just
before closing its socket, because the dataset was replaced. The
client reconnects and re-logs-in from there.

Every response payload is a bincode-encoded Result<T, ServErr>. ServErr
is server_errors' own type, and wraps the db's DbError when the failure
really is a db failure. Its variants: DbErrorContainer(DbError) for db
faults, ServerError(ErrorDetails) for internal-but-recoverable,
ClientGone { user_name } for a disconnected client, RejectedBecause
(String) for client refusals, BadCode(String) for internal code faults
that aren't db, and DriftVersionMismatch for the drift check. If you
add a request, its handler must return Result<ItsPayloadType, ServErr>.

## server_core - 2026-11-10

There's no wrapper struct around the db any more. The shared state is
`Arc<std::sync::Mutex<LiveForever>>`; handlers lock it, take `&LiveForever`,
and drop the guard before awaiting. The Telephone lives in this crate too,
as the `telephone` module, behind its own handle. The db lock may be held
while calling Telephone, never the reverse: the telephone's task never
touches the db.

Every client request goes socket -> queue -> executor -> handler -> reply.
The socket parses the websocket message, submits a Job to the QueueManager,
and awaits the reply. One executor task, started from main, takes jobs
(important channel first, then normal, via a biased select), calls
do_request, and sends the result back through the job's oneshot.

A Job carries the request, the payload, a message id, a socket id, and a
WhereTheRequestCameFrom (an enum: FromAClient(Option<String>) for a
message that came in off some client's socket, FromTheServer for the
socket's own internal pull). The Option is None for Register, Login and
GetCurrentDrift, which have no version to send. The socket id is a u64 minted from an
atomic counter when the socket is born. It names a connection before
anyone has logged in, which is what lets the socket register its nudge
channel at birth instead of at login.

do_request is async. Its signature is (request, payload, state, telephone,
socket_id, where_the_request_came_from) -> Vec<u8>, where `state` is
`&Arc<Mutex<LiveForever>>`. Every handler returns bytes and nothing else.
Two gates live here. The login gate asks the telephone for the name
attached to the socket id; a socket with no name may only call Register
or Login. The drift gate (in
core::refuse_if_the_client_is_on_a_stale_drift_version) skips Register,
Login, GetCurrentDrift and FromTheServer pulls, and for everything else
compares the client's claimed drift_version against the current row,
refusing a mismatch with ServErr::DriftVersionMismatch. Register,
GetAllYrsPages, Ack, GetMyUnsentLetters, WipeDataset and GetCurrentDrift
lock the db briefly, do their work, and drop the guard; Login and
SendLoveLetter do their own locking because they need to await the
Telephone in between db calls. Each handler has a named fn in
allowed_requests/: register_user, send_all_pages, acknowledge_letter,
send_unsent_letters, log_in, accept_letter, get_current_drift.

Login does two things beyond checking the user exists: it tells the
telephone to attach the name to the socket id, then tells the telephone
to notify that socket. A notify is "go pull": the telephone carries no
letters. The socket's own pull is what brings the backlog.

The executor also drains a log channel: anything that wants to write a log
row sends a ServerLogEntry down it, and the executor is the one that
actually inserts, because it's the only thing that owns the db. log_error
(in logging.rs) prints and sends an entry. The session id is read at
startup by setup::setup_db and installed into logging.rs as a global, so
every row from this run carries it.

main.rs is wiring only. setup.rs opens the db and creates every table.
socket.rs holds socket_main, the per-connection entry point.
queue_executor.rs holds the executor loop. lib.rs re-exports all three.

The socket in socket.rs is a `SocketState`: the websocket, the socket
id, a log label, and a notify receiver. Its loop selects on two things:
the websocket and its notify channel. The channel is created at birth and
handed to the telephone at birth, so the socket listens from the moment
it exists - login changes nothing about the channel, it only gives the
telephone a name to reach it by. A `TelephoneCommand` arriving on the
channel tells the socket what the telephone wants. Two values:
`GoPullYourUnsentLetters` means pull; the socket submits
GetMyUnsentLetters through the queue for itself (message_id 0, which
never reaches the client), then pushes each letter back to the client as
its own `Response::LoveLetterFromServer { letter, drift_version }`
message, one letter per message, so each is acked on its own. A client
that misses a push still pulls on the next notify, because the unsent
rows stay until acked. `TellTheClientTheDriftVersionChangedThenClose`
means the socket is stale: it submits GetCurrentDrift through the queue
for itself, sends the client `Response::TheDriftVersionChanged {
drift_version, drift_reason }`, and breaks out of its loop, which closes
the websocket.

After a Login reply comes back, the socket asks the telephone who it is
(user_of(socket_id)) and uses that name as its log label. If the login
failed there is no name and the label stays the address. When the loop
ends, the socket calls telephone.remove_socket(socket_id).

## telephone - 2026-11-10

A module inside server_core (src/telephone/), not a crate any more. It
used to be the connected_clients crate; it moved because the socket id
design ties the telephone and the socket loop into one story.

Exposes one struct, `Telephone`. It's a handle to a task that owns a
HashMap<u64, SocketEntry> - socket id -> { notify sender, user name }.
The per-socket channel carries a `TelephoneCommand` enum, not `()`:
`GoPullYourUnsentLetters` means "you have something to pull", and
`TellTheClientTheDriftVersionChangedThenClose` means "the drift version
just changed and you're stale; ask the executor for the current one,
tell the client, close". `notify_drift(Some(socket_id))` sends it to
every logged-in socket except the one with that id; wipe calls it after
minting a new drift version, so its own caller stays connected and
everyone else is told and closed. Sockets whose channel is already gone
are dropped from the map and logged, same as `notify_all`. The
telephone carries no letters; the letters stay in the db as unsent rows.
Every method sends a Command to the task; the task is the only thing
that touches the map, and it runs on tokio, draining one channel.

Telephone is Clone. register_socket and remove_socket are sync and
fire-and-forget: a socket registers at birth and removes itself at
disconnect. attach_name(socket_id, name) is async and waits until the task
has recorded the name, so the executor's login gate can rely on a
following user_of seeing it. user_of(socket_id) returns the name attached
to a socket, or None. notify_socket(socket_id) notifies one connection;
notify_all(except) notifies every logged-in connection except `except`.
is_connected(user_name) and connected_user_names() ask the map without
touching it; nothing uses them right now except a test.

notify_all skips entries with no name: an entry that exists but hasn't
logged in can't have a letter queued for it, so there is nothing for it to
pull. That, plus the one-job-at-a-time executor, is why a live letter
can't fan out to a user between attach_name and the login notify.

When a client's channel turns out to be gone (their socket task dropped
the receiver), notify_all drops that client from the map and reports it in
the returned NotifyReport { reached, dropped }. The caller logs the drops.
The task also sends a ServerLogEntry down the log channel it was given at
construction; that's how this module reports without owning the db.

## Delivery - 2026-10-27

Event-driven, no polling, at-least-once. Pull model on the client side:
the server nudges, the client fetches.

On a new letter: deserialize, apply_letter to update the pages table,
insert the letter (this assigns the letter id), read the registered user
list, then for every registered user except the sender - queue an unsent
row for that user, online or not. Then notify_all(sender). Every connected
client except the sender's socket is told "go pull". That socket submits
GetMyUnsentLetters through the queue for its user, gets every letter still
rowed for it, and pushes each to the client as its own
LoveLetterFromServer(letter) message. The client applies and acks each
one. Offline clients aren't connected, so notify_all doesn't reach
them; their unsent rows sit until they log in.

On Login: the socket registered its channel at birth, so login only
attaches the name to the socket id and then notifies that socket once. The
socket pulls its backlog through GetMyUnsentLetters and pushes each letter
to the client. The rows stay until each is acked.

On Ack: the client sends the letter id it applied. The server deletes
that user's unsent row for that letter. A repeated ack for an already
deleted row is a no-op.

The client is expected to dedup by letter_id, because a duplicate is
possible: the server nudged, the connection dropped before the ack, and
on reconnect the row is still there and shows up in the next pull.

The push wire format is `Response::LoveLetterFromServer { letter,
drift_version }`, one message per letter, carrying the current drift
version so the client can throw away a letter built on a dataset it
isn't on. The drift-changed push is `Response::TheDriftVersionChanged {
drift_version, drift_reason }`, sent just before the socket closes. The
pull wire format is a Message with request = GetMyUnsentLetters and no
payload; the reply carries a bincode of
Result<GetMyUnsentLettersPayload, ServErr>, and the payload itself
carries the current drift version alongside the letters. The pull is
what the socket uses internally on a notify; the client never sends it
directly.

## Gotchas - 2026-10-22

Cargo needs the package name on the left of a dependency line, not the lib
name. yrs's package name is "yrs" but its lib name is "my_yrs_lib", so
Cargo.toml says yrs.path = "..." and the code says use my_yrs_lib::.

The db crate is named db_core (package) even though its directory is db/,
because z_db also has a crate named db and the lockfile can't hold two.

The db file path is relative to the directory the server is launched from
(personal_db_wrapper uses "zakken.sqlite"). Do not change the launch
directory, or the server silently opens a fresh empty database.

Never hold the db lock across an .await. If axum complains that the
socket_main future isn't Send, that is the cause. A socket must also
never hold the db lock while awaiting its queue reply, because the
executor needs that same lock to produce the reply and it would deadlock.
(The socket task itself no longer touches the db at all; the rule still
matters for login and send_love_letter, which lock briefly and drop
before awaiting.)

local_cargo.sh is a per-directory wrapper around cargo. `c c` checks the
whole workspace with `--all-targets`, so `#[cfg(test)]` code is included,
and also checks client_internals/web (wasm target) and
client_internals/android. `c t` runs `cargo test --workspace`. `c r` runs
zakken_server from the zakken_server root.

yrs's anti_deadlock prints "DEBUG_MODE is OFF (direct call, no timeout)" on
every method call. That's from the yrs crate, not from the server.

## What's left - 2026-10-27

Cross-client delivery has been exercised end to end. On 2026-10-27 the
web client and the mock client were run side by side. The web client's
queued letters flushed to the server (20 in one burst after a stuck
send loop was fixed), the mock client synced them, and a mock-client
edit pushed to the web client and was acked. The receive path
(`Response::LoveLetterFromServer` -> apply -> ack) works in both
directions.

The mock client now creates pages with is_main_menu_page = true, so
mock-created pages appear in the web menu.

Retention - deleting a letter once every registered user has acked it - is
deferred. A letter with no unsent rows is harmless in the table. Note that
"no unsent rows" now means "everyone has acked", which is different from
the old meaning.

Migrations. schema/migration/ is empty on purpose. See db_core above.

## Reconcile handlers - 2026-11-11

Three new request handlers in `allowed_requests/`, plus a fourth for
version reads:

- `get_page_fingerprints.rs` - returns page_id + fingerprint for every
  page the server has, plus the current drift version. Fingerprint is
  SHA-256 of raw `blobbed_page` bytes (via `drift_guard_shared::fingerprint`).
- `get_pages_by_ids.rs` - takes `page_ids: Vec<String>`, returns full
  data for the ones it has. Unknown ids are skipped silently.
- `get_reconcile_version.rs` - returns the current reconcile_version
  (a "0" default if never bumped, from `operations_key_value::get_reconcile_version`).
- `reconcile_letters.rs` - the write handler. Decodes the whole batch
  first (any decode failure rejects the whole batch with nothing
  applied), then opens ONE transaction, checks both
  `target_drift_version` and `target_reconcile_version` against the
  current rows, loops `apply_one_letter` for each letter, bumps
  reconcile_version inside the same transaction, commits, then
  notifies every connected socket except the sender's once. Any
  failure rolls the whole batch back.

All four are exempt from the drift check. The first three are reads a
lonewolf client needs; `reconcile_letters` checks its own target
versions inside the handler, so the outer gate doesn't apply.

`send_love_letter.rs` gained `apply_one_letter` (pub(crate)), extracted
from what used to be inline in `accept_letter`. Same four steps:
apply, insert, queue unsent for every user except the sender, return
the letter_id. Does NOT open its own transaction. `accept_letter` was
renamed `main_for_accepting_a_letter` and now wraps its single
`apply_one_letter` call in a begin/commit, same behaviour as before.
`reconcile_letters` opens its own transaction around the loop.

`operations_key_value.rs` gained `get_reconcile_version` and
`bump_reconcile_version`. The version lives under the
`reconcile_version` key. Bumped inside the transaction that applies
the batch, so a failure leaves it unchanged.

`server_core/Cargo.toml` gained `drift_guard_shared` and
`diff_viewer_internals` as path deps. The server now imports the same
SHA-256 fingerprint helper the web client uses.

New variants on `Request`: `GetPageFingerprints`, `GetPagesByIds`,
`GetReconcileVersion`, `ReconcileLetters`. New payloads:
`GetPagesByIdsPayload`, `ReconcileLettersPayload`,
`GetPageFingerprintsPayload`, `PageFingerprint`,
`GetReconcileVersionPayload`. `ReconcileLettersPayload` carries
`target_drift_version: String`, `target_reconcile_version: String`,
`letters: Vec<Vec<u8>>`.

See `../nov11.md` for the design decisions and `../plan.md` for the
flow summary.
