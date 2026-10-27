# client_internals

How a client talks to `zakken_server`. Three folders:

- `shared_internals_for_server` — target-agnostic. The response router:
  every request gets a fresh id, and the reply to that id goes back to
  whoever is waiting. A response whose id has no waiter (nudges are the
  case that matters, message id 0) goes on its own channel. Depends on
  `futures` + `server_protocol` + `error_stuff` only.

- `web` — the browser half. Opens a `WebSocket` via `web_sys`, writes
  outgoing messages, decodes incoming JSON, and drives the shared router.
  Modeled on
  `rust_practice/server_template_3/client_side/how_to_talk_to_server`.
  Wasm only; excluded from the workspace.

- `android` — the Android half. Same idea, over a native websocket,
  exposed across uniffi. Skeleton; the socket crate hasn't been chosen.
  Excluded from the workspace.

Why split this way: the router is the same in both, so it lives once.
The socket is different in each, so it lives twice. The socket half is
the only part that knows about a platform.
