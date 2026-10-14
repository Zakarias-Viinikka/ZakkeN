# zakken_server — AI stored context

Written by AI for AI. Read this before touching the server. Update the
relevant section when its content changes and bump that section's date. If
code and this file disagree, the code wins and this file should be
corrected.

The project-wide picture lives one level up, in
ZakkeN/STORED_AI_CONTEXT.md. This file is about the server only.

## What this is — 2026-10-14

The axum websocket server for ZakkeN. Clients describe edits as "love
letters" — a serialized intent. The server applies each letter to its own
copy of the pages, stores the letter, and fans it out to the other
registered clients. It's the only place that knows which clients are
online and which clients still need which letters.

## Workspace layout — 2026-10-14

A cargo workspace. Members:

- db (package name db_core) — schemas and db operations, nothing else.
- server_core — the server binary and the request handlers.
- error_stuff — ServErr, the workspace error enum.
- server_protocol — the server's own Message/Response/Request. Not
  z_db's protocol; that one belongs to the browser.
- love_letter_parser — letter bytes -> target_page_id.
- mock_client/mock_client_core — mock client logic, no printing.
- mock_client/cli_gui — the mock client's REPL.

## db_core — 2026-10-14

Schemas in db/src/db/schema/, one file per table, tbl_ prefix:
tbl_registered_users, tbl_love_letters_from_clients,
tbl_unsent_love_letters, tbl_socket_testing_table. There is no tbl_pages
here — db_core depends on client_table_blueprints and uses its tbl_pages,
so the client and server share one definition of that table.

schema/migration/ exists and is empty on purpose. Every table is created by
a create_table_if_not_exist function at server startup, which is fine while
the schema is frozen and won't be the first time a column changes.

Operations live beside schema:

- user_registration.rs — create_table_if_not_exist, user_exists,
  create_user. create_user errors if the name is already taken.
- operations_love_letters.rs — the two letter tables. create_tables_if_not_exist,
  insert_letter, get_letter, get_all_registered_user_names,
  get_unsent_letter_ids_for_user, user_has_unsent_row, queue_unsent_for_user,
  delete_unsent_row.
- operations_pages.rs — the pages table. create_table_if_not_exist,
  page_exists, get_page, get_all_pages, insert_page, update_page_blob,
  update_page_status.
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

## server_protocol — 2026-10-14

The server's own Message, Response, Request and payload structs. Its
lib.rs is pure re-exports; message.rs, request.rs and builders.rs hold the
definitions. Same shape as rust_practice/server_template_3.

Request variants: Register, Login, SendLoveLetter, GetAllYrsPages.

Every response payload is a bincode-encoded Result<T, DbError>. This
matters. An earlier version returned a bare ok_serialized() on success and
a serialized DbError on failure, and the client told them apart by
checking whether the payload was empty — which was wrong, because both are
non-empty. If you add a request, its handler must return
Result<ItsPayloadType, DbError>.

## server_core — 2026-10-14

ServerState holds the db connection (LiveForever) and ConnectedClients
together behind one Mutex, because every handler that touches the map also
touches the db.

ConnectedClients owns the user_name -> UnboundedSender<i64> map. The i64 is
a letter id. The struct is the only thing that touches the map. Its send_to
returns ServErr::ClientGone if the receiver was dropped, and the caller
removes that user.

allowed_requests/ has one file per request. mod.rs holds do_request, which
dispatches and also holds the login gate: a socket that hasn't logged in
can only call Register or Login, everything else gets an error. The gate
lives here, not in the handlers, because a Result<T, DbError>::Err
serializes to the same bytes no matter what T is.

logging.rs holds the log helpers. Its style is deliberately plain English
("updated its copy of page X") rather than dumping JSON, because the raw
dumps were unreadable.

The socket loop in main.rs selects on two things: the websocket and the
socket's own push channel. A push carries a letter id; the loop loads the
letter and writes it. Register and Login are different messages on purpose:
Register creates the account and fails if the name is taken, Login
identifies the socket and fails if the name doesn't exist. Only Login makes
a socket receive pushes. After Login, the socket gets a channel pair, its
sender half goes into ConnectedClients, its receiver half stays with the
socket task.

## Delivery — 2026-10-14

Event-driven, no polling.

On a new letter: deserialize, apply_letter to update the pages table,
insert the letter, then for every registered user except the sender — if
online, push the letter id down their channel; if offline, insert an unsent
row. The sender is skipped because it already applied the edit locally.

On Login: backfill reads that user's unsent rows, pushes them, deletes
them. So an offline user's backlog arrives in order the moment they log in.

## Gotchas — 2026-10-14

Cargo needs the package name on the left of a dependency line, not the lib
name. yrs's package name is "yrs" but its lib name is "my_yrs_lib", so
Cargo.toml says yrs.path = "..." and the code says use my_yrs_lib::.

The db crate is named db_core (package) even though its directory is db/,
because z_db also has a crate named db and the lockfile can't hold two.

local_cargo.sh is a per-directory script the user's shell function `c`
finds by walking up from cwd. It sets RUSTFLAGS for mold and forwards to
cargo. zakken_server/local_cargo.sh runs zakken_server;
mock_client/local_cargo.sh runs cli_gui. If a directory has no
local_cargo.sh, `c r` finds the nearest one above it, which is often the
wrong thing.

yrs's anti_deadlock prints "DEBUG_MODE is OFF (direct call, no timeout)" on
every method call. That's from the yrs crate, not from the server.

## What's left — 2026-10-14

Cross-client delivery has never been exercised. Only one mock client
exists, and it deliberately ignores pushes. Two mock clients with different
user names, one of them listening for pushes, is the test.

The mock client always creates pages with is_main_menu_page = false. The
letter field exists and is threaded correctly; the mock client just doesn't
have a rule for which page should be the menu.

Retention — deleting a letter once every registered user has received it —
is deferred. A letter with no unsent rows is harmless in the table.

Migrations. schema/migration/ is empty on purpose. See db_core above.
