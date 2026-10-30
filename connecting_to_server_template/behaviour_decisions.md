# Behaviour decisions

This file exists so that at the end of a refactor, or when handing the template to a new AI, there's one list of what the template is supposed to do. Each paragraph below is a claim. "Claim" means: if a change makes the code stop doing this, either the change is wrong or the claim has changed and the paragraph needs updating.

## How to use it as a quiz

Hand the AI this file and the source tree. Ask: "For each paragraph, does the code do this? If not, which is wrong — the code or the paragraph?" Then triage the mismatches yourself.

Not every paragraph is a behaviour the user would notice. Some are structural (which function calls which, where a rule is enforced). Structural claims matter because violating them produces weird behaviour later, and the quiz catches drift the same way either kind would.

The last section, "Not decisions", is the exception: those are placeholders, not rules, and the right answer to "shouldn't this do X?" is "yes, but in the real project".

## State machine

There are two sync modes: `Connected` and `NeedsReconcile`. There is no `Paused`. Pausing is a way of entering `NeedsReconcile` with the reason `YouPaused`. (`sync_mode.rs`)

`NeedsReconcile` always has a reason set. The seven reasons are `YouPaused`, `YouWiped`, `ServerChanged`, `WipeIncomplete`, `AdoptIncomplete`, `Reconciling`, `ReconcileIncomplete`. Entering `NeedsReconcile` with no reason is a bug. (`sync_mode.rs`)

"Lonewolf" is not a sync mode. It is `ConnState::Connected` combined with `SyncMode::NeedsReconcile`: the socket is up, login succeeded, sync is stopped. `is_lonewolf()` tests this combination and nothing else. (`server_link/conn_state.rs`)

Mode and reason persist across reloads. If you were lonewolf when you reloaded, you come back lonewolf. `Connected` does not persist to localStorage; it's the in-memory default on boot. (`sync_mode/persistence.rs`)

## Gating

`is_gated()` reads `SyncMode != Connected`. It is the single predicate for "sync operations should not run". (`sync_mode.rs`)

Four places read `is_gated()`. `receive_letter` refuses to apply anything while gated. `kick_send` refuses to start a send loop while gated. The per-send check inside `kick_send`'s drain loop refuses to send any single letter while gated. `restart_sync_stack` runs login but skips bootstrap and pull at the tail. (`server_link/receiving.rs`, `server_link/sending.rs`, `server_link/restart.rs`)

The gate is not "offline". Gated means "connected and syncing is stopped". Login and re-registration still run while gated, because they're about talking to the server, not about syncing the dataset.

## Server

The server boots off. "Start server" does two things: reveals the panel and turns the server on. "Turn off server" / "Start server" toggle after that. Power off keeps data. (`server.rs`, `server/power.rs`)

Two fake latencies: 500ms light, 900ms heavy. They are not uniform. `AreYouOnline`, `Login`, `Register` are light. `SendLetter`, `WipeDataset`, `AdoptDataset`, `GetAllPages` are heavy. (`server/latency.rs`)

Replies are deferred. The server mutates state at the moment the message arrives, then delays the reply by the latency. If power drops during the delay, the reply is dropped — the client's caller hangs, matching a real network. State changes that already happened stay applied. (`server/latency.rs`, `server.rs`)

"Fail next request" arms a one-shot. The next mutating request (`Register`, `Login`, `Send`, `Wipe`, `Adopt`) fails with a fault-armed message and clears the arm. The request after that succeeds. (`server.rs::take_armed_fault`)

`Register` and `Login` are distinct. `Register` refuses a name that's already taken with `ERR_USER_ALREADY_EXISTS`. `Login` refuses a name that isn't registered with `ERR_NO_SUCH_USER`. Those two strings are the contract; both sides match on them. (`protocol.rs`, `server.rs::register_user`, `server.rs::log_client_in`)

Neither `WipeDataset` nor `AdoptDataset` clears `registered_users`. Registered users are accounts, not dataset content. Only the "Another client did" operations clear the user list. (`server.rs`)

## Test bench: "Another client did this"

"Wiped the server" bumps `dataset_generation`, clears pages, clears the backlog, clears `registered_users`. If the current client was live at that moment, the server sends `ServerSaysYouAreNowLonewolf { what_happened }`. The socket is not closed. (`server.rs::simulate_other_client_wiping`)

"Reconciled with the server" queues three `pretend_edit` letters. No generation bump, no user-list clear, no socket close. From this client's POV, a burst of normal letters. (`server.rs::simulate_other_client_reconciling`)

"Told the server to start over from its version" has the same shape as "Wiped" — generation bumped, users cleared, pages replaced with `CANNED_PAGE_ID_C`, backlog cleared, lonewolf message sent — but the page set is populated, because the other client's data came with it. (`server.rs::simulate_other_client_starting_over_from_its_version`)

All three operations change server state regardless of whether the current client is live. The lonewolf message only fires if the current client was live. An offline client learns about the wipe on its next reconnect, via `log_in` seeing a new generation and `stop_sync_if_dataset_changed` transitioning to `NeedsReconcile + ServerChanged`. (`server.rs`, `server_link/restart.rs`)

## Client: server_link

`restart_sync_stack` returns `RestartOutcome` — `Synced`, `AlreadyStopped`, `JustChanged`, `Failed(String)`. It never pops. `announce_connect_outcome` is the layer that turns the outcome into a popup. (`server_link/restart.rs`, `data_ops.rs`)

`log_in` tries `Register` first, falls through to `Login` on `ERR_USER_ALREADY_EXISTS`. First connect registers. Every later connect logs in. The order is not "login then register" — the server rejects a login for an unknown user, so the first connect has to register before it can log in. (`server_link/restart.rs`)

On `ConnectionLost`, `go_offline_after_connection_lost` sets `ConnState::Offline`, clears the sending flag, and pops a "connection lost" warning only if the client was `Connected` at that moment. (`server_link/conn_state.rs`)

On `ServerSaysYouAreNowLonewolf`, the client does *not* go offline. It sets `NeedsReconcile + ServerChanged`, clears seen letters, clears pending edits, pops one warning naming what happened, and calls `kick_lonewolf_retry`. The socket stays up. (`client/server_messages.rs`)

The drain loop inside `kick_send` re-checks `Connected` and `is_gated()` for every letter, not just at the start. A mid-send gate change stops the loop before the next letter. (`server_link/sending.rs`)

Send failures split. Server errors pop a warning ("Sending failed: … N edits still queued"). Transport failures are silent, because `ConnectionLost` already popped a warning about the connection, and the second popup would be noise. (`server_link/sending.rs`)

The receive order per letter is fixed: if gated, bump ignored-while-stopped and do not ack. If already seen, ack only. Otherwise apply, mark seen, bump applied, ack. (`server_link/receiving.rs`)

## Retries

Two retries exist and they do not overlap. They fire on disjoint `ConnState` values.

The offline retry fires when `ConnState` is `Offline` or `FailedToConnect` and the client is on. It sleeps 5s, pops "Trying to reconnect…", calls `restart_sync_stack`, and either announces success via `announce_connect_outcome` and exits, or continues the loop. If a manual connect succeeds while the retry is sleeping, the retry's next iteration notices the state is no longer offline and exits silently. (`app_main.rs::retry_reconnect_when_offline`)

The lonewolf retry fires when the client receives `ServerSaysYouAreNowLonewolf`. It sleeps 2s, calls `restart_sync_stack`, and either exits on success or tries again. Up to 3 attempts. Silent on success. Silent on failure too — the popup from `handle_server_says_you_are_now_lonewolf` is the only popup, and it already told the user what happened. (`server_link/restart.rs::kick_lonewolf_retry`)

The two retries don't collide. Offline retry runs on `Offline` or `FailedToConnect`. Lonewolf retry runs on `Connected`. Those are mutually exclusive.

## UI

The Client box title is a reactive string derived from `ConnState` and `SyncMode`: `Client — Offline`, `Client — Connecting`, `Client — Failed to connect`, `Client — Connected`, `Client — Lonewolf`. "Lonewolf" appears only when both `ConnState::Connected` and `SyncMode::NeedsReconcile` are true. (`app_main.rs::client_title`)

Stopped-sync buttons (Reconcile, Adopt my data, Wipe server & start from mine, Wipe mine & start from server) are visible only in lonewolf. Everywhere else the group is hidden. Not disabled — hidden. (`controls.rs`, `<Show when=move || show_stopped.get()>`)

Server-off disables: power controls, Wipe server, Push pretend letter, Fail next request, and all three "Another client did this" buttons. Client-off disables: Connect to server, Pause sync, every button in the stopped-sync group, and Wipe server. Disabled, not "refused by popup".

"Fail next request" is not disabled when it's already armed. Arming twice is a no-op (the arm stays armed). No "you already armed it" popup.

## Popups

One popup per verb. Refusals first. If a verb refuses and also fails, the refusal is the popup — you never get both. (`data_ops/verb_error.rs`)

`VerbError::Refused` becomes a `Warning` popup. `VerbError::Failed` becomes an `Error` popup. This split is enforced by `announce_verb_error`, not by each call site.

Popup coalescing: same kind + same message resets the existing toast's timer instead of stacking. Different kind or different message stacks as a new toast. (`cool_popup.rs`)

Lifetimes: Info 6s, Success 5s, Warning 8s, Error 10s.

## Not decisions

These look like decisions but they aren't. They're placeholders. If the AI keeps asking "shouldn't this do X?", the answer is "yes, but in the real project".

`local_pages.rs` reads from a placeholder, not a real local db. `apply_server_pages_locally` logs a count. `read_local_pages_placeholder` is a placeholder. `start_over_fresh` logs the page count and doesn't build pages.

`classify_pages_placeholder` in `reconcile/classify.rs` classifies by whether the server returned any pages. The real classifier compares per-page state.

The pending queue is a `Vec<LoveLetter>` signal, not a persistent outbox. `pending_count_signal` counts it. Nothing survives a reload.

A diff is a string here, not a real `TextEdit`. The template has no years, no CRDTs, and no real local db. Anything that in the real app would go through `BossOfYrs` or `web_internal_db` is a signal here.

The data-ops verbs (pause, wipe_server, adopt_local, wipe_local_and_pull_from_server) do real state-machine work but touch the server through the mock. `pause()` returns `Result<(), String>` while every other verb returns `VerbError` — leftover from an earlier shape. The popup is the same Warning either way.

`reconcile_cancel` restores the previous mode and reason from a session that holds them. If the previous mode was `NeedsReconcile` with no reason (possible after reload), the restore leaves the reason as `Reconciling`. This is a bug, preserved.

`client_server_coms` is a pair of `mpsc` channels, not a socket. The `Router` in the real `shared_internals_for_server` crate is the actual socket layer; here the "connection" is the channel pair and `ConnectionLost` is just a message.
