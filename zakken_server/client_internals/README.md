# client_internals

How a client talks to `zakken_server`. One crate:

- `shared_internals_for_server` — everything the client needs to hold
  a connection. The response router assigns each request a fresh id
  and routes the reply back to whoever is waiting. The per-target
  socket halves live inside it as `src/socket_web.rs` and
  `src/socket_android.rs`, so `Router::new(url, on_letters,
  on_conn_status)` is the same call on both targets and only the
  `spawn` underneath differs. The web half opens a `WebSocket` via
  `web_sys`, writes outgoing messages, decodes incoming JSON, and
  retries on disconnect. The Android half is a placeholder until a
  native websocket crate is picked.

Earlier there were two extra crates, `client_web` and `client_android`,
one per target. They became empty when the socket code moved into the
shared crate so `Router::new` could stay identical for both, and were
removed.