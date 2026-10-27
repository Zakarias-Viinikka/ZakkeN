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
- error_stuff - ServErr, ErrorDetails (err_msg/file/method/probable_cause),
  and the log types LogCategory, LogLevel, ServerLogEntry.
- server_protocol - the server's own Message/Response/Request and payload
  structs. Not z_db's protocol; that one belongs to the browser.

server_side/ holds the server's own crates:
- db (package name db_core) - schemas and db operations, nothing else.
- server_core - the server binary, the request handlers, the queue
  executor, and the telephone module.
- love_letter_parser - letter bytes -> target_page_id.
- queue - Job, Priority, QueueManager. No database knowledge.

mock_client/ holds the mock client, at the top level:
- mock_client_core - mock client logic, no printing.
- cli_gui - the mock client's REPL.

## db_core - 2026-10-15

Schemas in db/src/db/schema/, one file per table, tbl_ prefix:
tbl_registered_users, tbl_love_letters_from_clients,
tbl_unsent_love_letters, tbl_socket_testing_table. There is no tbl_pages
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

## server_protocol - 2026-10-22

The server's own Message, Response, Request and payload structs. Its
lib.rs is pure re-exports; message.rs, request.rs and builders.rs hold the
definitions.

Request variants: Register, Login, SendLoveLetter, GetAllYrsPages, Ack,
GetMyUnsentLetters, Nudge. Ack carries an AckPayload { letter_id: i64 } -
the client saying it has applied that letter, so the server can delete
that user's unsent row. GetMyUnsentLetters carries no payload and returns
a GetMyUnsentLettersPayload { letters: Vec<UnsentLetter> }, where each
UnsentLetter is { letter_id: i64, love_letter: Vec<u8> }. Nudge is
server-only: it arrives as a Response with message_id 0, request = Nudge,
empty payload. The server refuses to run a Nudge if one shows up as a
job.

Every response payload is a bincode-encoded Result<T, DbError>. This
matters. An earlier version returned a bare ok_serialized() on success and
a serialized DbError on failure, and the client told them apart by
checking whether the payload was empty - which was wrong, because both are
non-empty. If you add a request, its handler must return
Result<ItsPayloadType, DbError>.

## server_core - 2026-10-27

ServerState holds only the db connection (LiveForever), behind
Arc<std::sync::Mutex<ServerState>>. The Telephone lives in this crate too,
as the `telephone` module, but behind its own handle. The db lock may be
held while calling Telephone, never the reverse: the telephone's task
never touches the db.

Every client request goes socket -> queue -> executor -> handler -> reply.
The socket parses the websocket message, submits a Job to the QueueManager,
and awaits the reply. One executor task, started from main, takes jobs
(important channel first, then normal, via a biased select), calls
do_request, and sends the result back through the job's oneshot.

A Job carries the request, the payload, a message id, and a socket id. The
socket id is a u64 minted from an atomic counter when the socket is born.
It names a connection before anyone has logged in, which is what lets the
socket register its nudge channel at birth instead of at login.

do_request is async. Its signature is (request, payload, state, telephone,
socket_id) -> Vec<u8>. Every handler returns bytes and nothing else. The
login gate lives here: do_request asks the telephone for the name attached
to the socket id, and a socket with no name may only call Register or
Login. Register, GetAllYrsPages, Ack and GetMyUnsentLetters lock the db
briefly, do their work, and drop the guard; Login and SendLoveLetter do
their own locking because they need to await the Telephone in between db
calls. Nudge is refused as a job because it's server-only.

Login does two things beyond checking the user exists: it tells the
telephone to attach the name to the socket id, then nudges that socket so
the client pulls its backlog.

The executor also drains a log channel: anything that wants to write a log
row sends a ServerLogEntry down it, and the executor is the one that
actually inserts, because it's the only thing that owns the db. log_error
(in logging.rs) prints and sends an entry. The session id is read at
startup by setup::setup_db and installed into logging.rs as a global, so
every row from this run carries it.

main.rs is wiring only. setup.rs opens the db and creates every table.
socket.rs holds socket_main, the per-connection entry point.
queue_executor.rs holds the executor loop. lib.rs re-exports all three.

The socket in socket.rs is a `SocketSession`: the websocket, the socket
id, a log label, and the nudge receiver. Its loop selects on two things:
the websocket and its nudge channel. The nudge channel is created at birth
and handed to the telephone at birth, so the socket listens from the
moment it exists - login changes nothing about the channel, it only gives
the telephone a name to reach it by. A () arriving on the channel means
the telephone wants this client to pull; the socket writes a Nudge down
the websocket (a Response with message_id 0, request Nudge, empty
payload), and the client's move is to respond with GetMyUnsentLetters,
apply each letter, and Ack each. Nudges carry no letter id; the fetch is
the client's side.

After a Login reply comes back, the socket asks the telephone who it is
(user_of(socket_id)) and uses that name as its log label. If the login
failed there is no name and the label stays the address. When the loop
ends, the socket calls telephone.remove_socket(socket_id).

## telephone - 2026-10-27

A module inside server_core (src/telephone/), not a crate any more. It
used to be the connected_clients crate; it moved because the socket id
design ties the telephone and the socket loop into one story.

Exposes one struct, `Telephone`. It's a handle to a task that owns a
HashMap<u64, SocketEntry> - socket id -> { nudge sender, user name }. The
() on the channel is a nudge: "you have something to pull". Every method
sends a Command to the task; the task is the only thing that touches the
map, and it runs on tokio, draining one channel.

Telephone is Clone. register_socket and remove_socket are sync and
fire-and-forget: a socket registers at birth and removes itself at
disconnect. attach_name(socket_id, name) is async and waits until the task
has recorded the name, so the executor's login gate can rely on a
following user_of seeing it. user_of(socket_id) returns the name attached
to a socket, or None. nudge_socket(socket_id) nudges one connection;
nudge_all(except) nudges every logged-in connection except `except`.
is_connected(user_name) and connected_user_names() ask the map without
touching it; nothing uses them right now except a test.

nudge_all skips entries with no name: an entry that exists but hasn't
logged in can't have a letter queued for it, so there is nothing for it to
pull. That, plus the one-job-at-a-time executor, is why a live letter
can't fan out to a user between attach_name and the login nudge.

When a client's channel turns out to be gone (their socket task dropped
the receiver), nudge_all drops that client from the map and reports it in
the returned NudgeReport { reached, dropped }. The caller logs the drops.
The task also sends a ServerLogEntry down the log channel it was given at
construction; that's how this module reports without owning the db.

## Delivery - 2026-10-27

Event-driven, no polling, at-least-once. Pull model on the client side:
the server nudges, the client fetches.

On a new letter: deserialize, apply_letter to update the pages table,
insert the letter (this assigns the letter id), read the registered user
list, then for every registered user except the sender - queue an unsent
row for that user, online or not. Then nudge_all(sender). Every connected
client except the sender gets a Nudge down its socket. Nudged clients
send GetMyUnsentLetters, receive every letter that's still rowed for them,
apply each, and Ack each. Offline clients aren't connected, so nudge_all
doesn't reach them; their unsent rows sit until they log in.

On Login: the socket registered its channel at birth, so login only
attaches the name to the socket id and then nudges that socket once. The
socket writes a Nudge and the client pulls its backlog through
GetMyUnsentLetters. The rows stay until each is acked.

On Ack: the client sends the letter id it applied. The server deletes
that user's unsent row for that letter. A repeated ack for an already
deleted row is a no-op.

The client is expected to dedup by letter_id, because a duplicate is
possible: the server nudged, the connection dropped before the ack, and
on reconnect the row is still there and shows up in the next pull.

The nudge wire format is a Response with message_id 0, request = Nudge,
empty payload. The pull wire format is a Message with request =
GetMyUnsentLetters and no payload; the reply carries a bincode of
Result<GetMyUnsentLettersPayload, DbError>.

## Gotchas - 2026-10-22

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
(The socket task itself no longer touches the db at all; the rule still
matters for login and send_love_letter, which lock briefly and drop
before awaiting.)

local_cargo.sh is a per-directory wrapper around cargo. `c c` checks the
whole workspace. `c t` runs `cargo test --workspace`. `c r` runs
zakken_server from the zakken_server root, cli_gui from mock_client.

yrs's anti_deadlock prints "DEBUG_MODE is OFF (direct call, no timeout)" on
every method call. That's from the yrs crate, not from the server.

## What's left - 2026-10-22

Cross-client delivery has still never been exercised end to end. The mock
client now knows how to receive: it handles a Nudge by pulling
GetMyUnsentLetters, applying each letter to its local boss (or
page_status), and Acking each. The same path is exposed as the `sync`
REPL command, so you can force a pull without waiting for a nudge. Two
mock clients with different user names, one editing while the other
syncs, is still the test that proves "user A edits, user B receives" -
it just hasn't been run.

The mock client always creates pages with is_main_menu_page = false. The
letter field exists and is threaded correctly; the mock client just doesn't
have a rule for which page should be the menu.

Retention - deleting a letter once every registered user has acked it - is
deferred. A letter with no unsent rows is harmless in the table. Note that
"no unsent rows" now means "everyone has acked", which is different from
the old meaning.

Migrations. schema/migration/ is empty on purpose. See db_core above.