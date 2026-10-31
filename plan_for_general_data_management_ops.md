# Plan: general data management operations

2026-10-28

This is the canonical plan. The earlier pre-plan
(artifacts/28_preplan.md) is superseded and deleted.

Section 1 is the design. Sections 2 and 3 are addenda that resolve
questions the design left open and, where they differ, override it.

---

## 1. Design

### TASK: implement the "general data management operations" feature for ZakkeN: export, import, pause/resume, wipe server, adopt, reconcile.

Web client is cqrs_state_synchronization/web_interface (Leptos/wasm);
server is zakken_server. Android comes later, so put logic that
doesn't touch browser/DB in target-agnostic code (the builder crate
`data_builder_for_operations_that_need_to_be_correct` is the right
home: builders stay pure, executors touch the db). This document is
the full design. UI is NOT the writer's job (see "UI contract" for
what to expose). Writing `BossOfYrs::from_hard_copy` is the human's
job; assume it exists with signature (page_id, user_id, time, blocks)
-> Result<Arc<BossOfYrs>, YrsError>, inserting blocks sorted by (frac,
id) using insert_new_block_with_preexisting_id at AtEnd.

If a step turns out impossible because of code you read that I hadn't
seen, use the stated fallback; if there is none, stop and report.
Never invent a workaround that merges data across lineages (defined
below).

### READ FIRST (I designed this without seeing these)

1. server `Telephone` (telephone.rs): per-socket registry
   (register_socket, attach_name, user_of, notify_socket, notify_all,
   remove_socket). I need: (a) a way to close every socket except one,
   (b) a per-socket "generation at login" stamp. Known fact from
   socket.rs: a socket's run loop exits when its notify receiver
   yields None, i.e. when the telephone drops that socket's sender. So
   "kick" = remove those sockets' entries from the telephone (make
   sure remove_socket is idempotent, because socket_main also calls it
   on exit).
2. `Router` in shared_internals_for_server and the worker in
   receive.rs: does login re-run on every new socket after a
   reconnect? run_worker currently logs in once. The server attaches
   the user name per socket, so a reconnected socket without a fresh
   login fails every request with "not logged in". If re-login doesn't
   already happen, add it: on each ConnStatus::Connected after the
   first, the worker re-runs login + generation check (and, if mode is
   Connected, kick_send + pull_unsent). A second channel the worker
   selects on is fine. Wipe/adopt kicks depend on this.
3. How executors open/commit a transaction (macro
   start_transaction_if_not_in_one!, web_internal_db db_helper
   begin/commit/rollback). Verbs need ONE outer transaction around
   several executor calls. Same question for the server's LiveForever
   (transactions? delete_all_rows? otherwise delete_row_where with
   SelectArgument::All).
4. `run_exclusive(batcher, closure)` in complicated_signal_flow: it
   flushes the batcher, takes WRITE_LOCK, runs the async closure,
   releases. merge.rs::apply_letter and friends call run_exclusive
   internally, so calling them while already holding the lock
   deadlocks. In verbs use the pure builders (build_merge_new_page,
   build_merge_blob, build_merge_disable) plus apply_merge_plan
   directly inside ONE run_exclusive closure.
5. Anything else that decodes Login's reply (mock_client_core, etc.):
   grep and update.

### THE CORE PROBLEM: LINEAGE

yrs updates reference items by (client_id, clock). A doc built from
scratch (from_hard_copy, any fresh Doc) has new items: a different
LINEAGE from the doc it was copied from, even with identical content.
An update from one lineage never integrates cleanly into another: if
it depends on missing items yrs parks it as pending forever (edits
silently never arrive); if not, merging two independent copies of the
"same" page unions them and gives duplicate block ids.

Rules that follow:

- Never merge a page snapshot/update into a doc of a different
  lineage.
- Two sides agree only if one side takes the other's actual blob, or
  builds edits on top of the other's blob.
- from_hard_copy is used in exactly one place: import. Imported data
  must go through adopt or reconcile before sync resumes.
- Do not build "deterministic seed docs". Not needed with this design.

### INVARIANTS (every piece of code should preserve these)

I1. After any verb finishes, each page exists in one lineage across
    client and server (or the client is gated).

I2. "Gated" (sync mode != Connected) means: nothing is sent, nothing
    received is applied or acked, no bootstrap, no pull_unsent.

I3. Every destructive local write is flush -> one run_exclusive
    closure -> (inside) one db transaction. No network I/O inside the
    closure. Verbs do network first, local writes last.

I4. Any transition INTO Connected ends with a page reload (put reload
    behind one function `restart_sync_stack()` so Android can swap
    it). Transitions out of Connected take effect immediately via
    gating and need no reload. After a destructive local phase,
    trigger the reload right after run_exclusive returns and write
    nothing else in between. While the verb is running, the UI shows a
    blocking overlay so no keystrokes buffer in the batcher for a boss
    that is about to be thrown away.

I5. Any dataset-generation change clears seen_server_letters in the db
    AND the in-memory SEEN_IDS in receive.rs (add one
    `clear_seen_everywhere()`).

I6. Server: every wipe/adopt mints a new generation first, kicks all
    other sockets, and the server rejects SendLoveLetter from sockets
    whose login-generation != current.

I7. Destructive verbs (import, adopt, wipe, reconcile apply) run only
    in the tab that holds the two-tab lock. Export may run anywhere.
    If after a reload the tab shows TwoTabs spuriously (old tab hasn't
    released the lock yet), add a few short retries in
    two_tab::acquire.

### SYNC MODE (replaces the boolean "disconnected flag")

Three states, persisted in client key_value_storage under key
"sync_mode" ("connected" | "paused" | "needs_reconcile"), cached in a
static ArcRwSignal (readable synchronously from the router callback;
set = write storage first, then update the cache). Load it at the top
of start_link, BEFORE Router::new, so no letter can arrive before the
mode is known. Provide is_gated() = mode != Connected.

- paused: user pressed Disconnect. The user may resume freely (set
  connected + reload).
- needs_reconcile: local docs and the server's dataset can't be
  assumed to share lineage. Resume is refused; only reconcile / adopt
  / wipe (and export) are allowed. Entered by: import, wipe, login
  generation mismatch, and the start of reconcile/adopt (if they fail
  midway the user simply reruns them). Left only by successful
  reconcile/adopt (-> connected + reload). Cancelling reconcile before
  apply restores the previous mode and reloads.

A manual pause where the server's generation changed in the meantime
is upgraded to needs_reconcile by the login check on the next boot.

Gating points (all of them; the pre-plan listed only two):

 1. the Router on-letters closure in core.rs: drop the letter if
    gated;
 2. kick_send: no-op if gated;
 3. run_sender's inner loop: re-check the mode before every send_one
    (a loop already in progress must stop);
 4. run_worker: login still runs (verbs need an authenticated socket),
    but skip bootstrap and the initial pull_unsent when gated;
 5. handle_one: if gated, return immediately without marking seen,
    applying, or acking (the letter stays on the server and is pulled
    after resume);
 6. the recursive pull_unsent after CreateNewPage inside handle_one.

Local edits keep working while gated; they accumulate in
client_love_letter (PENDING_COUNT shows it). Expose the mode signal so
the human can style the border. Known cost: while gated the server
keeps re-pushing the unacked backlog on each notify; accepted for v1.

### DATASET GENERATION

- Server: key "dataset_generation" in key_value_storage, stored as a
  decimal string. Value = current unix time in microseconds (monotonic
  enough, and unique even if the whole server DB file is deleted, so a
  stale client can't collide). Minted at startup if absent (next to
  where bump_session_id is called); re-minted on wipe and adopt. Also
  keep it in an in-memory AtomicI64 updated whenever it is
  minted/loaded, for the SendLoveLetter guard. Add a generic upsert
  helper to operations_key_value (bump_session_id already does
  delete+insert; reuse that pattern).
- Login reply changes from Ok(()) to
  Ok(DatasetGenerationPayload{dataset_generation: i64}). Wipe and
  adopt replies use the same payload type. After a successful login
  the server stamps the socket (in the telephone) with the generation
  it just returned.
- Client: after identity login succeeds (after the NoSuchUser/register
  dance), compare to the stored "dataset_generation":
   * no stored value (fresh or pre-feature client): store the server's,
     continue;
   * equal: continue;
   * different: set mode needs_reconcile, clear_seen_everywhere(), show
     one popup (only on the transition) saying the server's dataset
     changed and sync is paused until Reconcile/Adopt. Do NOT store the
     new generation yet.
  Keep the server's value in a static CURRENT_LOGIN_GENERATION for the
  verbs. Keep the existing NoSuchUser clear-seen stopgap; it's
  harmless.
- The new stored generation is written only when
  reconcile/adopt/wipe completes (from CURRENT_LOGIN_GENERATION or from
  the verb's reply).

### PURE LAYER (target-agnostic; unit-test it)

Types (serde): HardCopyFile{format_version: u32 = 1, pages, backlinks};
HardCopyPage{page_id, is_main_menu_page: bool, blocks};
HardCopyBlock{block_id, content, meta, frac_position};
HardCopyBacklink{page_that_holds_link_id, page_being_linked_to_id}.
Blocks are in (frac, id) order. There is NO is_title and NO per-block
is_part_of_main_menu_page: derive them when writing mirror rows with
the same rule the existing builders use (title = first block by frac
order; block's main-menu flag = the page's flag). Do not write a
second derivation.

Frac strings are informational only (export, and ordering on import).
Rebuilt docs get new frac strings, so every comparison uses ORDER,
never frac string equality.

BossOfYrs addition (you write it, in yrs_wrapper.rs next to
get_entire_page): `to_hard_copy_blocks` returning blocks with text,
meta, id, frac sorted by (frac, id). It's a thin variant of
get_entire_page that keeps the frac read_block_at already returns. Add
the uniffi record needed.

Functions to write in the builder crate:

1. page_blocks_from_blob(blob, page_id, user_id, time) ->
   Vec<HardCopyBlock> via doc_from_snapshot + to_hard_copy_blocks.
2. diff_page(client_blocks, server_blocks) -> Option<PageDiff> (None if
   the ordered (id, content, meta) lists are equal). is_main_menu_page
   is ignored in comparisons (it can't be changed through letters).
   PageDiff carries what the UI needs: page_id; client_title and
   server_title (first block content on each side); counts
   {only_client, only_server, changed, moved}; and rows aligned by
   block_id, each row {block_id, status: Same|Changed|OnlyClient|
   OnlyServer|Moved, client: Option<{content, meta}>, server:
   Option<{content, meta}>}. Row order: walk the server order, and
   insert client-only rows right after their client-order predecessor
   (top if none). "Moved" = a common block not in the longest common
   subsequence of the two orders; the same LIS is reused by the replay
   below.
3. classify_pages(local_pages, server_pages) -> ReconcilePlan. Inputs
   per page: id, is_main_menu_page, blob, status blob, active (use the
   existing page_status_is_active helper from receive.rs; make it
   pub(crate)). Output buckets (ids), plus blobs kept in a non-UI side
   structure:
   * take_from_server: server-active, not local;
   * push_to_server: local-active, not on server;
   * identical: both active, diff_page == None;
   * conflicts: Vec<PageDiff>;
   * disable_locally: server says disabled, local active (also: server
     disabled and absent locally = ignore, don't recreate);
   * disable_on_server: local disabled, server active;
   * local-disabled and absent on server = ignore.
   Disabled always wins; it is a tombstone and can't be undone.
4. build_client_wins_update(server_blob, client_blocks, page_id,
   user_id, time, session_id) -> (snapshot_of_edit, final_blob).
   Replays the client's content on top of the SERVER's lineage:
   a. W = doc_from_snapshot(server_blob); bookmark = create_bookmark(W).
   b. delete_block every block W has that the client doesn't.
   c. For common blocks with different content: apply the text change
      using the same text_diff get_diff + builder edit_block path that
      edit_title_for_page uses (so offset conventions match the rest
      of the app; the returned ctx is discarded, we only want W
      mutated). Fallback: one whole-text Replace at position 0. For
      differing meta: edit_text_block with EditTarget::Meta and a
      whole-string Replace.
   d. insert client-only blocks with
      insert_new_block_with_preexisting_id at AtEnd, in client order.
   e. enforce client order: walk i over the client order; if W's
      current order[i] != target[i], move_block(target[i], above =
      target[i-1] or None, below = W's current order[i]). Optional
      optimization: skip blocks in the LIS of common blocks.
   f. snapshot_of_edit = generate_diff_snapshot(W, bookmark); final_blob
      = W.snapshot().
   Test: merging snapshot_of_edit into a doc built from server_blob
   must yield the client's ordered (id, content, meta) list, with no
   duplicate ids.
5. prepare_import(file_text) -> Result<Vec<PreparedPage>, String>
   (validation + build, all in memory, nothing written). Validation:
   parseable; format_version == 1 (reject others with a clear message);
   page ids unique; block ids unique within a page; every page has >=
   1 block (first block is the title); drop (and count) backlinks
   pointing at unknown pages. For each page: from_hard_copy ->
   snapshot -> build_merge_new_page plan. Any failure here aborts
   before any db write.
6. build_hard_copy(pages_with_blobs, backlinks) -> HardCopyFile (skips
   disabled pages entirely, per the original spec).

### SERVER CHANGES

Protocol (server_protocol): append Request::WipeDataset and
Request::AdoptDataset (append at the end of the enum). Add
DatasetGenerationPayload{dataset_generation: i64};
AdoptDatasetPayload{pages: Vec<AdoptPage>};
AdoptPage{page_id, is_main_menu_page: bool, snapshot: Vec<u8>}.
Re-export in lib.rs; fix the exhaustive matches the compiler flags
(request_name in logging, do_request). GetAllYrsPages is unchanged.

db crate: new operations_dataset module: get/ensure/mint generation
(see above); wipe_dataset_tables (delete every row of pages,
love_letters_from_clients, unsent_love_letters; keep registered_users
and key_value_storage); adopt_pages.

adopt (server) order: (1) decode/validate EVERY snapshot first
(doc_from_snapshot) and return Err before mutating anything if any
fails; (2) mint the new generation; (3) kick other sockets (async, in
server_core, after dropping the std mutex guard; never hold the guard
across an await); (4) wipe the three tables; (5) for each page call
the existing apply_letter(LoveLetter::CreateNewPage{page_id,
is_main_menu_page, snapshot_of_edit}), which runs the existing
create_page: it stores the blob as given, computes the version
bookmark, and creates a fresh empty page_status. Do NOT rebuild docs
on the server. Use a transaction for steps 4-5 if LiveForever has one;
if not, partial failure is recoverable because the client keeps all
its data and adopt is idempotent (it starts with a wipe).

wipe (server): mint, kick, wipe the three tables, reply with the new
generation.

Requests are serialized by the QueueManager executor, so wipe/adopt
are atomic relative to all other requests.

Handlers: new files allowed_requests/wipe_dataset.rs and
adopt_dataset.rs, wired into do_request the way the others are. They
need telephone and socket_id.

SendLoveLetter guard: in do_request (or accept_letter), if the socket's
login stamp != the current generation, return
Err(IllegalInput("dataset changed; log in again")). Reason: a letter
already queued from a kicked socket would otherwise merge an
old-lineage snapshot into the new dataset (for CreateNewPage on an
existing page this duplicates blocks).

Login handler: reply with the generation and stamp the socket after
attach_name. Backlinks are NOT sent or stored on the server (no table
exists; nothing writes backlinks today).

### CLIENT PLUMBING

- New module general_data_management_ops/ in web_interface: mode.rs,
  shared.rs (read all local pages incl. blobs; busy-overlay signal;
  restart_sync_stack; clear_seen_everywhere), export.rs, import.rs,
  wipe.rs, adopt.rs, reconcile.rs.
- A global ROUTER handle (static Mutex<Option<RouterHandle>>), set in
  start_link, so verbs can send requests. Verbs that need the server
  fail with a clear error if CONN_STATE != Connected or there is no
  CURRENT_LOGIN_GENERATION.
- identity::login returns the generation (Result<i64, String>) and the
  check above runs right after it, before load_seen_from_db.
- send.rs: factor out `send_letter_direct(router, LoveLetter) ->
  Result<(), DbError>` (build payload via love_letter_to_bytes +
  SendLoveLetterPayload, same reply decoding as send_one). Used by
  reconcile; it bypasses client_love_letter.
- A new db helper that reads all page rows with blobs (page_id,
  blobbed_page, page_status, is_main_menu_page) in ONE query, and one
  that reads the backlinks table (two text columns).
- A wipe_data_tables_unlocked() (children first: every_block_in_existence,
  backlinks if the table exists, client_love_letter, seen_server_letters,
  then pages). The existing delete_everything takes WRITE_LOCK itself,
  so factor it so both share the unlocked part.
- Replacing a page locally (`upsert_page_from_blob(page_id, is_main,
  blob)`, used inside the verb closures): if the page doesn't exist,
  build_merge_new_page + apply_merge_plan. If it does exist, preferred:
  build_merge_blob with a fresh EMPTY boss and the blob as the
  snapshot, passing the current mirror rows as existing_blocks, so the
  executor's mirror reconcile does an in-place replace with no row
  deletion (check that build_merge_blob only uses the boss it is given
  and doesn't re-read the old blob from the db). Fallback: delete the
  page's block rows, delete backlink rows referencing it, delete the
  pages row, then build_merge_new_page. Never merge the blob into the
  existing local boss (union = wrong).

### VERBS

EXPORT (works in any mode, no server): flush_now; inside run_exclusive
read only (all pages query + backlinks query), then leave the lock;
build the hard copy from the in-memory blobs outside the lock (skip
disabled pages). Two single queries under the lock are enough. No
transaction needed; a keystroke landing after the flush is simply not
in the file. Serialize pretty JSON and trigger a browser download of a
Blob named like zakken-backup-<date>.json. Keep the download piece in
its own small function (Android will write a file instead).

IMPORT: (1) read file text, prepare_import (all validation and doc
building, in memory); (2) set mode needs_reconcile; (3) one
run_exclusive closure containing one transaction: wipe_data_tables_unlocked,
then apply_merge_plan for each prepared page, then write backlink rows
with new_backlink_row; any error rolls back and leaves the old data;
(4) restart_sync_stack(). Server untouched. USER_ID/SESSION_ID in
key_value_storage are untouched. Do not clear the stored generation.

PAUSE / RESUME: pause = set mode paused (immediate). resume = refuse
unless mode == paused; otherwise set connected + restart_sync_stack
(boot re-runs login, generation check, bootstrap, pull).

WIPE SERVER: set needs_reconcile; send WipeDataset; on Ok(new_gen): in
one run_exclusive, delete client_love_letter rows; clear_seen_everywhere();
store new_gen; leave mode needs_reconcile; popup. Local pages are
untouched, so no reload is needed. (A following reconcile will push
everything as push_to_server.)

ADOPT (client is source of truth): (1) set needs_reconcile; busy
overlay; (2) under run_exclusive, read all local pages, drop disabled
ones, capture {page_id, is_main, blob} (the full blobbed_page blob,
which is a full snapshot); (3) send AdoptDataset with those; on Err:
leave mode needs_reconcile, tell the user the server may be partially
written and adopt can be rerun; (4) on Ok(new_gen): in one
run_exclusive verify the local blobs still equal the captured ones
(else abort with a message), delete client_love_letter rows,
clear_seen_everywhere(), store new_gen, set mode connected; (5)
restart_sync_stack. The client keeps its own docs untouched. The
server stored the same bytes, so both sides share one lineage.

RECONCILE (two phases around user input):
 PLAN: remember the previous mode; set needs_reconcile; flush; read all
 local pages (one run_exclusive read); GetAllYrsPages; classify_pages;
 return a UI-safe ReconcilePlan (summary counts per bucket, the
 conflicts with PageDiff) and keep the blobs in a private session
 object. Cancel before apply = restore the previous mode +
 restart_sync_stack.
 APPLY(decisions: page_id -> Server | Client, for conflicts only):
  Network phase first (abort on the first error; nothing local has
  been touched yet, rerunning reconcile is safe):
   - push_to_server: send CreateNewPage{page_id, is_main_menu_page,
     snapshot_of_edit = the client's FULL blob} via send_letter_direct.
     One letter per page; no per-block letters.
   - conflicts decided Client: build_client_wins_update on the SERVER
     blob captured in the plan; wrap snapshot_of_edit in the "generic
     merge letter" = EditBlock with an empty text edit (Insert "" at
     0), edit_target Text, block_id empty (apply_bootstrap_row in
     receive.rs already builds this shape; the server and receivers
     ignore everything but the snapshot). Send via send_letter_direct.
     Keep final_blob for the local phase.
   - disable_on_server: send DisablePage{page_id, snapshot_of_edit =
     local page_status blob}.
  Local phase (one run_exclusive + one transaction, using builders +
  apply_merge_plan, never merge.rs::apply_letter): first verify every
  local blob still equals what the plan captured (else abort: "local
  data changed, reconcile again"); then upsert_page_from_blob for:
  take_from_server (server blob), identical (server blob; contents are
  equal and this removes any lineage doubt), conflicts decided Server
  (server blob), conflicts decided Client (final_blob); for
  disable_locally apply the disable via build_merge_disable using the
  server's status blob; delete all client_love_letter rows (they're
  redundant or obsolete now and may be from another lineage); commit.
  The local page's is_main_menu_page always takes the server's value
  except for push_to_server pages.
  Finish: store CURRENT_LOGIN_GENERATION as "dataset_generation", set
  mode connected, restart_sync_stack(). After the reload the worker
  pulls GetMyUnsentLetters; letters that the server queued for us
  meanwhile were not acked, so they are applied then (same lineage,
  idempotent).
 Direct sends are not durable across a crash; acceptable, because the
 next reconcile detects whatever is still different.

### UI CONTRACT (human builds the UI; expose these)

Signals: SYNC_MODE; DATASET_OP_BUSY (Option<String> message that
drives a blocking full-screen overlay; set it at the start of every
destructive verb and until reload).

Functions, each returning Result<_, String> with user-presentable
errors: export_download(); import_from_text(String); pause(); resume();
wipe_server(); adopt_local(); reconcile_plan() -> ReconcilePlan;
reconcile_apply(decisions); reconcile_cancel(). UI is responsible for
confirmations on import/wipe/adopt and for a "use server for all / use
mine for all" convenience. The recommended conflict view is a per-page
summary (counts + both titles) with an expand into the aligned
side-by-side rows from PageDiff; default collapsed.

### ORDER OF WORK

1. Pure layer + BossOfYrs read method + unit tests (diff_page,
   classify_pages, build_client_wins_update incl. no duplicate ids and
   the reorder cases, prepare_import validation incl. round trip
   export->import giving equal ordered (id, content, meta), hard-copy
   ignores disabled pages).
2. Server: protocol, generation, operations_dataset, telephone kick +
   stamp, handlers, SendLoveLetter guard, login reply. cargo check on
   the server and on mock_client_core (fix its login decode).
3. Client plumbing: mode + gating points, login/generation check,
   router global, send_letter_direct, db helpers.
4. Verbs in this order: export, import, pause/resume, wipe, adopt,
   reconcile.
5. Manual test with the mock client as second user: (a) wipe from web:
   mock client's socket is kicked, on reconnect it enters
   needs_reconcile; (b) web adopt then mock client reconcile: edits
   made on both sides after that converge; (c) export, import in a
   fresh profile, reconcile against the old server: identical pages
   take the server blob and edits flow both ways; (d) edit the same
   page on both sides while paused, reconcile with Client and with
   Server picks; (e) delete the whole server DB file, restart: client
   sees a generation mismatch, not silent data loss.

### KNOWN LIMITS (leave as is)

Adopt sends all pages in one websocket message (check axum's default
max message size if a dataset gets large; chunking is future work).
Backlinks are local-only until something writes them. While gated, the
server keeps re-pushing the unacked backlog. Direct reconcile sends
are not crash-durable. is_main_menu_page can't be changed after page
creation.

---

## 2. Addendum: resolutions to questions raised

Overrides section 1 where they differ.

1. Stale login stamp after wipe
   - The server's wipe handler and adopt handler must both re-stamp
     the CALLER's socket with the newly minted generation (add a
     telephone method set_generation(socket_id, gen); call it after
     the std mutex guard is dropped). Other sockets are kicked, not
     re-stamped.
   - Wipe does not reload. After wipe, the client must also set
     CURRENT_LOGIN_GENERATION to the new value, in addition to storing
     "dataset_generation". Adopt does the same before its
     restart_sync_stack. Reconcile's finish step stores
     CURRENT_LOGIN_GENERATION, so without this it would store the
     pre-wipe value.
   - Reconcile after wipe then works on the same socket with no
     re-login. The sender is gated while mode is needs_reconcile, so
     no stray letters go out from the caller in the meantime.

2. is_main_menu_page on existing local pages
   - Do not rely on apply_merge_plan to update the
     pages.is_main_menu_page column; it only reads the flag for new
     pages.
   - In upsert_page_from_blob: if the page exists locally and its flag
     equals the incoming flag, use the in-place path. If the flags
     differ (essentially never; it can only happen with a hand-edited
     import file), use the delete-and-rebuild fallback (delete the
     page's block rows, backlink rows referencing it, the pages row,
     then build_merge_new_page with the incoming flag). Add no new
     column-update code.
   - The flag is set once at creation via CreateNewPage and no letter
     variant changes it later. Do not write code that treats it as
     frozen; prepare_import only validates that it is a bool.

3. Server wipe
   - wipe_dataset_tables uses the existing delete_all_rows (z_db) on
     each table, in this order: unsent_love_letters,
     love_letters_from_clients, pages. Do not add a new bulk-delete
     path, and do not call delete_row_where with SelectArgument::All
     (sql_builder refuses it). Keep registered_users and
     key_value_storage.
   - If delete_all_rows does not reset autoincrement ids, that is
     fine: every generation change clears the client's seen set
     (invariant I5), so id reuse cannot cause dropped letters.

4. page_status_is_active
   - classify_pages (builder crate) takes active: bool per page. The
     client computes it with page_status_is_active(status_blob,
     page_id) for BOTH local and server pages before calling.
     pub(crate) in web_interface is correct; no cross-crate
     visibility is needed.

---

## 3. Addendum: mock client staleness

Add under "Mock client changes" (section 1 has none; this is the
section) and update the test plan.

The mock client gets a minimal staleness mode. The point is that it
must never send or apply anything built from old-lineage state.

State: a `known_generation: Option<i64>` (in memory only; process
restart is already a fresh start) and a `stale: bool`.

Login: decode the new Ok(DatasetGenerationPayload) reply. If
known_generation is None, store it. If it equals the stored value,
continue. If it differs, set stale = true, print one clear line
("server dataset changed; local state is old. Type `resync`."), and do
NOT update known_generation. A login failure of "no such user" after a
server DB wipe counts as stale too.

Other triggers for stale = true:

- The socket closes or errors. The server kicks other sockets on
  wipe/adopt and restarts close everything. If the mock already has
  reconnect logic, keep it, but a reconnect never clears stale by
  itself.
- A SendLoveLetter reply that is Err(IllegalInput) containing "dataset
  changed".

While stale:

- Every editing command (edit_random_page,
  insert_block_on_random_page, and anything else that builds a letter
  from local pages) refuses with the same printed message and sends
  nothing.
- `sync` and any pull or send is refused.
- Incoming LoveLetterFromServer pushes are dropped without applying or
  acking (they stay on the server). Applying a new-lineage letter to
  old-lineage in-memory pages is exactly the corruption we are
  preventing.
- Read-only commands (listing pages) still work but print a stale
  warning.

New REPL command `resync`: drop the in-memory pages map, the
seen-letter set, and any outgoing queue; open a fresh connection (new
socket, so a fresh generation stamp); log in (register if the user is
gone); store the login generation as known_generation; run load_pages
exactly as at startup; set stale = false. This is equivalent to
restarting the process, done in place.

Check before implementing (I haven't seen the mock's code): how the
REPL dispatches commands and where the pages map and seen set live, so
resync can reset them in one place; whether the mock has any reconnect
loop, and whether a kicked socket currently crashes the REPL or just
errors on the next send.

Test plan updates:

- Test (a) wipe from web: the mock's socket is kicked, the mock
  reports stale, edits are refused. After `resync` the mock has empty
  pages (the server was wiped), which is correct.
- Test (b) web adopt, then mock `resync`: the mock now holds
  new-lineage pages; edits made on both sides afterwards converge.
  Before `resync`, confirm that an edit attempt is refused and nothing
  reaches the server.
- Test (e) server DB deleted and restarted: the mock notices (socket
  loss, then a generation or no-such-user mismatch), refuses to edit,
  and `resync` recovers.
- If you want the mock to do real page-level reconcile later, that is
  a separate project; the mock does not implement it.

The mock needs three stale triggers, not just one. A reconnect after a
kick logs in fresh and gets the new stamp, so the server guard only
blocks sockets that stayed connected across the change. The mock's own
client-side check is the only thing that catches the reconnect case.

---

## 4. Addendum: reconcile is a process, not a plan

Overrides section 1 where it differs, and supersedes the shapes named in it.

The model in section 1 — `classify_pages` returning a `ReconcilePlan`, a `ReconcileDecisions` map, `diff_page`, `build_client_wins_update`, a session holding the plan — was wrong. The real model:

- Only `local_pages` is passed around.
- The client hands `local_pages` to the server.
- The server checks, page by page, whether it has that page exactly as the client does.
- For a page the server does not have exactly, it sends its full version of that page.
- For a page that matches, nothing is sent.
- The comparison happens live inside that exchange. Nothing is computed up front and nothing is stored between layers.

That means `ReconcilePlan`, `PageDiffSummary`, `ReconcileDecisions`, `classify_pages`, `diff_page`, and `build_client_wins_update` are placeholders that do not match the model and should not be built against. `reconcile_plan() -> ReconcilePlan` is the wrong signature. The two phases (plan then apply, around user input) were an artefact of the plan object; the real flow is one process that starts from the server-side check and runs to completion.

Two shapes that remain correct and are not affected:

- The `SendLoveLetter` generation guard (section 1, "SERVER CHANGES"): a letter from a stale socket is still refused.
- The `CURRENT_LOGIN_GENERATION` write on reconcile finish (section 2, addendum item 1): still happens.

The template in `connecting_to_server_template` mocks this — three verbs (`reconcile_plan`, `reconcile_apply`, `reconcile_cancel`), a modal that reads `local_pages` and pretends page-by-page, nothing stored in between. It is a stand-in for the model above, not an implementation of it. See `connecting_to_server_template/behaviour_decisions.md` "Not decisions — reconcile".
