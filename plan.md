# Server: register + love letters

Updated 28 October 2026 (first written 14 October 2026)

## What exists

zakken_server is a workspace in four folders. `shared/` holds
error_stuff and server_protocol. `server_side/` holds db (package name
db_core), server_core, love_letter_parser and queue. `mock_client/`
holds mock_client_core and cli_gui. `client_internals/` holds
shared_internals_for_server.

db_core owns the schemas and the db operations, nothing else. Schemas
live in db/src/db/schema/, one file per table with a tbl_ prefix:
tbl_registered_users, tbl_love_letters_from_clients,
tbl_unsent_love_letters, tbl_socket_testing_table. The pages, logs and
key_value_storage tables are not here - db_core depends on
client_table_blueprints and uses its tbl_pages, tbl_logs and
tbl_key_value_storage, so the client and server agree on those schemas
by construction. schema/migration/ exists but is empty; migrations go
there later. Operations live beside schema, one file per concern:
user_registration.rs, operations_love_letters.rs, operations_pages.rs,
operations_logs.rs, operations_key_value.rs, apply_letter.rs.

apply_letter.rs is the server's equivalent of cqrs's executor. It's
small because the server's job is small: it receives a snapshot, not a
character diff. CreateNewPage inserts a row, and merges instead if the
page already exists; everything else loads the page's blob, merges the
snapshot in via BossOfYrs::merge_with_snapshot, and saves. DisablePage
does the same against the page_status blob via YrsActivePages. It
deliberately doesn't call cqrs's builder.

server_protocol owns the server's own Message, Response, Request and
the payload structs. Not z_db's - that one belongs to the browser. Its
lib.rs is just re-exports; message.rs, request.rs and builders.rs hold
the actual definitions. Request has Register, Login, SendLoveLetter,
GetAllYrsPages, Ack, GetMyUnsentLetters. Response is either a Reply
carrying message_id, request and payload, or LoveLetterFromServer
carrying one letter and no id, because the client acks it individually.
Every reply payload is bincode-encoded Result<T, DbError>, so success
and failure are distinguishable on the wire. Frames on the socket are
JSON; builders.rs decodes them.

queue - Job, Priority, QueueManager. No db knowledge. Every request
goes through it, carrying the socket id. The socket's internal "go
pull" is submitted the same way, with message_id 0.

telephone - a module inside server_core now, not a crate. Owns the
socket id -> { notify sender, user name } map. The channel carries ()
and means "go pull"; the letters stay in the db. No db knowledge, so it
can never take the db lock; that fixes the lock order at
db-then-telephone forever. Every method talks to one task that owns the
map.

server_core - the server. The shared state is Arc<Mutex<LiveForever>>,
passed straight to the queue executor. main.rs is wiring: it opens the
log channel, builds the telephone, makes the two queue channels,
starts the executor, and serves the axum route /ws on port 3000.
setup.rs opens the db, creates every table it needs if missing, and
bumps the server run's session id. allowed_requests/ dispatches on
Request, one file per request. logging.rs holds the log helpers; the
executor drains the log channel and writes the rows.

Startup creates: socket_testing_table (and seeds it), registered_users,
love_letters_from_clients, unsent_love_letters, pages, logs,
key_value_storage.

## What happens when

A client opens a socket and sends Register (creates an account, refuses
if the name is taken) or Login (identifies this socket as that user).
The socket registers its notify channel at birth, so Login only has to
attach the name to the socket id and notify that socket once. Before
login it can only register or log in; do_request refuses everything
else.

Delivery has two triggers, both event-driven. Nothing polls.

On a new letter: the server deserializes it, applies it to its pages
table, stores it in love_letters_from_clients, then walks every
registered user. The sender is skipped. Every other user gets an unsent
row, whether they're online or not. Then it notifies every connected
user except the sender. A notified socket submits GetMyUnsentLetters for
its own user through the queue, gets back every letter still rowed for
it, and pushes each one to the client as its own LoveLetterFromServer
message, so each is applied and acked on its own.

On Login: the same notify fires once, so the socket pulls its backlog
the moment it logs in. The rows stay until acked, so a dropped
connection gets the letter again on reconnect, and the client dedups by
letter_id.

On Ack: the server deletes that user's unsent row for that letter. That
is the only thing that removes a row.

## What's left

Cross-client delivery works, and it's been observed end to end. On
2026-10-28 a web-client edit reached the mock client via sync, and a
mock-client edit reached the web client via the server's push and was
acked. The web client is a real second client now: the socket and
router live in client_internals/shared_internals_for_server, and the
connection lifecycle in
cqrs_state_synchronization/web_interface/src/server_link/.

The mock client applies and acks pushes that arrive while it is waiting
for a reply, and creates pages with is_main_menu_page = true, so the web
menu shows pages it makes.

## Next

Dataset generation and sync recovery. The server owns a
dataset_generation value in key_value_storage, minted once when the db
is fresh and changed only when the dataset is deliberately replaced.
Login would carry it, and the client would refuse to sync against a
different one. On top of that: export everything to JSON, load from
JSON and reset the server, wipe locally, and let a client tell the
server to wipe itself and adopt the client's dataset. Web first;
Android deferred but kept compatible. The plan is in
artifacts/plan_for_general_data_management_ops.md, and it depends on a
yrs hard-copy primitive (BossOfYrs::from_hard_copy) that isn't built
yet. Not started.

The NoSuchUser clearing of the client's seen-letter set is a stopgap
for "server db wiped" until generation ids land.

## Deferred

Retention - deleting a letter once every registered user has acked it.
A letter with no unsent rows is harmless sitting in the table, so
cleanup can wait.

Migrations. schema/migration/ is empty on purpose. Every table is
created by create_table_if_not_exist on startup, which is fine while
the schema is frozen and won't be the first time a column changes.

Identity. "Same name = same user" with no auth: a second device using
the same name takes over the first one's socket. Fine for the local
network this runs on, not a design that survives anything wider.