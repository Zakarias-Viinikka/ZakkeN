# Server: register + love letters

Wednesday, 14 October 2026

## What exists

zakken_server is a workspace. Members: db (crate name db_core), server_core, error_stuff, server_protocol, love_letter_parser, mock_client/mock_client_core, mock_client/cli_gui.

db_core owns the schemas and the operations. Schemas live in db/src/db/schema/, one file per table with a tbl_ prefix: tbl_registered_users, tbl_love_letters_from_clients, tbl_unsent_love_letters, tbl_socket_testing_table. The pages table is not here — db_core depends on client_table_blueprints and uses its tbl_pages, so the client and server agree on that schema by construction. schema/migration/ exists but is empty; migrations go there later. Operations live beside schema, one file per concern: user_registration.rs, operations_love_letters.rs, operations_pages.rs, apply_letter.rs.

server_protocol owns the server's own Message, Response, Request and the payload structs. Not z_db's — that one belongs to the browser. Its lib.rs is just re-exports; message.rs, request.rs and builders.rs hold the actual definitions. Request has Register, Login, SendLoveLetter, GetAllYrsPages. Every response payload is bincode-encoded Result<T, DbError>, so success and failure are distinguishable on the wire.

love_letter_parser turns letter bytes into a target_page_id.

server_core is the server. ServerState holds the db connection and ConnectedClients. ConnectedClients owns the user_name -> channel-sender map and is the only thing that touches it. allowed_requests/ dispatches on Request, one file per request. logging.rs holds the log helpers.

Server startup creates every table it needs if it doesn't exist yet: socket_testing_table, registered_users, love_letters_from_clients, unsent_love_letters, pages.

## What happens when

A client opens a socket and sends Register (creates an account, refuses if the name is taken) or Login (identifies this socket as that user). Login is what makes the socket receive anything; before login it can only register or log in.

After Login succeeds, the socket gets a channel pair. Its sender half goes into ConnectedClients; its receiver half stays with the socket's task, which now listens to both the websocket and that channel.

Delivery has two triggers, both event-driven. Nothing polls.

On a new letter: the server deserializes it, applies it to its pages table (update the page's blob, merge, save), stores the letter in love_letters_from_clients, then walks every registered user. The sender is skipped. For each other user: online, push the letter id down their channel; the socket loop loads the letter and writes it. Offline, insert an unsent row for them. That's the whole trigger.

On Login: backfill reads every unsent row for that user, pushes each letter id to the just-created channel, and deletes the rows. So offline users get their backlog in order, the moment they log in.

## What's left

Cross-client delivery has never been exercised. Only one mock client exists and it deliberately ignores pushes (expect_response skips anything whose id it isn't waiting for). Two mock clients with different user names, one of them listening for pushes, is the test. Until then, "user A edits, user B receives" is written but unproven — and specifically the merge-into-page-blob path is the one I'd want to see before trusting it.

The mock client always creates pages with is_main_menu_page = false, so a main-menu page can never come from it. The letter field exists and is threaded correctly; the mock client just doesn't have a rule for which page should be the menu.

plan.md itself was stale until this rewrite.

## Deferred

Retention — deleting a letter once every registered user has received it. A letter with no unsent rows is harmless sitting in the table; cleanup can wait.

Migrations. schema/migration/ is empty on purpose. Every table is created by create_table_if_not_exist on startup, which is fine while the schema is frozen and won't be fine the first time a column changes.