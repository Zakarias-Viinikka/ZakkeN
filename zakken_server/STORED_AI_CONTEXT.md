# zakken_server — AI stored context

Written by AI for AI. Read this before touching the server. Update the
relevant section when its content changes and bump that section's date. If
code and this file disagree, the code wins and this file should be
corrected.

The project-wide picture lives one level up, in
ZakkeN/STORED_AI_CONTEXT.md. This file is about the server only.

## What this is — 2026-10-15

The axum websocket server for ZakkeN. Clients describe edits as "love
letters" — a serialized intent. The server applies each letter to its own
copy of the pages, stores the letter, and pushes it to the other
registered clients. It's the only place that knows which clients are
online and which clients still need which letters.

## Workspace layout — 2026-10-15

A cargo workspace in three folders. The split mirrors
rust_practice/server_template_3.

shared/ holds what more than one side needs:
- error_stuff — ServErr, ErrorDetails (err_msg/file/method/probable_cause),
  and the log types LogCategory, LogLevel, ServerLogEntry.
- server_protocol — the server's own Message/Response/Request and payload
  structs. Not z_db's protocol; that one belongs to the browser.

server_side/ holds the server's own crates:
- db (package name db_core) — schemas and db operations, nothing else.
- server_core — the server binary, the request handlers, and the queue
  executor.
- love_letter_parser — letter bytes -> target_page_id.
- queue — Job, JobReply, Priority, QueueManager. No database knowledge.
- connected_clients — owns the user_name -> push-sender map, behind one
  task. No database knowledge, so it can never take the db lock.

mock_client/ holds the mock client, at the top level:
- mock_client_core — mock client logic, no printing.
- cli_gui — the mock client's REPL.

## db_core — 2026-10-15

Schemas in db/src/db/schema/, one file per table, tbl_ prefix:
tbl_registered_users, tbl_love_letters_from_clients,
tbl_unsent_love_letters, tbl_socket_testing_table. There is no tbl_pages
here — db_core depends on client_table_blueprints and uses its tbl_pages,
so the client and server share one definition of that table. The logs and
key_value_storage tables also come from client_table_blueprints.

schema/migration/ exists and is empty on purpose. Every table is created by
a create_table_if_not_exist function at server startup, which is fine while
the schema is frozen and won't be the first time a column changes.

Operations live beside schema:

- user_registration.rs — create_table_if_not_exist, user_exists,
  create_user. create_user errors if the name is already taken.
- operations_love_letters.rs — the two letter tables.
  create_tables_if_not_exist, insert_letter, get_letter,
  get_all_registered_user_names, get_unsent_letter_ids_for_user,
  user_has_unsent_row, queue_unsent_for_user, delete_unsent_row.
- operations_pages.rs — the pages table. create_table_if_not_exist,
  page_exists, get_page, get_all_pages, insert_page, update_page_blob,
  update_page_status.
- operations_logs.rs — create_table_if_not_exist, insert_log. Writes rows
  to the logs table.
- operations_key_value.rs — create_table_if_not_exist, get_value,
  bump_session_id. bump_session_id reads the current id, increments it,
  writes it back, and returns the new one, so every server run has its own
  id.
- apply_letter.rs — the server's executor. Takes a LoveLetter, writes the
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

## server_protocol — 2026-10-15

The server's own Message, Response, Request and payload structs. Its
lib.rs is pure re-exports; message.rs, request.rs and builders.rs hold the
definitions.

Request variants: Register, Login, SendLoveLetter, GetAllYrsPages, Ack.
Ack carries an AckPayload { letter_id: i64 } — the client saying it has
applied that letter, so the server can delete that user's unsent row.

Every response payload is a bincode-encoded Result<T, DbError>. This
matters. An earlier version returned a bare ok_serialized() on success and
a serialized DbError on failure, and the client told them apart by
checking whether the payload was empty — which was wrong, because both are
non-empty. If you add a request, its handler must return
Result<ItsPayloadType, DbError>.

## server_core — 2026-10-15

ServerState holds only the db connection (LiveForever), behind
Arc<std::sync::Mutex<ServerState>>. ConnectedClients no longer lives here;
it has its own crate and its own handle. That fixes the lock order for
good: the db lock may be held while calling ConnectedClients, never the
reverse, because the connected_clients crate doesn't depend on db_core.

Every client request goes socket -> queue -> executor -> handler -> reply.
The socket parses the websocket message, submits a Job to the QueueManager,
and awaits the reply. One executor task, started from main, takes jobs
(important channel first, then normal, via a biased select), calls
do_request, and sends the result back through the job's oneshot.

do_request is async. Its signature is (request, payload, state, clients,
sender) -> HandlerOutcome. HandlerOutcome { bytes, just_identified,
incoming_letters }. Register, GetAllYrsPages and Ack lock the db briefly,
do their work, and drop the guard; Login and SendLoveLetter do their own
locking because they need to await the connections crate in between.

The login gate lives in do_request: a socket that hasn't logged in can
only call Register or Login, everything else gets an error. The gate works
because a Result<T, DbError>::Err serializes to the same bytes no matter
what T is, so the gate doesn't need to know each handler's return type.

The executor also drains a log channel: anything that wants to write a log
row sends a ServerLogEntry down it, and the executor is the one that
actually inserts, because it's the only thing that owns the db. log_error
(in logging.rs) prints and sends an entry. The session id is read at
startup by setup::open_and_prepare and installed into logging.rs as a
global, so every row from this run carries it.

main.rs is wiring only. setup.rs opens the db and creates every table.
socket.rs holds handle_socket. queue_executor.rs holds the executor loop.
lib.rs re-exports all three.

The socket loop in socket.rs selects on two things: the websocket and the
socket's incoming_letters channel. A letter id arriving means another
client edited a page; the socket loads the letter, wraps it in a
LoveLetterResponse (which carries the letter id), and writes it. Register
and Login are different messages on purpose: Register creates the account
and fails if the name is taken, Login identifies the socket and fails if
the name doesn't exist. Only Login makes a socket receive pushes. After
Login, the executor returns the receiver half of that user's channel in
HandlerOutcome.incoming_letters, and the socket keeps it.

## connected_clients — 2026-10-15

Owns the user_name -> UnboundedSender<i64> map. The i64 is a letter id.
The map is private to the crate; every method goes through a Command sent
to one task that owns the map. The task runs on tokio and drains one
channel.

ConnectedClients is Clone. add_client is sync on purpose: login has to
call it inside the same db-lock critical section that runs the backfill,
so a new letter can't overtake the backlog. send_to, is_connected,
connected_user_names and fan_out are async and await a oneshot reply.
remove_client is fire-and-forget.

fan_out(letter_id, except) -> FanOutReport { reached, dropped } sends the
letter id to every connected user except `except`, records who was
reached, drops dead channels from the map, and reports who was dropped.
The caller then logs the drops. This is the one place the crate touches
something outside its own state: it sends a ServerLogEntry down a log
channel it was given at construction. That channel is how it reports
without depending on db_core.

## Delivery — 2026-10-15

Event-driven, no polling, at-least-once.

On a new letter: deserialize, apply_letter to update the pages table,
insert the letter (this assigns the letter id), read the registered user
list, then for every registered user except the sender — queue an unsent
row for that user, online or not. Then fan_out the letter id to every
connected user except the sender. Users the fan-out reached get the letter
pushed immediately; users who were offline, or whose channel turned out to
be dead, are reached later by the backfill. Nobody is delivered twice by
the server: the row is the record that they still owe an ack.

On Ack: the client sends the letter id it applied. The server deletes that
user's unsent row for that letter. A repeated ack for an already deleted
row is a no-op.

On Login: add_client registers the socket's channel, then the backfill
reads that user's unsent rows and pushes them, oldest first. The rows stay
until the client acks, so if a push fails or the socket dies mid-backfill,
the row is still there for next login.

The client is expected to dedup by letter_id, because a duplicate is
possible: the server might push a letter, the connection drops before the
ack, and on reconnect the row is still there and gets pushed again.

The push wire format is a Response with message_id 0, request
SendLoveLetter, and payload = bincode of LoveLetterResponse { love_letter,
letter_id }.

## Gotchas — 2026-10-15

Cargo needs the package name on the left of a dependency line, not the lib
name. yrs's package name is "yrs" but its lib name is "my_yrs_lib", so
Cargo.toml says yrs.path = "..." and the code says use my_yrs_lib::.

The db crate is named db_core (package) even though its directory is db/,
because z_db also has a crate named db and the lockfile can't hold two.

The db file path is relative to the directory the server is launched from
(personal_db_wrapper uses "test_db.sqlite"). Do not change the launch
directory, or the server silently opens a fresh empty database.

Never hold the db lock across an .await. If axum complains that the
handle_socket future isn't Send, that is the cause. A socket must also
never hold the db lock while awaiting its queue reply, because the
executor needs that same lock to produce the reply and it would deadlock.

local_cargo.sh is a per-directory wrapper around cargo. `c c` checks the
whole workspace. `c t` runs `cargo test --workspace`. `c r` runs
zakken_server from the zakken_server root, cli_gui from mock_client.

yrs's anti_deadlock prints "DEBUG_MODE is OFF (direct call, no timeout)" on
every method call. That's from the yrs crate, not from the server.

## What's left — 2026-10-15

Cross-client delivery has never been exercised. Only one mock client
exists, and it deliberately ignores pushes, so it never acks. The ack path
— the client applying the letter and sending Ack back, the server deleting
the unsent row — has never run end to end. Two mock clients with different
user names, one of them listening for pushes and sending acks, is the test.

The mock client always creates pages with is_main_menu_page = false. The
letter field exists and is threaded correctly; the mock client just doesn't
have a rule for which page should be the menu.

Retention — deleting a letter once every registered user has acked it — is
deferred. A letter with no unsent rows is harmless in the table. Note that
"no unsent rows" now means "everyone has acked", which is different from
the old meaning.

Migrations. schema/migration/ is empty on purpose. See db_core above.