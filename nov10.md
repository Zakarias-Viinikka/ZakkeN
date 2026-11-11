# 2026-11-10 — decisions and open work

## What this is

Snapshot of what got decided while implementing the drift version system, and
what's still open. Written same-day so nothing gets lost between sessions.
The code is the source of truth; this file is the map of why it looks the way
it does and what's still planned.

## Decisions

### The receiver checks. Every time.

Whoever receives a request compares the drift version the sender claims
against the one the receiver is currently on. Mismatch means the receiver
throws it away. This applies in both directions:

- server receives a client request → server checks
- client receives a spontaneous server push → client checks

It applies because of races. A change built on an old drift version must
never get applied after the other side has moved to a new one. Without a
check, a request sent before the drift sits in the queue and runs against
data it wasn't built for.

### The check has explicit exemptions

Register, Login, and GetCurrentDrift skip the drift check. So does the
socket's own internal pull. Reasons:

- Register and login are how a client learns the server's current version.
  Checking them is circular.
- GetCurrentDrift is how a client explicitly asks what the current version
  is. Same reason.
- The pull isn't a client message. There's no client on either end. It has
  no version to check.

Represented in code as `WhereTheRequestCameFrom::FromAClient(Option<String>)`
vs `WhereTheRequestCameFrom::FromTheServer`. The `Option` is there because
register/login/GetCurrentDrift have no version to send (the client hasn't
learned it yet, or is asking what it is).

### Drift version changes unregister everyone

Every bump source unregisters all registered users. Wipe does this today
because wiping deletes everything anyway. Adopt must do it explicitly when
it's built.

Stale clients stop accumulating unsent love letters. Their next request
bounces off "no such user" instead of a drift-version refusal. Recovery goes
through register + login again.

This is enforced today by wipe's table deletion, not by an explicit step.
When adopt lands, it needs the explicit step.

### The server tells stale clients why before closing

Before the server closes a stale client's socket, it sends the client a
message saying the drift version changed and why. This is the drift-changed
push. Clients don't have to discover the situation by failing a request.

The push fires from `notify_drift`, which sends to every logged-in socket
except the caller's own socket. Wipe calls it after bumping. The caller
stays connected; everyone else is told and closed.

The wipe caller is excluded because its socket stays alive across the bump.
It uses the reply's drift version, not a reconnect.

### Drift solution is picked by the caller, not the server

Whoever causes the drift attaches a `DriftSolution` (an enum in
`server_protocol`) that says what stale clients should do about it. Today's
values:

- `WipeSelfAndAdoptServer` — throw away local data and pull everything from
  the server.
- `Reconcile` — merge local and server data page by page.

Absent (`None`) means the user picks manually. That picker isn't built yet.

The solution lives on the `drift_version` row and travels in login's reply.

### Login reply always carries the current version

That's the only way a client learns what the server is on. Without it, drift
is invisible.

The reply also carries the reason (a human-readable string saying what
caused it) and the solution.

### Server errors use the server's error type

`ServErr` is the wire error type now, not `DbError`. Client refusals and
internal faults don't get to wear db-error labels, even when the underlying
operation touched the db.

`ServErr` variants and what they mean:

- `DbErrorContainer(DbError)` — a real db fault.
- `ServerError(ErrorDetails)` — internal breakage, no clean recovery.
- `ClientGone { user_name }` — the client's socket is gone.
- `RejectedBecause(String)` — the client sent something the server won't
  accept.
- `BadCode(String)` — server-side bug that isn't a db fault.
- `DriftVersionMismatch` — the check refused.

### No placeholders

A `todo!()`, a panic, or a broken build beats a silent placeholder. Empty
string defaults and `Option::None`-that-means-something get forgotten.
Panic or fail to compile instead.

(This is a workflow rule, noted here for porting into the workflow prompt.)

### The telephone is not optional

If the telephone's task dies while sockets are still connected, the server
panics. A gone telephone means the server has lost a piece it can't function
without.

### Connected means "login AND drift comparison finished"

`ConnectionStatus` gained `LoggingIn`. The state machine:

- Socket opens → `LoggingIn`. Sends are gated.
- Login succeeds, drift comparison runs → `Connected` or `Lonewolf`.
- Login fails → `Offline` (falls through; `FailedToConnect` stays in the
  enum, unused).

`sync_may_run()` keeps its single line: `CONN_STATE == Connected`. No extra
flag.

### Failed socket open falls to Offline

`FailedToConnect` was meant to be "server responded but the socket failed."
But the browser gives one error for both "server is down" and "server
refused," so the distinction is fake. Fails → Offline. `FailedToConnect`
stays in the enum unused until something meaningful produces it.

### Colour scheme

Border and connection indicator:

- Connected — green
- Connecting — yellow
- LoggingIn — yellow
- Offline — blue
- Lonewolf — blue
- LoneWolfAndOffline — blue
- TwoTabs — grey

Blue for "not connected and aware" because it's not an error. Yellow for "in
progress." Green for working. Grey for TwoTabs because it's not actually a
connection state, it's the state of being blocked by another tab.

### The drift-changed push is not a love letter

Spontaneous pushes come in two flavours: love letters and drift-changed.
The router forwards both to a single owner of spontaneous messages. The
owner matches on variant. Love letters get forwarded to the receive task.
Drift-changed enters lonewolf.

Before this, the router had a per-type callback (`on_love_letters`). Now it
has one callback for whole `Response`s. Any future spontaneous variant
needs no router change.

### delete_everything queues DisablePage letters

Not a new letter variant. `DisablePage` is already the delete-page letter —
it merges into the page_status doc, the server stores the result, other
clients receive it, and the CRDT handles concurrent edits.

`delete_everything` now:

1. Flushes the batcher.
2. Reads every page id.
3. Inside one `run_exclusive` and one `run_in_transaction`, for each page
   that isn't already disabled: build the disable-page context, run the
   disable executor steps. One queued letter per page.
4. Kicks the sender and bumps `MENU_TICK`.

No table wiping. No `enter_lonewolf`. The letters go through the normal send
loop. A reload doesn't bring the pages back because the server has the
disable state.

Hard-delete on the server was rejected on purpose. Consequences of hard
delete:

- A client that still has the page open sends edits to a page the server no
  longer has, gets "no such page," and retries forever.
- No tombstone, so reconnecting clients can't tell "deleted" from
  "never synced."
- The server has no block table, so there are no server-side block rows to
  delete anyway.

## Structure changes

- `server_errors` is the crate. Was `error_stuff`. The folder is
  `shared/server_errors`. `server_protocol` re-exports `ServErr` so the
  client gets it with the wire types.
- `db_actions_for_drift` is the module. Was `operations_drift_version`.
- `enter_new_drift_version` is the function. Was `bump`.
- `TelephoneCommand` is the per-socket enum. Was `WhatTheTelephoneWants`.
  Two variants: `GoPullYourUnsentLetters`,
  `TellTheClientTheDriftVersionChangedThenClose`.
- The telephone's task split into `task/mod.rs`, `task/lookups.rs`,
  `task/notifications.rs`. Shared dead-socket cleanup lives in
  `drop_dead_sockets`.
- The socket's queue plumbing and response-sending factored into
  `make_request_from_the_socket_directly_and_await_reply` and
  `send_response_to_client`. Both the pull path and the drift-changed path
  use them.
- `drift/` folder under `owner_of_interacting_with_server/`. Holds
  `enter_lonewolf.rs` and `owner_of_spontaneous_messages_from_server.rs`.
- `get_drift_version()` / `set_drift_version()` are globals in
  `shared_internals::drift_version`. Router reads them on every send.
- `AppBorder` component wraps the app in `main.rs`. Border colour tracks
  connection state.
- Dev panel grew a "wipe self and adopt server" button, under an `<hr>`.

## Wire

- `Message.drift_version` is `Option<String>`. `None` means the client
  hasn't learned the version yet (register, login, GetCurrentDrift). `None`
  on anything else is refused.
- `GetMyUnsentLettersPayload` gained `drift_version: String`. The socket
  stamps it onto every `LoveLetterFromServer` push.
- `Response::LoveLetterFromServer { letter, drift_version }` — the push
  carries the current version, so the client can throw away a letter built
  on a dataset it isn't on.
- `Response::TheDriftVersionChanged { drift_version, drift_reason }` — sent
  just before the socket closes.
- `Request::GetCurrentDrift` — no payload, returns the current
  version + reason + solution. Exempt from the check.
- `WipeDatasetPayload { drift_solution: Option<DriftSolution> }` — the wipe
  request's payload. Dev tools currently send `None`.
- `LoginReply` gained `drift_solution: Option<DriftSolution>`.

## Open work

### Blocking on the client side

Nothing user-facing yet. Client always enters lonewolf on mismatch; the
`drift_solution` in the reply is ignored.

### Drift_solution reaction at login

The mismatch branch in `main_for_login_and_registration.rs` enters lonewolf
and pops a message. It should:

- `WipeSelfAndAdoptServer` → call `wipe_local_and_pull_from_server`.
- `Reconcile` → run reconcile (not built).
- `None` → stay in lonewolf with a popup (today's behaviour).

The picker for `None` isn't built.

### Wipe button picker

Dev tools send `None` for `drift_solution`. To test end-to-end, the wipe
needs a real choice. Either a picker UI or a hardcoded value for now.

### Reconcile verb

Not built. The existing button in the dev panel says `todo!()`. Needed
before `DriftSolution::Reconcile` means anything.

### Adopt as second bump source

Not built server-side. When it lands, it must also unregister all users and
mint a new version, same as wipe. It's not in scope today.

### Adopt racing with concurrent edits

Already in `todo.md`. Summary: adopt can wipe an edit that another client
already got acked. The fix is the other client's reconcile step, not the
server. Not urgent until adopt exists.

### Lonewolf doesn't clear pending letters

Earlier in the day I made `enter_lonewolf` clear the pending letter queue
and the seen set. That was wrong — entering lonewolf doesn't mean throwing
away local data; that's what wipe / reconcile / adopt are for. Reverted.

But the question is still open: should stale queued letters from the old
dataset fire if the client leaves lonewolf? If the answer is no, some verb
needs to clear them — reconcile and wipe do it as part of their flow.
Today, leaving lonewolf via the existing exit (wipe_local_and_pull_from_server)
clears them. Any future exit needs to as well.

### The drift-changed push has never actually fired

The wire variant exists, the socket sends it, the client handles it. But
end-to-end testing requires two clients with different data and one wiping.
Not done.

### LoggingIn not yet exercised

The state exists, the transition exists, but it fires briefly during login
and is hard to see. Would be worth confirming the yellow flash is
`LoggingIn` and not `Connecting` in the browser.

### Session-ready flag — dropped

The plan originally called for a client-side flag meaning "login AND drift
check done." Replaced by the `LoggingIn` state. No flag.

### Workflow prompt

The "no placeholders" rule needs to land in the workflow prompt, not just
`human_decisions.md`.

### Android docs

Updated to describe what a working sync needs on the wire, including the
drift rules. Done.

---

**Superseded 2026-11-11.** The "Open work" section below is out of date.
Reconcile is built end to end (see `nov11.md`). The decisions here are
still accurate; only the open-work list moved on.
