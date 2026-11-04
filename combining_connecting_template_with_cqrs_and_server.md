# Combining the connecting template with cqrs and the server

Written 2026-11-04. This is the implementation plan for porting sync
recovery and data management from `connecting_to_server_template/` into
the real client (`cqrs_state_synchronization/web_interface/`) and the
real server (`zakken_server/`).

Where this plan differs from `plan_for_general_data_management_ops.md`,
this plan wins. The main differences: no pause or resume;
classify_pages / diff_page / build_client_wins_update are gone; and
"kicking" other sockets is one option in step 8, not a decision.

The amendments at the end record what the human answered on 2026-11-04.
They are folded into the steps below; the amendments section is the
receipt, not the instruction.

## Read first

1. This file.
2. `ZakkeN/STORED_AI_CONTEXT.md` — the project map.
3. `ZakkeN/coding_style.md` — naming.
4. `plan_for_general_data_management_ops.md` — reference only. Where it
   disagrees with this file, this file wins. Its classify_pages,
   diff_page, build_client_wins_update, pause/resume and socket-kicking
   sections are obsolete.
5. `connecting_to_server_template/` and its `behaviour_decisions.md` —
   reference for SHAPE only. Do not port the fake server, power/latency,
   internet_switch, "another client did this" buttons, fault injection,
   or client_server_coms.

Files the plan depends on but the planning AI never saw are listed at
the end. Steps that depend on them say "check X first".

---

## Background you will keep coming back to

**Dataset, blobs, lineage.** The dataset is everything the user owns:
all pages and blocks. The server holds one copy and each client holds
another. Each page is a CRDT document (a yrs doc) serialized to a blob.
The client's sqlite tables (`every_block_in_existence` and friends) are
mirror rows, a queryable shadow of the blobs rather than the truth.
Edits travel as love letters, each carrying a small blob update.

**Lineage** is the key concept. A yrs update refers to items by
(creator id, clock), so two docs share a lineage only if one descends
from the other's actual bytes. A doc rebuilt from plain text (which
import does) has a new lineage even when it looks identical on screen.
An update from one lineage can't merge into another. Either it parks as
"pending" forever and edits silently vanish, or it unions two copies and
duplicates every block id.

The whole port follows one rule: **never merge across lineages.** When
two sides must agree, one side takes the other's blob, or builds edits
on top of the other's blob.

**Sync mode and gating.** Two modes: `Connected` (sync runs normally)
and `NeedsReconcile` (sync is stopped because local and server data
can't be assumed to share lineage). **Gating** means refusing sync work
while the mode, the hard lock (step 4) or a per-socket session-ready
flag (step 8) says not to run. Sync work means sending letters,
applying or acking received letters, bootstrap and pulling. Local edits
keep working while gated and pile up in `client_love_letter`. Each time
the app enters `NeedsReconcile` it records a reason (server changed,
imported, you wiped, wipe incomplete, adopt incomplete, reconciling,
reconcile incomplete). Entering with no reason is a bug.

**Lonewolf.** `ConnState::Connected` combined with
`SyncMode::NeedsReconcile`: the socket is up and logged in, but syncing
is stopped. It is not a third mode.

**Dataset generation.** A number the server changes whenever it replaces
its whole dataset (wipe, adopt, or the DB file being deleted). A client
remembers the generation it last synced under, and a mismatch at login
means its lineage assumptions are dead. Step 8 designs this.

**The verbs.** Export writes a JSON backup. Import replaces local data
from a backup. Wipe server empties the server. Adopt makes the server
take this client's pages. Reconcile compares local against server page
by page and merges them. Wipe, adopt and reconcile all start by setting
`NeedsReconcile` and stay in it until they succeed.

**The yrs primitives, already built.** `freeze_doc(FrozenBlock list)`
builds a fresh `BossOfYrs` from a block list. `freeze_doc_builder`
reads a live `BossOfYrs` into a block list. `FrozenBlock` carries
`block_id`, `content` and `meta`. It has **no frac**. File order is the
order; `freeze_doc` inserts in the order it is given, and the rebuilt
doc computes its own fracs. Wherever this plan says "from_hard_copy" or
"to_hard_copy_blocks", read `freeze_doc` / `freeze_doc_builder`. The
implementer should read the real signatures before writing step 5.

## Five ground rules

1. **Set `NeedsReconcile` before any destructive step**, and leave it
   only after the last step succeeded. Mode lives in localStorage and
   the data in the db, so they can't change atomically together. This
   ordering makes every crash land on the safe side.
2. **Every destructive local write is**: flush the batcher, then one
   `run_exclusive` closure, then (inside it) one db transaction. No
   network I/O inside the closure. Verbs do network work first and
   local writes last. Never call `merge.rs::apply_letter` or anything
   else that calls `run_exclusive` from inside a closure (it
   deadlocks). Use the pure builders plus `apply_merge_plan`.
3. **Every transition into `Connected` ends with a page reload**,
   behind a single function `restart_sync_stack()` so Android can swap
   it. A transition into `Connected` writes the *persisted* mode only,
   not the in-memory signal, and then reloads. The gate stays closed
   until the reload; no extra flag is needed. Transitions out of
   `Connected` take effect immediately.
4. **Any change of dataset generation clears the seen-letters set** in
   both places (the `seen_server_letters` table and the in-memory
   `SEEN_IDS`) through one `clear_seen_everywhere()`.
5. **Destructive verbs only run in the tab that holds the two-tab
   lock.** Export may run anywhere.

---

## Step 1 — Sync mode, reason, persistence, and one gate predicate

**What and why.** Everything else reads this state, so it comes first.
Create a new folder `owner_of_sync_state_and_recovery/` in
`web_interface/src/` (register it in `lib.rs`). It contains a sync-mode
file, a persistence file and, later, the hard lock and busy files.

The mode and reason live in static signals, like the template's
`sync_mode.rs`. They persist to localStorage using the template's
`persistence.rs` approach: synchronous, so the static can initialise
itself on first read and no letter can arrive before the mode is known.
Use namespaced key names. Leaving `NeedsReconcile` clears the reason.
`Connected` isn't persisted.

Add a single predicate, `sync_may_run()`. It is true only when the mode
is `Connected`, the hard lock (step 4) is off, and — from step 8 — the
current socket is session-ready. `restart_sync_stack()` writes the
persisted mode and then calls `location.reload()`; the in-memory signal
is not updated, so the gate stays closed. Also add `is_lonewolf()`
(connected and gated), as in the template's `conn_state.rs`.

Then wire the gate into every sync site in the real code:

- `handle_one` in the receiving file returns immediately when gated,
  before marking seen, applying or acking. The letter stays on the
  server and is pulled after resume. The recursive pull after
  `CreateNewPage` sits inside `handle_one`, so it is covered.
- The kick function in `db/main_send_unsent_love_letters.rs` does
  nothing when gated.
- `run_sender`'s inner loop re-checks before every `send_one`, so an
  in-progress loop stops.
- The worker still logs in when gated, but skips the initial
  `main_send_unsent_love_letters()`, bootstrap and the initial pull.
  Don't flip `START_OVER_FRESH_ATTEMPTED` when bootstrap is skipped.

One race needs a fix: `handle_one` can pass the gate check and then wait
on the write lock while a destructive verb takes it. When the verb's
closure ends, the stale letter would apply into the new data. Add one
gate check as the first line inside the closure that `apply_letter` runs
in `main_for_applying_love_letter_from_server_to_app.rs`. If gated it
returns a "skipped" outcome that behaves like the existing unknown-page
path: not marked seen, not acked.

**Done when.** With `needs_reconcile` set by hand in devtools
localStorage and the page reloaded, editing a block raises
`PENDING_COUNT` and nothing reaches the server log. A letter from a
second client isn't applied and the server still holds its unsent row.
Clearing the key and reloading drains and applies everything.

## Step 2 — Reconnect and re-login, lonewolf retry

**What and why.** Much of auto-connect already exists. `main.rs`
connects once the db and ids are ready, `core.rs` signals
`ConnStatus::Connected` on every (re)connect, and the receive worker
re-runs login each time, because each new server-side socket starts with
no user attached.

First, check the `Router` in `shared_internals_for_server`, which the
planning AI couldn't see. If it already retries a dead socket forever,
the offline retry (fires only while `Offline` or `FailedToConnect`)
already exists. Add nothing, and verify it by killing and restarting
the server. If it gives up, add a thin supervisor that retries every
few seconds and stops once the state leaves those two values.

Second, pull the worker's reconnect branch into one function,
`establish_session`, with a single-flight guard (a flag that makes a
second call return immediately while one is running). Today it does
login. From step 8 it also does the generation check and sets the
session-ready flag.

The **lonewolf retry** is a small helper. It runs `establish_session` up
to three times, two seconds apart, and is silent on success. It exists
because after the server says the dataset was replaced, this socket's
server-side stamp is stale (step 8), so a fresh login on the same socket
is needed before any verb can talk to the server. It never runs
concurrently with the offline retry, because it only fires on a
"dataset replaced" notice, which can only arrive on a live socket. Also
add `enter_lonewolf(reason)`: set `NeedsReconcile` with `ServerChanged`,
`clear_seen_everywhere()`, call a placeholder hook for the popup (the
human does the UI), then start the retry. Nothing calls it until step 8,
but expose it through the existing test-method macros so it can be
exercised by hand.

**Done when.** Killing and restarting the server brings the client back
to `Connected`. Calling `enter_lonewolf` from devtools gates sync,
re-logs-in on the same socket and leaves the client connected but gated.

## Step 3 — The data-ops page and the shared verb conventions

**What and why.** Verbs need a home. Add a dedicated page (new file in
`leptos_components/`, route in `main.rs`, a "Settings" button in
`menu.rs`). Per the decision, it is not a modal or a strip. For now it
is a heading plus one plain button per verb, added step by step; the
human styles it later.

Also add shared plumbing in the new folder:

- A single-flight busy signal holding an optional message. A verb that
  starts while another runs refuses.
- A verb wrapper that does the refusal checks first (holds the tab
  lock, session-ready if the verb needs the server, not hard-locked,
  not busy), then runs the verb under busy, and returns
  `Result<_, String>` with a user-presentable error.
- `ensure_tab_lock()`. Returns true if this tab already holds the Web
  Lock. Otherwise it tries `two_tab_guard::acquire()`, and verbs refuse
  if that fails. This covers a tab that never started the link (for
  example no server port), which would otherwise hold no lock at all.

The busy signal must also block input, because keystrokes typed during
a destructive verb would buffer for a doc about to be thrown away.
Check what `APP_LOCKED` in `shared_owners/ui_signals.rs` does. If it is
the app-wide input lock, set it alongside busy. Otherwise a minimal
full-screen blocker is required scaffolding, though its look is the
human's.

Finally, make `two_tab_guard::acquire` retry a few times about 200 ms
apart. After a reload the old page may not have released the lock yet,
and the new page would wrongly report `TwoTabs`.

**Done when.** The page opens from the menu, and a stub verb button
shows a busy state and refuses when pressed twice.

## Step 4 — Hard lock

**What and why.** A one-way, in-memory stop. If the same letter is
rejected by the server twice in a row while the socket is up, retrying
won't help, so all sync stops until the page is reloaded. Distinct from
`NeedsReconcile` (recoverable via verbs) and from offline (recoverable
by reconnecting). Add a locked signal and `lock_and_announce()` (sets
the flag, then calls one placeholder announce hook for the human) in the
new folder, and include the flag in `sync_may_run()`. Reload clears it.
Export is the only verb that still works while locked, and wipe, adopt,
reconcile and import refuse until reload.

In `send_unsent_love_letters_to_server_if_there_are_any.rs`, change
`run_sender`'s server-error branch. Today a failure breaks out and waits
for the next kick, with a popup at three failures. New behaviour: on the
first server rejection of a letter, retry that same letter once
immediately. A second rejection locks. Remove the three-failure popup.
Transport errors stay as they are (stop the loop and leave the row), and
don't count.

**Classify rejections before counting.** "Not logged in" doesn't count.
It triggers `establish_session` instead. The "dataset changed" rejection
from step 8 doesn't count either. It means `enter_lonewolf`. Put both
message strings in shared constants in `server_protocol`, like the
template's `ERR_*` strings, and have both sides use them. The client
currently matches `msg.contains("no such user")` by hand, so that is the
style to improve on.

**Done when.** Temporarily make the server reject one specific letter.
The client retries once, then locks and the sender stops. A second
client's letters are no longer applied, and a reload unlocks it (and
locks again if the rejection persists). A reconnect-time "not logged in"
rejection does not lock.

## Step 5 — Wrapping freeze_doc and the read-back into pure builders

**What and why.** The human has built `freeze_doc` and
`freeze_doc_builder` in the yrs crate. This step wraps them in the
target-agnostic builder crate
(`data_builder_for_operations_that_need_to_be_correct`), so Android
reuses it and everything can be unit-tested natively. Builders stay
pure and executors touch the db.

**Read the real signatures first.** Wherever this plan or the old plan
doc says `from_hard_copy` / `to_hard_copy_blocks`, the real names are
`freeze_doc` / `freeze_doc_builder` with `FreezeDocInput` and
`FrozenBlock`. `FrozenBlock` has no frac field. File order is the
order; import feeds blocks to `freeze_doc` in file order; the mirror
rows' frac values come from the rebuilt doc, never from the file.
Adapt the types and `prepare_import` to what is actually there. The
validation rules and the "builders stay pure" rule don't change.

Add the serde types for the backup file: a versioned file with pages and
backlinks; each page has its id, a main-menu flag and its blocks; each
block has id, content and meta. There is no stored "is title" and no
per-block main-menu flag. Derive them when writing mirror rows using the
rule the existing builders already use, and don't write a second
derivation. Every comparison uses order, never frac equality.

Add three functions:

- Read a page's block list from a blob (using `freeze_doc_builder`).
- Build a backup file from pages and blobs, skipping disabled pages
  entirely.
- `prepare_import(text)`, which validates and builds everything in
  memory and writes nothing. Validation: the text parses; the format
  version is 1 (anything else gets a clear message); page ids are
  unique; block ids are unique within a page; every page has at least
  one block; backlinks pointing at unknown pages are dropped and
  counted. For each page it calls `freeze_doc`, snapshots, and produces
  a `build_merge_new_page` plan. Any failure aborts before any db work.

Tests: export then import round-trips to equal ordered
(id, content, meta) lists, disabled pages are skipped, and each
validation rule rejects.

**Done when.** The native tests pass. Importing a hand-edited file with
a duplicate block id fails in `prepare_import` with a readable message.

## Step 6 — Export

**What and why.** Export works in any mode and needs no server. Add a
db helper that reads every page row with blobs (id, blob, status,
main-menu flag) in one query, and one that reads the backlinks table.
Make `page_status_is_active` in the receiving file `pub(crate)` so
export can decide which pages are disabled.

The verb flushes the batcher, runs `run_exclusive` containing only those
two reads, and leaves the lock. Then, outside the lock, it builds the
file from the in-memory blobs and serializes pretty JSON. A keystroke
landing after the flush is simply not in the file, which is fine. Put
the browser download (a Blob named `zakken-backup-<date>.json`) in its
own small function, because Android will write a file instead.

**Done when.** The file's pages and blocks match the menu, a disabled
page is absent, and export works while gated and in a `TwoTabs` tab.

## Step 7 — Import

**What and why.** Import replaces all local data from a backup. It is
destructive and the imported docs have new lineage, so it must leave the
client in `NeedsReconcile` (reason: imported) until a reconcile or adopt
resolves it. The server isn't touched, and the stored generation and the
ids in `key_value_storage` are left alone.

Order: (1) `prepare_import` on the text; (2) ensure the tab lock and set
`NeedsReconcile`; (3) flush, then one `run_exclusive` containing one
transaction: wipe the data tables, then `apply_merge_plan` for each
prepared page, then write the backlink rows; any error rolls back and
leaves the old data; (4) `restart_sync_stack()`. Use the executor's
existing `run_in_transaction` (the macro `start_transaction_if_not_in_one!`
joins an outer transaction, which is how several executor calls share
one).

Add `wipe_data_tables_unlocked()` in the db layer. It deletes children
first: block mirror rows, backlinks (if the table exists),
`client_love_letter`, `seen_server_letters`, then pages. Factor the
existing `delete_everything` so it and the new function share it, since
`delete_everything` takes the write lock itself. Pending letters are
intentionally discarded. Also clear the in-memory seen set.

**Done when.** After importing into a fresh profile, the menu shows the
pages, the mode survives a reload, nothing is sent, and a corrupt file
is rejected with the old data intact.

## Step 8 — Server dataset generation: work this out with the human

**The problem.** Today nothing tells a client that the server's dataset
was replaced. Wipe, adopt or a deleted DB file would let a client keep
sending letters built on old lineage. The server would merge them into
new data (duplicate blocks, or edits parked forever), and the client
would never notice. We need a way for the client to detect a replaced
dataset on login, for the server to refuse stale senders, and for
connected clients to learn.

Read the "Server" sections of `plan_for_general_data_management_ops.md`
as the starting sketch. They are a sketch only, and the template fakes
all of it, so **don't implement anything before the human answers these
questions.** Recommendations follow each question.

**What is a generation and where does it live?** Recommendation: a
decimal string of the current unix time in microseconds, stored in the
server's `key_value_storage` and mirrored in an in-memory atomic. It is
minted at startup if absent (next to `bump_session_id`) and re-minted by
wipe and adopt. Because it comes from the clock, even a deleted DB file
yields a new value, which gives the "DB deleted" scenario for free. This
needs an upsert helper next to `bump_session_id`'s delete-then-insert
pattern.

**How does a client learn it?** Recommendation: the login reply changes
from `Ok(())` to a payload with the generation, and wipe and adopt reply
with the same payload. Every decoder must change together. That means
`try_login` in `main_for_login_and_registration.rs`, `mock_client_core`,
and anything else that decodes login (grep for it). The decision is
whether to accept a wire break. Client and server ship together, so the
recommendation is to accept it.

**How does the server stop stale senders?** Recommendation: the
telephone stamps each socket with the generation returned at login, and
`do_request` refuses every request except `Register` and `Login` when
the socket's stamp differs from the current generation, with the shared
"dataset changed" error from step 4. Guard `Ack` and pulls too, not just
`SendLoveLetter`: a stale ack could delete a fresh unsent row if ids get
reused after a table wipe, and a stale pull would hand new-lineage
letters to an old client. Wipe and adopt re-stamp the caller's socket
with the new value.

**How do other connected clients find out: close their sockets or push a
message?** What is already decided (from the human's decisions list) is
that the lonewolf retry fires when the client receives a "you are now
lonewolf" message from the server — so a pushed message exists as a
concept. The *mechanism* is not settled. Two options, and the human
picks:

- **Push.** Add a `Response` variant for it. Widen the telephone's
  per-socket channel (today it carries only "go pull") to a small enum,
  and have the socket loop in `socket.rs` send the new message when it
  arrives. Extend the `Router` with a second callback for it. Stale
  sockets stay open and are told to re-login (the step 2 retry).
- **Close the other sockets.** On wipe/adopt, the server closes the
  other sockets. The client's normal reconnect re-logs-in, and the
  generation check at login catches it. No protocol or telephone
  changes. The cost is that the client goes briefly Offline instead of
  staying connected, and the human has said they want it to stay
  connected, so push is probably right. But it is a step 8 question.

A client that was offline learns at its next login either way.

**Session-ready flag.** A stale client reconnecting will get a fresh
server-side stamp at login, so the server guard alone does not protect
against a send slipping through between "login returned" and "the
client's own generation check done". Add a per-socket session-ready flag
that means "login AND generation check both completed". It goes false on
any `ConnStatus` other than `Connected` and at the start of each
reconnect. Sending requires session-ready as well as `sync_may_run()`.
The hard lock (step 4) doesn't need it: the rule that "not logged in"
doesn't count for the lock is enough on its own.

**Order inside wipe and adopt?** Recommendation, in order: validate
everything first (adopt decodes every snapshot and returns an error
before mutating anything), mint the generation, notify or close other
sockets (after dropping the std mutex guard, never hold it across an
await), wipe the three tables, then for adopt insert each page through
the existing `apply_letter` `CreateNewPage` path so the server stores
the client's bytes as given. Wipe the tables with the existing
`delete_all_rows`, never `delete_row_where` with All, which the builder
refuses. Order: unsent letters, letters from clients, pages. Keep
registered users and `key_value_storage`. Requests are serialized by the
queue executor, so these are atomic against other requests. Use a
transaction if the db wrapper supports it. If not, partial failure is
recoverable, because adopt starts with a wipe and the client still has
everything.

**What does the client do on a mismatch?** Recommendation, inside
`establish_session`: after a successful login, compare with the stored
`dataset_generation` in the client's key-value table. No stored value
(new or pre-feature client) means store the server's and continue. Equal
means continue. Different means lonewolf handling: `NeedsReconcile` with
`ServerChanged`, `clear_seen_everywhere()`, one popup hook only on the
transition, and don't store the new value. Keep the login's value in a
`CURRENT_LOGIN_GENERATION` static for the verbs. The stored value is
written only when wipe, adopt or reconcile completes.

**Other questions.** Adopt sends all pages in one websocket message
(JSON with base64 inflation). Check the axum max message size, and note
chunking as future work. For the mock client, keep it compiling with the
new login reply. Its staleness mode from the old plan is optional, since
the e2e plan favours a second real browser.

**Done when (after the design is settled and built).** A second browser
logged in as another user receives the "dataset replaced" message (or
reconnects, if the human picks that option) when the first wipes,
becomes lonewolf, and a fresh login on the same socket succeeds.
Deleting the server DB file and restarting makes a client detect a
mismatch at its next login instead of silently continuing.

## Step 9 — Wipe server

**What and why.** Wipe empties the server's dataset and keeps local
data. Client side, after the refusal checks: set `NeedsReconcile`
(reason: you wiped), send the wipe request, and on success (in one
`run_exclusive`) delete all `client_love_letter` rows,
`clear_seen_everywhere()`, store the new generation in the key-value
table, and set `CURRENT_LOGIN_GENERATION` too. The pending letters must
go. They describe edits to pages the server no longer has, and sending
them later would be rejected and eventually hard-lock the client. Local
pages are untouched and no reload is needed, because the mode stays
`NeedsReconcile`.

On failure keep `NeedsReconcile` and set the reason to wipe incomplete,
so the user can rerun. Updating `CURRENT_LOGIN_GENERATION` matters
because the caller's socket was re-stamped server-side, and reconcile's
finish step stores whatever that static holds.

**Done when.** A second browser goes lonewolf, the server tables are
empty, and the wiping client stays connected but gated with its pages
intact.

## Step 10 — Adopt

**What and why.** Adopt says "my data is the truth": the server discards
its dataset and takes this client's blobs byte for byte. Because the
server stores the same bytes, both sides end up in one lineage with no
rebuilding. Flow: set `NeedsReconcile` (reason: adopt incomplete) and
show busy. Flush, then under `run_exclusive` read all local pages, drop
disabled ones (the server starts them with empty status, and a
local-only disabled page is later ignored by reconcile), and capture id,
flag and blob. Send the adopt request.

On error leave the mode as it is and tell the user the server may be
partly written and that adopt is safe to rerun. On success, in one
`run_exclusive`, first verify the local blobs still equal the captured
ones. If they changed, abort with a message and stay gated. Then delete
`client_love_letter` rows, `clear_seen_everywhere()`, store the new
generation and update `CURRENT_LOGIN_GENERATION`, set `Connected`, and
call `restart_sync_stack()`. The client's own docs are never rebuilt.

**Done when.** After adopt, the second browser (lonewolf) can reconcile
and edits made on both sides afterward converge. The adopting client's
menu is unchanged after the reload.

## Step 11 — Reconcile

**What and why.** Reconcile brings a gated client and the server back
into one lineage per page, letting the user decide conflicts. It is one
running process (a single async verb), not a plan object. The client is
the driver and the server only answers small questions. The only state
kept is inside that running verb: previous mode and reason (for cancel),
the local blobs captured at the start (for a final check), and what's
been done so far.

### 11a. Small server additions (human designs payloads)

Two new request kinds, no composite request:

1. A **compare** request, where the client sends every local page's id
   plus a fingerprint of its blob and of its status blob, and the
   server replies per page with same, differs, or not-on-server, plus
   the ids of pages the server has that the client didn't list.
2. A **get one page** request returning blob, status and main-menu flag
   (today only get-all exists).

Put the fingerprint function in `server_protocol`, since both sides must
compute identical values. **Fingerprint = SHA-256 of the raw blob bytes
(and of the status bytes).** Byte-identical always means the same
lineage, which is the whole point. The alternative — hashing content
instead of bytes — can say "same" for two pages that look identical but
have different lineage, and keeping the local blob then breaks later
edits. (The server *does* have yrs, since `apply_letter` already decodes
docs. The reason for byte hashing is lineage safety, not a lack of yrs.)

The downside of byte hashing is that two synced replicas might encode to
slightly different bytes, giving false "differs" results. Those cost one
extra fetch and a silent replace in 11b, which is harmless. Add a test
checking whether two synced replicas encode identically. If
over-reporting turns out to be common, upgrade the fingerprint to a hash
of the yrs state vector plus delete set, which is exact. Don't build
that upfront.

The guard from step 8 applies to both requests.

### 11b. What the verb does per page

- **Byte-identical blob and status**: nothing.
- **On the client only**: push it by sending `CreateNewPage` with the
  local full blob (the server stores it as given), followed by a
  `DisablePage` with the local status blob if the page is disabled. A
  local-only page that is disabled is ignored and not pushed. Default is
  to push. Offering "discard locally instead" per page is an optional UI
  extra.
- **On the server only**: fetch it and create it locally from the server
  blob. If the server's status says disabled, ignore it.
- **Differs**: fetch the server's version, then branch. If the page is
  disabled on either side, skip the merge UI. Take the server's content
  blob and converge the disabled state both ways (send `DisablePage`
  with the local status if local is disabled, merge the server's status
  locally via `build_merge_disable` if the server is disabled). Disabled
  always wins and is a tombstone. Otherwise compare ordered
  (id, content, meta) lists. If they are equal, silently replace the
  local blob with the server's (identical content means nothing is
  lost, and the lineage question disappears). If they differ, it is a
  conflict for the user.

### 11c. Merge UI with web_diff_viewer

The viewer works on lists of block texts and returns the right side's
texts in visual order. Layout: **left = server texts (frozen
reference), right = local texts (editable, result)**. Reading left to
right matches "old vs new", and pressing Done without touching anything
keeps the user's own work. The result is built on the server's blob, so
an untouched right side equals "keep mine" for the whole page.

The edit-script builder diffs the final texts against the server texts,
so it works the same either way. "Use server for all" bypasses the
viewer and uses the server texts. "Use mine for all" is the untouched
right side. If the viewer lacks a per-row "take left version of this
row" action, adding one is a small viewer change for the human — flag it
in the step report.

The verb awaits the user without any plan object. When it hits a
conflict it puts the two text lists into a "current conflict" signal and
awaits a oneshot receiver. The UI sends back the final texts (or a
cancel) through the paired sender.

Check first that `block_diff_viewer_leptos` and the real app use the
same Leptos version, and decide how `web_interface` depends on the
`web_diff_viewer` crates (a path dependency).

### 11d. The pure edit-script builder (builder crate, with tests)

Input: the server blob, the final text list, the current time and user
id. It loads the blob into a doc, takes a bookmark (`create_bookmark`),
then makes the doc's ordered texts equal the final list.

To work out which blocks were kept, edited, moved, inserted or deleted,
run `block_text_diff::rows_for_two_pages` between the server's texts and
the final texts, since the viewer only gives strings back. Kept blocks
stay as they are. Edited blocks get a text edit through the same
`text_diff::get_diff` path `edit_title_for_page` uses (so offset
conventions match; fall back to one whole-text replace). Deleted blocks
are removed. Inserted blocks get fresh ids and a default meta (look at
what `add_block` uses). Moved blocks use `move_block` (skipping blocks
that are part of the longest unmoved run is an optional optimisation).

The output is a diff snapshot against the bookmark
(`generate_diff_snapshot`) plus the doc's full final blob. Meta
differences between local and server aren't visible in the viewer, so
the server's meta wins for kept blocks. That is an accepted limit. If
the final texts already equal the server's, the result is "no edit", so
send nothing and just replace local with the server blob.

Tests: merging the diff snapshot into a doc built from the server blob
yields exactly the final texts with no duplicate ids; reorder, delete,
insert and edit cases; and the key lineage test. Take a doc built by
`freeze_doc` with the same content as a server doc, replace it with the
server's blob, make a local edit, and confirm that edit's snapshot
merges into the server doc with no pending state.

### 11e. Applying, network before local

Before sending anything, check the connection and session-ready. Sends
bypass `client_love_letter` and the gate, so factor `send_letter_direct`
out of `send_one` (same payload building and reply decoding). The
client-wins result goes as a "generic merge letter": an edit-block
letter with an empty text edit and empty block id, plus the diff
snapshot. `apply_bootstrap_row` in the receiving file already builds
this shape, and receivers ignore everything but the snapshot, so other
clients pick the change up as a normal letter. The first transport or
server error aborts. Nothing local has changed, so rerunning is safe.

The local phase is one `run_exclusive` with one transaction, using
builders plus `apply_merge_plan`. First verify each affected local
page's blob still equals what was captured at the start (else abort with
"local data changed, reconcile again"). Then `upsert_page_from_blob`
for every page whose local copy changes.

Add that helper in the db layer. If the page doesn't exist, use
`build_merge_new_page` plus `apply_merge_plan`. If it exists with the
same main-menu flag, use `build_merge_blob` with a fresh empty boss and
the new blob as the snapshot, passing the current mirror rows as the
existing blocks, so the mirror is replaced in place. If the flags differ
(essentially never), delete the page's block rows, its backlink rows and
its page row, then rebuild with `build_merge_new_page`. Never merge a
blob into the existing local boss, because that is the lineage union.
Check `merge_page.rs` to confirm `build_merge_blob` only uses the boss
it is given and doesn't re-read the old blob from the db. If it does,
use the fallback everywhere. The local main-menu flag always takes the
server's value, except for pages pushed to the server.

Whether to apply page by page or collect and apply at the end is the
human's choice. Pages are independent, so both work. Per-page survives a
crash better, and collecting allows one transaction.

### 11f. Finish and cancel

Finish: delete all `client_love_letter` rows (every pending edit is
either already in a blob that matches the server, or obsolete),
`clear_seen_everywhere()`, store `CURRENT_LOGIN_GENERATION` as the
stored generation, set `Connected`, and call `restart_sync_stack()`.
After the reload the worker pulls the letters the server queued
meanwhile, and applying them is idempotent. On any failure the mode
stays `NeedsReconcile` with reason reconcile incomplete. Cancel before
the local phase restores the previous mode and reason and, if that means
returning to `Connected`, reloads.

**Done when.** See step 12.

## Step 12 — End-to-end checks with two real browsers

Use two browser profiles with different usernames, because the two-tab
lock stops two tabs in one profile. These replace the old mock-client
test list.

1. A wipes the server. B receives the push (or reconnects, whichever
   step 8 chose), becomes lonewolf, its edits queue and nothing is
   sent, and A's wipe leaves A's own data intact.
2. A adopts. B reconciles. B's pages come back as byte-different but
   content-identical (silently replaced) or as conflicts (viewer).
   Afterward, edits on both sides converge in both directions.
3. A exports, B (fresh profile) imports and reconciles against the
   existing server. Identical pages take the server blob and edits flow
   both ways.
4. Edit the same page on both sides while one is gated, then reconcile
   choosing "mine" once and "server's" once.
5. Delete the server DB file and restart. The client reports a mismatch
   at its next login and is gated, not silently syncing.
6. Provoke the hard lock as in step 4, and confirm export still works
   while locked.

If a page's edits stop arriving after any scenario, suspect lineage
first: look for a path that merged one doc into another of different
lineage.

---

## Out of scope, and one thing you may trip over

The audit findings, the e2e harness, connection-state UI and the exact
wire shapes are not part of this plan.

One existing behaviour you'll probably see: the receive worker shows a
popup whenever bootstrap refuses because local pages already exist, and
a reload after any verb will trigger it. That is existing behaviour, so
don't fix it as part of this.

## What the planning AI couldn't see and would want to

- `shared_internals_for_server` (the Router, `socket_web.rs`): retry
  behaviour and where the new callback goes.
- `Telephone` `commands.rs` and `task.rs`.
- `db/main_send_unsent_love_letters.rs`, `db/write_lock.rs`, and the
  batcher's `exclusive.rs`.
- `merge_page.rs` (the builders), `apply_merge_plan` and its transaction
  helpers.
- `main_for_applying_love_letter_from_server_to_app.rs`, the
  `reads_wrapper` modules and `key_value_storage_db_actions`.
- `expose_methods_for_testing`, plus how `APP_LOCKED` is used.
- `yrs` `snapshots.rs` and `YrsActivePages`, the `love_letter` crate,
  and `block_text_diff`'s public API.
- `mock_client_core`.

---

## Prompt to hand to the implementing AI

You are implementing a feature in the ZakkeN project together with a
human, following the attached implementation plan. The plan is the
source of truth. Read it fully before touching anything.

Read first, in this order: the plan; `ZakkeN/STORED_AI_CONTEXT.md`;
`ZakkeN/coding_style.md`; `plan_for_general_data_management_ops.md`
(reference only: where it disagrees with the plan, the plan wins, and
sections about classify_pages, diff_page, build_client_wins_update,
pause/resume and socket kicking are obsolete);
`connecting_to_server_template/` and its `behaviour_decisions.md`
(reference for SHAPE only; do not port the fake server, power/latency,
internet_switch, "another client did this" buttons, fault injection, or
client_server_coms).

How to work:

- Do ONE step at a time, in the plan's order. At the end of each step,
  stop and report: what changed (files), how the plan's "done when"
  check can be performed, and anything uncertain. Wait for the human
  before continuing.
- Before editing any file you haven't read, read it. The plan lists
  files the planning AI never saw. When a step depends on one, read it
  first and adjust to what is really there. If something the step needs
  isn't available, say which file you need. Don't guess.
- Step 8 (server dataset generation) is a design discussion. Do NOT
  implement it until you have walked the human through the plan's
  questions and they have answered. Then implement exactly what was
  agreed.
- The yrs primitives `freeze_doc` and `freeze_doc_builder` already
  exist. Read their real signatures in the `yrs` crate before writing
  step 5. Do not rewrite them.
- The human builds all UI. You only expose signals, async verb
  functions returning `Result<_, String>`, and clearly named
  placeholder hook functions where a popup or overlay would go. Don't
  decide layout, text or styling.
- Do not fix, mention, or work around the known audit findings. If a
  step trips on a known bug, say "known bug, not fixing" and move on.
- Pure logic goes in the builder crate with native unit tests;
  executors and db access stay in wasm-only code.

Invariants to check whenever something misbehaves:

1. Never merge across lineages (a doc rebuilt from text is a new
   lineage). One side takes the other's blob, or builds edits on top of
   it.
2. Set `NeedsReconcile` before any destructive step; leave it only
   after the last step succeeds.
3. Destructive local writes: flush the batcher, then one
   `run_exclusive` closure, then one transaction. No network inside.
   Never call `merge.rs::apply_letter` (or anything that calls
   `run_exclusive`) inside a closure (deadlock). Use pure builders plus
   `apply_merge_plan`.
4. Any transition into `Connected` ends in `restart_sync_stack()` (page
   reload), and nothing runs between the last destructive write and
   the reload.
5. Any dataset-generation change calls `clear_seen_everywhere()` (db
   table and in-memory `SEEN_IDS`).
6. Gating goes through the single `sync_may_run()` predicate, and the
   hard-lock count ignores transport errors, "not logged in" and
   "dataset changed".

Keep each report short and concrete. Ask the human whenever a decision
isn't covered by the plan.

---

## Amendments (2026-11-04)

1. **Naming.** Everywhere the plan says `from_hard_copy` /
   `to_hard_copy_blocks`, use the real `freeze_doc` /
   `freeze_doc_builder` / `FreezeDocInput` / `FrozenBlock`. Backup
   blocks have no frac. File order is the order. Mirror-row fracs come
   from the rebuilt doc.
2. **Step 8 push message.** The lonewolf message is decided as a
   concept. The mechanism (Response variant, widened telephone channel,
   second Router callback) is a recommendation. Raise the cheaper
   alternative (server closes other sockets, client reconnects, login
   generation check catches it) with the human too. Do not implement
   before they choose.
3. **Step 11a fingerprint.** SHA-256 of raw blob bytes and status
   bytes, in a shared function in `server_protocol`. Add a test checking
   whether two synced replicas encode identically. If over-reporting
   turns out to be common, propose upgrading to a hash of state vector
   plus delete set; do not build that up front. The server DOES have
   yrs; lineage safety is the reason for byte comparison.
4. **Retiring flag dropped.** Transitions into `Connected` write the
   persisted mode only, not the in-memory signal, then reload. The
   `apply_letter`-closure gate check stays. `session-ready` moves into
   step 8 and means "login AND generation check completed on this
   socket". Step 4 still ignores "not logged in" and "dataset changed"
   for the lock count.
5. **Viewer direction.** Left = server texts (frozen reference), right
   = local texts (editable, result). Edit-script builder unchanged.
   "Use server for all" bypasses the viewer. "Use mine for all" is the
   untouched right side. Flag to the human that a per-row "take left"
   action may be needed in the viewer.
