# How a client talks to the server

The client has to first register with the server before doing anything. If the username is taken it will return an error. After that the client has to log in to connect to the server.

Relevant files found at: `server_side/server_core/src/allowed_requests/register.rs` and `server_side/server_core/src/allowed_requests/login.rs`

The server will spontaneously send the unsent love letters once logged in, one message per letter.

The client starts disconnected. Until it has asked for all pages, anything the server sends is ignored.

The server in general expects the client to send an "I have received love letter with x id" whenever it receives a love letter. Until then the server will keep sending that love letter every time it logs in.

It's also the client's responsibility to keep track of duplicate love letters it has received from the server.

## How to talk to the server — the client's own internals

The client builds a request out of three things. The message id, which is the client's responsibility — a counter that starts at 1 and increases by one for each request. The request variant, which comes from `zakken_server/shared/server_protocol/src/request.rs`. And the payload, which also lives in `request.rs` and is serialized via `.to_payload()`, which comes from `protocol::serialization` in z_db.

What replies deserialize into can be found at `zakken_server/shared/server_protocol/src/responses.rs`.

The client doesn't touch the socket itself. It calls `send_request(request, payload)` on the `RouterHandle` from `shared_internals_for_server` and awaits the payload that comes back. `Router::new(url, on_letters, on_conn_status)` creates the socket connection and returns that handle.

Spontaneous messages from the server are a separate thing on the wire — `Response::LoveLetterFromServer`, which carries one letter and no message id. The router doesn't know what to do with them, so it hands each one to a callback the app supplies when it builds the router.
