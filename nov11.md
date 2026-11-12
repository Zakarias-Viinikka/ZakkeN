# 2026-11-11 — reconcile flow

## What this is

Same shape as `nov10.md`: decisions made while building the reconcile
flow, and what's still open. Code is the source of truth; this file is
the map of why it looks the way it does.

## Decisions

### Reconcile uses four new requests, all exempt from the drift check

`GetPageFingerprints`, `GetPagesByIds`, `GetReconcileVersion`, and
`ReconcileLetters`. The first three are reads a lonewolf client needs
to run reconcile. `ReconcileLetters` is a write, and it checks its own
claimed target version inside the handler (not in the drift gate).
Everything else is still gated.

### Fingerprint = SHA-256 of raw page bytes

Both sides call the same helper in `drift_guard_and_ops/shared`.
Hashing bytes rather than decoded block texts means a page with
identical text but a different yrs encoding (different edit history)
still shows up as a mismatch. Accepted tradeoff: cheaper than decoding
every page on the server, and the worst case is an extra card the user
can skip by picking "use server's".

Why SHA-256 and not `DefaultHasher`: the client and server are built
separately (different targets, possibly different rustc). `DefaultHasher`
can change between compiler versions; SHA-256 is standardised and stable.

### reconcile_version prevents two reconcilers from colliding

`reconcile_version` is drift_version but for reconciles. Without it,
two clients can both read the current dataset, both prepare a batch
off that snapshot, both send — and the second batch lands on data the
first already changed. That breaks stuff in a nasty way even though
it's unlikely. With it, whichever batch lands first bumps the version;
the second's claimed version is stale; its batch is refused with
`DriftVersionMismatch`; the client reloads and starts over.

Lives in `key_value_storage` under `reconcile_version`. "0" if never
bumped. Bumped inside the same transaction as the batch, so a failed
apply leaves the version unchanged.

### Reconcile does NOT bump drift_version

Other clients are not informed, not disconnected. Each lonewolf client
reconciles separately. The alternative (bump drift, notify everyone)
would kick every connected client into lonewolf every time any client
reconciled, which is worse.

### The client wipes itself after a successful reconcile

The order: build the batch, send it, wipe locally, reload. On reload
the client re-logs-in against the server's current drift version,
pulls the fresh dataset back, and adopts it. That's how the client
ends up in sync without ever needing a "reconcile succeeded" state.

### Planner is pure, synchronous, on the client

`drift_guard_and_ops/client/src/reconcile_planner/`. Input: the server's
blob (opened into a scratch `BossOfYrs` with a unique actor name, so
yrs clock collisions are impossible) and the target block texts.
Output: `Vec<LoveLetter>`.

Algorithm: LCS-align the initial block list vs target. Walk hunks left
to right. Equal blocks are already in place. In a replace hunk, pair
removed[i] with inserted[i] as `EditBlock` on the server's block id;
extra removed → `RemoveBlock`; extra inserted → `CreateNewBlock`. No
`MoveBlock`; a reorder becomes remove + create. Each op is applied to
the scratch doc, then `generate_diff_snapshot` captures it, then
`build_a_love_letter` wraps sketch + snapshot. Self-check at the end:
scratch doc's texts must equal target; if not, error and send nothing.

### The diff viewer outputs only text

`DiffViewer` (in `web_diff_viewer/block_diff_viewer_leptos`) takes
`left_page: Vec<String>` and `right_page: Vec<String>`, outputs the
edited right side as `Vec<String>`. No block ids in the output. So
the planner recovers block identity by LCS-matching against the
server's blocks. "Pick mine" and "Merged" both reduce to "here is the
target list of block texts"; the planner decides which ones to edit,
which to remove, which to create. "Pick server's" means no letters —
the server already has that state.

## Structure changes

- New workspace member: `drift_guard_and_ops/shared` (package
  `drift_guard_shared`). SHA-256 fingerprint helper.
- New workspace member: `drift_guard_and_ops/diff_viewer_internals`
  (package `diff_viewer_internals`). Blob → ordered block texts.
- `drift_guard_and_ops/client/src/reconcile_planner/`. The planner.
- `server_core/src/allowed_requests/send_love_letter.rs`:
  `accept_letter` renamed `main_for_accepting_a_letter`. Per-letter
  work extracted into `apply_one_letter`, callable from both
  `main_for_accepting_a_letter` and `reconcile_letters`. The
  transaction lives at the caller, not inside `apply_one_letter`.
- New handler files under `allowed_requests/`:
  `get_page_fingerprints`, `get_pages_by_ids`, `reconcile_letters`,
  `get_reconcile_version`.
- `cqrs_state_synchronization/web_interface/src/leptos_components/reconcile_page/`.
  The page.
- `cqrs_state_synchronization/web_interface/src/owner_of_interacting_with_server/reconcile_requests.rs`.
  Async wrappers around `router_handle::send`.
- `shared_internals::request_builder::ClientCommand` grew the four
  reconcile variants. `router_handle::send` now returns `ServErr`
  instead of `String` so reconcile can match on
  `DriftVersionMismatch`. Existing callers stringify with
  `format!("{e:?}")`.

## Wire

- `Request::GetPageFingerprints` (no payload) → reply
  `{ drift_version: String, pages: Vec<{ page_id, fingerprint }> }`.
- `Request::GetPagesByIds { page_ids: Vec<String> }` → reply reuses
  `GetAllYrsPagesPayload`. Unknown ids are skipped silently.
- `Request::GetReconcileVersion` (no payload) → reply
  `{ reconcile_version: String }`.
- `Request::ReconcileLetters { target_drift_version: String,
  target_reconcile_version: String, letters: Vec<Vec<u8>> }` →
  `Result<(), ServErr>`. Both versions are checked against the
  current rows inside the handler. All letters applied in ONE
  transaction; the whole batch rolls back on any failure. Bumps
  `reconcile_version` inside the same transaction.

## Open work

### Login-mismatch path enters lonewolf

The mismatch branch in `main_for_login_and_registration.rs` enters
lonewolf and pops a message. That's correct — routing to `/reconcile`
on mismatch is not the intended behaviour. Reconcile is entered via
the dev sidebar only.

`drift_solution` is still ignored for the other two values:
`WipeSelfAndAdoptServer` should call `wipe_local_and_pull_from_server`,
and `None` stays in lonewolf while a picker is built.

### Picker for drift_solution = None

Dev tools still send `None` for `drift_solution` on wipe. No picker.

### Adopt as a second bump source

Not built server-side. When it lands, it must also unregister every
user and mint a new drift version, same as wipe.

### e2e test for the mismatched-page path

`complete_e2e/tester/tests/test_006_reconcile/test_reconcile_client_only.py`
covers client-only pages. The diff-viewer path (both sides have the
same page id with different content) isn't covered yet.

## Update - 2026-11-12

Reconcile is built and merged (`3b1f79d`). Continued on adjacent work.

### complex_state/

The `shared_owner/` folders across the codebase were renamed to
`complex_state/`. The naming convention is documented in
`coding_style.md`: one file per piece of state that is created in one
place and touched by many. Default mechanism is a global getter and
setter, named with the `plexget_` / `plexset_` prefix so a call site
instantly says "I'm touching complex state".

Two files added under `complex_state/`:

`conn_state.rs` — the connection state signal, with
`plexget_conn_state()` (subscribes, for view closures),
`plexget_conn_state_untracked()` (no subscription, for send loops and
drift detection), and `plexset_conn_state(state)`. The doc comment
lists every reader and every writer, and for each writer says what
would stop happening if that writer didn't exist.

`drift_version.rs` — the drift version, both the in-memory cache
(which `shared_internals` reads synchronously on every send) and the
DB copy (which survives reloads).
`plexget_drift_version()` panics if the cache hasn't been populated;
only legitimate after login. `plexget_drift_version_from_db()` is
async and is what the login comparison uses.
`plexset_drift_version(v)` writes both copies.
`plexset_make_new_drift_version()` mints a fresh one internally so
callers don't need to know how a version is formed.

`new_drift_version(user_id, time_in_unix)` lives in
`drift_guard_and_ops/shared`, callable from either side.

### Reconnect task

The retry loop used to live inside `shared_internals::socket_web`,
running forever on socket close. Removed. The socket task now waits
for an external "try now" signal and opens at most one connection per
burst.

`web_interface` owns the retry decision now, in
`owner_of_interacting_with_server/main_for_reconnecting_automatically/`.
A task runs once per second. Each tick reads `ConnState` and, if the
client is offline, asks the router to try opening a connection.

The router gained `attempt_reconnect` on its handle. The app calls it
once right after the first setup so the first connection doesn't wait
for a tick.

### Deliberate-lonewolf button

New dev-panel button that puts the client into lonewolf on purpose:
bumps the local drift version (via
`plexset_make_new_drift_version`) and flips `ConnState` to lonewolf
(or lonewolf-and-offline if it wasn't connected). Wired to
`db/dev_actions::enter_lone_wolf`. Also exposed to the tester as the
`enter_lonewolf` window event so tests don't need the sidebar button.

### OfflineDeliberately — parked

A new connection state for "offline on purpose, don't poll to
reconnect". Not built. The plan: add the variant, add a guard to
`should_i_reconnect`, add a "go online" exit. Won't persist across
reload, so a reload always re-enables reconnect. Accepted.

### New expose methods

`method_read_conn_state` and `method_enter_lonewolf`. Both registered
in `expose_methods_main!()`.

### Login mismatch in lonewolf is correct

Docs earlier in this file said the login mismatch path should route
to `/reconcile`. It shouldn't — entering lonewolf is the intended
behaviour. Reconcile is entered via the dev sidebar only. The
`drift_solution = WipeSelfAndAdoptServer` case still doesn't act
automatically, and the picker for `None` still isn't built.
