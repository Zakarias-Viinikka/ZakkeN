# Web client ← server link: implementation plan (2026-10-27)

Goal: the web app talks to `zakken_server`. Two clients, one edits,
the other sees it. This plan covers the whole path: socket, identity,
merge primitive, receive worker, UI refresh, sending.

Built in order. Each phase compiles and is testable before the next
starts.

## Context: what already exists

- `zakken_server` works end to end for the shapes that exist. It's the
  only thing that applies letters and rows unsent letters per user.
- `client_internals/shared_internals_for_server` has the response
  router. Target-agnostic. The socket half is a placeholder in
  `client_internals/web`.
- `web_interface` (cqrs) has pages, blocks, editing, dragging,
  cross-page moves, the batcher, the local db. No networking:
  `load_from_server` is a no-op.
- `mock_client_core/src/client.rs` `apply_letter` is the reference for
  what "apply a letter" means. It mutates in-memory maps; the web side
  has to persist.

## Context: what the server does

From `guide_to_interacting_with_server.md`:

- On login the server notifies the socket. The socket pulls its
  backlog and pushes each letter as `LoveLetterFromServer`.
- On a new letter the server notifies every connected client except
  the sender.
- The client acks each letter by id. Unacked letters come back on
  reconnect.
- The client dedups by letter id.

## The build blocker

`server_side/server_core` and `cqrs_state_synchronization/error_stuff`
both have package name `error_stuff`. Cargo can't put both in one
lockfile. Fix: rename the server's package to `server_error_stuff` and
keep the lib name `error_stuff` inside it, so source code doesn't
change. Dependents add `package = "server_error_stuff"` to the
dependency line.

Consequence: `CqrsErr` and `ServErr` are two different types in the
same binary. Conversions between them, where needed, are explicit.

## Phase 0: dependency spike (do first)

1. Rename the server's `error_stuff` package as above.
2. Add as path dependencies of `web_interface`:
   - `love_letter` (ZakkeN/love_letter)
   - `server_protocol` (zakken_server/shared/server_protocol)
   - `shared_internals_for_server`
     (zakken_server/client_internals/shared_internals_for_server)
   - `client_web` (zakken_server/client_internals/web)
   - `futures` 0.3, if not already pulled transitively
3. `cargo check --target wasm32-unknown-unknown` in
   `cqrs_state_synchronization/web_interface`.

Risks:
- Duplicate `protocol` or `yrs` from a git source. `TextEdit` types
  would mismatch.
- A server dependency without wasm support.

Nothing else starts until this passes. Then re-run `c c` in
`zakken_server` to confirm the rename didn't break the server.

## The write lock

The "20 ms poll until no transaction is open" idea has a look-then-act
hole: the check is an async call to the db worker, so other tasks can
run between the check and the begin. Two writers can both see "clear"
and both start.

Fix: `WriteLock` with a `Cell<bool>`. `try_claim()` checks and sets in
one synchronous step; `acquire().await` loops on try with a small
sleep. Returns a guard that releases on drop, so `?` and early returns
can't leak it.

Every writing entry point takes it: `add_block`, `delete_block`,
`move_block`, `move_block_to_another_page`, `disable_page`,
`create_new_page`, the title write, the batcher flush, the
received-letter apply, and the send loop's row deletes.

The executors do not take it. They keep joining an open transaction
through `start_transaction_if_not_in_one!`. Under the lock, "a
transaction is open" can only mean the current holder opened it, so
joining is safe. If the executors took the lock themselves, a flush
would deadlock on its own inner calls.

Deadlock order: entry points call `flush_now()` first, which takes and
releases the lock inside the batcher. Only then do they take the lock
for their own executor. Nothing calls `flush_now()` while holding the
lock. `add_block_unflushed` runs inside the flush, which already holds
it.

The apply path: send the batcher a message; the batcher task does
flush, then take the lock, then apply, then release.

## Phase 1: connection, border, identity

### Socket (`client_internals/web`)

- Opens `ws://<window hostname>:3000/ws`. Host from the page, so it
  works from the phone on the LAN.
- Encodes with `message_to_json_str`, decodes with
  `json_str_to_response`, feeds `RouterHandles`.
- Built on the path deps from Phase 0, so no type mismatch.

### Router

Add `fail_all_pending()` to `ResponseRouter`. Without it, a dropped
connection leaves every waiting caller hanging, because the app still
holds router clones.

### Connection state

`ConnState`: `Offline | Connecting | FailedToConnect | Connected |
TwoTabs`.

- `Offline` from `navigator.onLine` and the `online`/`offline` events.
- `FailedToConnect`: socket failed to open while online.
- Retries: backoff 1s doubling to 30s cap; restart immediately on
  `online`.

Global `ArcRwSignal` statics, same pattern as `APP_LOCKED`.

### Border

On `MainContent`:
- Gray: `Offline`.
- Blue: `FailedToConnect`.
- Yellow: `Connecting`, applying, or unsent letters waiting.
- Green: `Connected` and caught up.
- TwoTabs: gray plus a message.

Replaces the debug text in `ConnectionIndicator`. `PageLoadState::Online`
is left for a later hash check.

### Two tabs

`navigator.locks.request(name, { ifAvailable: true }, cb)` through
`js_sys::Reflect`. The tab that doesn't get the lock skips all server
work and shows the TwoTabs message.

### Identity

`server_user_name` in `key_value_storage`.

- Missing: try `Register` with the setup name. On the "already
  registered" `IllegalInput`, try `name-1`, `name-2`, and so on. Save
  the one that worked.
- Saved but login says "no such user" (server db wiped): clear it and
  `bootstrapped`, start over.
- Link starts when `tables_ready` and `USER_ID` are both set.

## Phase 2: merge primitive, bootstrap, receive worker

### The primitive

One function: `merge_page(page_id, create_info, blob_snapshot,
status_snapshot)`. Used by both the bootstrap and the receive worker.
Doesn't touch `client_love_letter`, so received letters never echo
back.

Builder (pure): takes a boss, a snapshot, the page's existing mirror
rows, and a few flags. Merges the snapshot. Returns:
- the new blob
- a reconcile plan: insert / update content or frac / delete
- the final ordered `BlockRow` list
- a `changed` flag

`DisablePage` merges into the status doc via `YrsActivePages`.
`CreateNewPage` for an unknown page produces an insert plan.
Unit-tested on its own.

Executor: one transaction. Writes the blob and applies the reconcile
plan using the existing private helpers in `page_executor.rs`. Takes
the `WriteLock`.

`is_title` is not in yrs. The first block by frac order is the title
(matches how `create_page` makes them). `frac` from
`get_frac_position_of_block` per block; no uniffi record change.

`Convert::to_payload` is plain `bincode::serialize`, same as
`sketch_to_bytes`. The sender can read `client_love_letter` rows back
with `deserialize_sketch`.

### Bootstrap

On first connect for a saved server name:
1. `GetAllYrsPages`.
2. Run every `PageRow` through the primitive, using its `page_status`
   and disabled flag.
3. Set `bootstrapped`.
4. Send one explicit `GetMyUnsentLetters`; its reply is the "caught
   up" moment.

Pushes that arrive during bootstrap wait in a buffer, then apply after.

### Receive worker

One task, one letter at a time.

- Id already in the seen set: ack, done. The seen set tracks in-flight
  and applied ids, because duplicate pushes are common: each notify
  makes the server re-push every unacked row.
- Otherwise: decode, apply, ack, record id.
- A non-create letter for an unknown page: leave unacked, trigger
  another pull after any applied `CreateNewPage`. Yrs holds updates
  that arrive before their dependencies, so out-of-order letters
  within a known page are safe.
- A real apply error: "copy your changes, reload" popup, stop the
  worker, no ack. The server keeps the letter.
- Apply on the open page goes through `run_exclusive`: send the
  batcher a message; the batcher task does flush, then take the
  lock, then apply, then release. Typed text only reaches yrs at flush
  time, so a merge before the flush would use stale offsets.

Dedup is persistent, in `seen_server_letters`. A letter id is written
before its ack. On login, if the saved server name no longer exists
(login returns NoSuchUser), the table is cleared — a stopgap for
"server db wiped" until generation ids land. See the "Later" section
below.

## Phase 3: UI refresh

### Open page refresh

After apply: merge into `page1.boss_of_yrs` (the same Arc the batcher
edits) and set `page.blocks` from the builder's returned list. If the
page is open but still loading, treat as closed; the load reads the
db anyway. Read from yrs, not the db, because a keystroke between an
await and the reload would otherwise be lost.

### Cursor

In `MyBlockContent`'s value-sync effect, when the textarea has focus:
- Compute common prefix and suffix in UTF-16 units.
- Caret in prefix: stays.
- Caret in suffix: shifts by length change.
- Caret in changed middle: goes to start of the change.

`sync_from_props` already pushes changed content into each existing
box's content signal and reuses boxes by `internal_id`, so remote
edits reach `MyBlockContent` without recreating the textareas.

### Menu

Global tick signal plus a changed-pages list. Menu refetches
`get_all_menu_pages` when the tick changes. Also the likeliest path to
understanding the back-button bug.

### Remote disable of the open page

Popup and navigate to `/Menu`.

### Popups

One per letter that changed something. `PopupContainer` mounts once in
`App`; remove the one in `Menu` (currently popups on page views are
invisible).

## Phase 4: sending

`kick_send()` runs after each executor in `ui_actions`, after a
successful batcher flush, and on connect. Single-flight, with a flag
that makes one more pass if a kick arrives mid-send.

Reads `client_love_letter` by id ascending, rebuilds each letter with
`build_a_love_letter` and `love_letter_to_bytes`. Sends
`SendLoveLetter` and deletes the row on Ok. The table becomes a queue
instead of a history.

Failure handling:
- Transport failure (socket dropped, reply cancelled): leave the row,
  don't count as failure, resend on reconnect.
- Server Err: retry the same letter with backoff (1s doubling to 30s
  cap, reset on success or reconnect). Also retries on any
  `kick_send`.
- Stop at the first failing letter, do not skip ahead. Sending later
  letters past a failed one risks the server seeing an edit before the
  block it edits exists.
- After 3 consecutive server errors on the same letter, show the
  "copy anything you're afraid of losing, then reload" popup once.
  Retries continue slowly in the background. The app does not lock.
  The border stays yellow throughout.

The common dev case: server db wiped, every letter fails with "no such
page". Popup tells you, fix is reset both sides.

## Mock client changes

- `mock_client_core::create_page`: `is_main_menu_page = true` so the
  web menu shows mock-created pages.
- Nothing else.

## Known limits for v1

- If you wipe the server db, wipe the web db too, or the local pages
  never reach the server. (Generation ids fix this; until then this is
  the reset-both-sides routine.)

## Later: dataset generation and server reset

The server owns a `dataset_generation` value in `key_value_storage`.
It's minted once when the db is fresh and only changes when the
dataset is deliberately replaced. Login carries it. The client stores
the last one it saw and refuses to sync against a different one.

Actions that hang off this, all later:

- Client tells server "wipe yourself and adopt my dataset as the new
  generation" (client is source of truth).
- Server tells connected clients "the dataset changed" and disconnects
  them; their stored generation is now stale.
- Export everything to JSON.
- Load from JSON and reset the server.
- Remove everything I have (local wipe).

The `NoSuchUser` clear-seen-set in `identity.rs` is a stopgap for the
wiped-server case until generation ids land. It only catches the case
where the user table was also wiped; a partial wipe isn't caught.

The long-term shape: the server is a sync point, not the source of
truth. Every client can be source of truth. The generation id is what
lets clients agree on "same dataset" without merging across resets.

## Decisions (defaults if not answered)

- Derive the server host from the page hostname, port and path
  hardcoded.
- Delete `client_love_letter` rows after they're sent. The table
  becomes a queue.
- `error_stuff` package rename touches ~10 Cargo.toml lines, no source
  files.

## Test plan

Run the server and use the mock client as the second user.

- Phase 1: kill the server for blue; devtools offline for gray;
  second tab for TwoTabs.
- Phase 3: mock client edits, page updates.
- Phase 4: web edits, mock client `sync`s.
