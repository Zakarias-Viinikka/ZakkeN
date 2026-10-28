# Decisions, 2026-10-28

Things decided in conversation and written down here because they don't
belong in any other doc yet. When a decision is absorbed by a code
change or a plan, delete its line.

## Docs and file layout

- Plans go at the repo root, not in `artifacts/`. `artifacts/` holds
  historical snapshots (`oct_27_plan.md`) and nothing current.
- `plan.md` at the root is the server's current state and is kept
  honest. If it drifts, that's a bug.
- `guide_to_interacting_with_server.md` is written from the app's point
  of view. It describes what a client has to do; how messages travel is
  out of scope.

## Renames

- Renaming a Rust identifier means `astgrep rename`, not a chain of
  replaces. Only fall back to replace when astgrep can't do it (string
  literal, non-.rs file, non-identifier, keyword). One astgrep step per
  chain.

## Server protocol

- `request.rs` holds only request variants and request-only payloads.
  Reply-only payloads live in `responses.rs`. `lib.rs` re-exports
  everything, so call sites don't care.
- The server sends unsent letters on login. It has no notion of a
  client being "subscribed" - that's a client concern.
- The guide says: "The client starts disconnected. Until it has asked
  for all pages, anything the server sends is ignored." The gate is
  client-side.

## Client sync

- `start_over_fresh` (renamed from `bootstrap`) refuses to run if the
  local db already has pages. `GetAllYrsPages` returns the server's
  full set; merging that into a non-empty local db is a union across
  divergent lineages. Anything with local pages goes through
  reconcile/adopt.
- Login is re-run on every reconnect, not just the first connect. Each
  new server-side socket starts with no user attached, so every request
  would fail `not logged in` until a page reload.
- On socket death, the router drops every pending waiter so in-flight
  callers error out instead of hanging.

## Accepted as-is

- `create_new_page` uses two transactions (page insert, then title
  write). Not a bug. Don't refactor.

## Removed

- `client_internals/web` and `client_internals/android` are gone. The
  per-target socket halves live inside `shared_internals_for_server` as
  `socket_web.rs` / `socket_android.rs`, so `Router::new` is identical
  on both targets.
