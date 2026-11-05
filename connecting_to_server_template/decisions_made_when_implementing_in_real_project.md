# Decisions made when implementing in the real project

These are decisions made while porting the template's shapes into
`web_interface` and `zakken_server`, or made in the conversation that
produced the port plan. Where they contradict `behaviour_decisions.md`
or the template's code, the real project follows these.

## Sync mode

Two modes, not three. `Connected` and "sync is off". No `Paused`. Pausing
is not a user action; the app enters the stopped mode on its own when it
needs to.

The stopped mode is called `NeedsReconcile` in the template. In the real
project it is renamed `LoneWolfAndOffline`, because that is what it
actually is: lonewolf, but with the socket down instead of up.

`Lonewolf` and `LoneWolfAndOffline` both mean "sync is off". The only
difference is the socket. Lonewolf has the socket up, so verbs that need
the server (reconcile, adopt, wipe) can run. `LoneWolfAndOffline` has the
socket down, so they can't until a reconnect.

## Hard lock

Dropped. From the client's point of view, the hard lock and lonewolf are
the same thing: connected to the server, sync stopped, popup shown, the
user has to act. The differences the template had — forcing the socket
down, no recovery except reload — aren't needed. Reconciling is not
retrying the rejected letter, so a client that just got two rejections
can still reconcile its way out.

On the first server rejection of a letter, retry once. If the retry is
also rejected, go to lonewolf with a reason that says so. No separate
locked state, no forced offline, no reload required.

## ConnectionStatus

The client's status, not just the socket. It includes states that are
about sync (Lonewolf, `LoneWolfAndOffline`) alongside states that are
about the socket (Offline, Connecting, FailedToConnect, Connected),
because the UI reads one value to say what the user's situation is.

## Where the code lives

The sync state and recovery logic is a crate at the repo root,
`drift_guard_and_ops/`, as a workspace. It is not inside
`cqrs_state_synchronization/`, because the server may eventually need to
depend on parts of it, and a member of the cqrs workspace can't be
depended on by `zakken_server`.

First crate inside it is `client/`. It holds the client-side state and
verbs. Whether a server crate is also needed is unknown; the code will
tell us.

The name `drift_guard_and_ops` is for "the thing that guards against
drift, plus the operations that resolve drift". Drift is when the
client's and server's copies can no longer merge.

## Reconcile

Client-driven. The client asks the server for hashes of its local pages,
gets back which pages differ, then asks for the full version of each
differing page one at a time. No new composite request variant on the
server. The client designs the exact wire shapes.

The merge UI is `web_diff_viewer`. Left = server texts (frozen
reference). Right = local texts (editable, result). Pressing Done without
touching anything keeps the user's own work. "Use server for all"
bypasses the viewer.

Whether the merge is applied page by page or collected and applied at
the end is the human's decision; both work.

## Dataset generation

The mechanism is not settled. The template fakes it, and the port plan's
step 8 lays out the design questions and the recommendations. The
implementer discusses it with the human before writing any of it.

What is decided: a pushed "you are now lonewolf" message exists as a
concept. The mechanism (new response variant, widened telephone channel,
second Router callback) is a recommendation, not a decision. A cheaper
alternative exists: the server closes other sockets, the client
reconnects, and the login-time generation check catches it. The cost is
the client goes briefly offline instead of staying connected, and the
human wants it to stay connected.

The fingerprint for "which pages differ" is SHA-256 of the raw blob
bytes (and the status bytes). Byte-identical always means the same
lineage. A content fingerprint could say "same" for two pages that look
identical but have different lineage, which is exactly the bug the
design avoids.

## Flags

`retiring` (set just before `location.reload()` to stop work slipping
in) is dropped. Instead, transitions into `Connected` write the persisted
mode only, not the in-memory signal, then reload. The gate stays closed
until the reload.

`session-ready` (login AND the generation check both completed on this
socket) is kept, but it belongs to the dataset-generation step, not the
reconnect step. Without it, a send can slip through between "login
returned" and "the client's own generation check done", and that send is
real corruption.

## The yrs primitives

`BossOfYrs::from_hard_copy` and `to_hard_copy_blocks` from the old plan
are `freeze_doc` and `freeze_doc_builder` in the real code. A frozen
block carries id, content and meta — no frac. File order is the order;
mirror-row fracs come from the rebuilt doc.

## Request building for out-of-worker callers

Client commands that need the socket but aren't spawned from `start_link` (the wipe button, later reconcile/adopt/export/import) send through a small layer. The command is an enum (`ClientCommand`) with one variant per verb. The mapping from that enum to what the server expects (a `Request` plus its payload) lives in `shared_internals::request_builder`, one `build_*_request` function per command plus one `build_request` that dispatches. `web_interface`'s `router_handle` only holds the handle and sends; it doesn't know what a `WipeDataset` is. Adding a new command is a variant, a builder function, and one match arm.

## Default port fallback

The connecting gate starts closed on a fresh tab, because nothing has stored a port yet. To make a plain load connect without visiting the custom-port page, `connect_to_socket` sets the port to 3000 on any page except `/custom_port_setting_page`. That page needs the gate to stay closed until the tester fires `set_port`. This is a stopgap for the port-plan's `test_config.rs` work (`?test_mode=1&server_port=N` pinned in sessionStorage), which will replace it.

## Wipe

The wipe button is live and verified. Pressing it from the dev panel sends `WipeDataset` to the server; the server deletes every row from all six tables (pages, love letters from clients, unsent letters, registered users, logs, key_value_storage); a sqlite query after the click confirmed all six at zero. Because users are wiped too, the next client login fails with "no such user", clears the saved name, and re-registers. That's the expected flow for "start over completely", not a bug.

## Sync mode (replaced 2026-11-05)

The plan's sync mode — a separate persisted mode with a reason enum — is
dropped. It doesn't fit the ConnectionStatus model.

What replaces it: a single drift version number.

- The server stores a drift version, and a reason string when the
  version was bumped by a server-side action (wipe, adopt).
- The client stores its own drift version in key_value_storage.
- On login the client sends its version. Mismatch either way puts the
  client in Lonewolf.
- Import bumps the client's version locally. Wipe and adopt bump the
  server's. Same check catches both.
- No reason is stored on the client. When the client is the cause
  (import), a popup fires at the moment of the action — the user already
  knows why.
- After a reload with a mismatch, the popup says "out of sync with
  server", plus the server's reason if there is one.

Popup-once flag in key_value_storage: true when a connection completes
without being in Lonewolf, false when the mismatch popup is shown. If
false, don't re-show.

## Dataset generation (settled 2026-11-05)

The drift version is the plan's dataset generation. Server side: a
decimal string (unix microseconds) minted at startup if absent, re-minted
by wipe and adopt. Login reply carries it plus the optional reason.
Wipe and adopt replies carry it too. The session-ready flag from the
plan is kept as-is.

## What got built (2026-11-05)

The drift version landed on both sides. Server: table, mint at startup, bump on wipe, login and wipe replies carry version + reason. Client: stored version + popup-once flag, login comparison, `enter_lonewolf`, `sync_may_run()` wired into four gate sites. The web app's `ConnState` is now a `pub use` of the drift crate's `ConnectionStatus`, which grew `TwoTabs`, `Lonewolf`, `LoneWolfAndOffline`.

The batcher handle refactor also landed: `OnceLock` static in `shared_owners/batcher_handle.rs`, free functions instead of method calls on a handle passed through props.

Not built: adopt bumping the version, the SendLoveLetter guard, the session-ready flag, and every verb that would let a gated client reconcile or adopt. The only exit from lonewolf is a test-only wipe-and-reload.
