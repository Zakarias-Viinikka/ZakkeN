# How a client talks to the server

The client has to first register with the server before doing anything. If the username is taken it will return an error. After that the client has to log in to connect to the server.

Relevant files found at: `server_side/server_core/src/allowed_requests/register.rs` and `server_side/server_core/src/allowed_requests/login.rs`

The server will spontaneously send the unsent love letters once logged in, one message per letter.

The client starts disconnected. Until it has asked for all pages, anything the server sends is ignored.

The server in general expects the client to send an "I have received love letter with x id" whenever it receives a love letter. Until then the server will keep sending that love letter every time it logs in.

It's also the client's responsibility to keep track of duplicate love letters it has received from the server.

## How to talk to the server — the client's own internals

The client builds a request out of four things. The message id, which is the client's responsibility — a counter that starts at 1 and increases by one for each request. The request variant, which comes from `zakken_server/shared/server_protocol/src/request.rs`. The drift version the client thinks the server is on, which the server compares against its current one on every request except Register, Login, GetCurrentDrift and its own internal pulls. And the payload, which also lives in `request.rs` and is serialized via `.to_payload()`, which comes from `protocol::serialization` in z_db.

What replies deserialize into can be found at `zakken_server/shared/server_protocol/src/responses.rs`.

The client doesn't touch the socket itself. It calls `send_request(request, payload)` on the `RouterHandle` from `shared_internals` and awaits the payload that comes back. `Router::new(url, tx_forward_spontaneous_server_message, on_conn_status)` creates the socket connection and returns that handle. The caller passes a channel sender that every spontaneous server push goes into; an owner on the other side of that channel decides what each push means.

Spontaneous messages from the server are a separate thing on the wire — `Response::LoveLetterFromServer`, which carries one letter, the drift version the server is on, and no message id, and `Response::TheDriftVersionChanged`, which carries the new drift version + reason (sent just before the server closes this socket because its dataset was replaced). The router doesn't know what to do with them, so it forwards every spontaneous response into a channel the app supplies when it builds the router. The app matches on the variant there.

## Reconcile — the client's other three asks

Reconcile is the flow a lonewolf client runs to merge its pages back
into the server's dataset. Four requests, all exempt from the drift
check:

`GetPageFingerprints` takes no payload. Replies with the current
`drift_version` and, for every page the server has, a `page_id` plus a
`fingerprint` string. The fingerprint is SHA-256 of the page's raw
`blobbed_page` bytes, computed by the same shared helper the client
uses (`drift_guard_shared::fingerprint`). Same input, same output on
both sides.

`GetPagesByIds` takes `{ page_ids: Vec<String> }`. Replies with the
full data of each requested page (same `PageRow` shape as
`GetAllYrsPages`). Unknown ids are skipped silently.

`GetReconcileVersion` takes no payload. Replies with
`{ reconcile_version: String }`, the current monotonic counter.

`ReconcileLetters` takes `{ target_drift_version, target_reconcile_version,
letters: Vec<Vec<u8>> }` where each `letters` entry is a serialised
`LoveLetter` encoded the same way `SendLoveLetterPayload::love_letter`
is. The handler decodes every letter first (any failure rejects the
whole batch), opens one transaction, checks both target versions
against the current rows, applies every letter in order, bumps
`reconcile_version` inside the same transaction, commits, then notifies
every connected socket except the sender's once. Any failure rolls the
whole batch back. Success returns `Ok(())`.

`reconcile_version` is drift_version for reconciles, kept separate so
two clients can't both succeed off the same snapshot. Which batch
lands first bumps it; the second is refused with
`DriftVersionMismatch`, its client reloads and starts over. It does
not bump `drift_version` — other connected clients are not informed or
disconnected. Each lonewolf client reconciles on its own.

## Export — the server builds, the client downloads

`Export` is a new request, no payload yet. The client sends it and the
server builds the export archive itself. The server has OS file access
and the pages table already, so the client doesn't need to hold the
whole dataset or a zip in memory.

Server side: `allowed_requests/exp_and_import.rs::export_all_pages`.
It clears and recreates `export_temp/` (relative to the server's launch
dir, same as `zakken.sqlite`), reads every page id, writes each page as
`export_temp/<page_id>.json` — a JSON array of blocks, each
`{ content, metadata, frac_index }` — and the final step zips the folder
and returns the bytes.

Not built yet: unpacking `blobbed_page` into blocks
(`get_blocks_for_page`), and the zip step. Both are `todo!()`.

Reply shape for `Export` is not decided yet (raw zip bytes in the
reply, or a follow-up fetch). When it lands, that paragraph goes here.
