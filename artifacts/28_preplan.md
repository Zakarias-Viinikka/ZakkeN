# Pre-plan: export / import / wipe / adopt / reconcile / disconnect

2026-10-28

## What this is

A set of user-triggered operations that let a user snapshot the
dataset, restore it, reset the server, replace the server's dataset
with the client's, or reconcile per-page differences between local
and server. Web first, Android later. Everything is user-initiated;
nothing runs in the background. The intent is not real-time sync -
it's "the sync broke, or I need to move machines, or the server
died, give me tools to fix it."

## The hard copy

One JSON file. Pages and backlinks. No yrs blobs anywhere - no
blobbed_page, no page_status, no version, no disabled. Just the data
that would survive if you threw away the CRDT.

A page in the hard copy is: page_id, is_main_menu_page, and a list
of blocks. A block is block_id, content, meta, frac_position,
is_title, is_part_of_main_menu_page. Block ids are preserved so that
a client that adopts and later edits can send EditBlock against a
block the server also has by the same id.

A backlink is just page_that_holds_link_id and
page_being_linked_to_id. Nothing currently writes backlinks, so this
is a placeholder for the schema as designed - flagged so the smart
AI doesn't spend time wondering why it's there.

Disabled pages are excluded entirely. Their page row and every one
of their block rows are dropped from the hard copy.

The hard copy has a version field at the top level (integer, starts
at 1) so future format changes are detectable.

## The rebuild primitive

Assumed to exist: BossOfYrs::from_hard_copy(blocks: Vec<HardCopyBlock>)
-> Arc<BossOfYrs>. Takes a block list with ids, content, meta, frac,
and returns a fresh yrs doc that uses
insert_new_block_with_preexisting_id internally so the ids survive.
Fresh clocks. No history. This is a separate task, listed as a todo
for the human. It needs to exist on both client and server side.
This pre-plan assumes it exists and does not specify it.

## The dataset generation id

A monotonic integer stored in key_value_storage on the server, minted
at server startup if absent and re-minted on wipe / adopt. Login's
reply carries it. The client stores it in its own key_value_storage
under a matching key. On login, if the client's stored generation
differs from the server's, the client sets its disconnected flag,
clears seen_server_letters, and pops a message pointing at reconcile.
On any generation change (including the client's own adopt flow), the
client clears seen_server_letters - otherwise ids restarting at 1
would collide with old entries and letters would silently get
dropped.

The existing NoSuchUser path in identity.rs - which clears the seen
set when the user table is gone - is a different case and stays. It
catches full server wipes. Generation catches partial resets. Both
matter.

## The verbs

Disconnect is a flag in key_value_storage, sticky across reload.
While set, the socket stays open, but two things are gated: the
router's on_love_letters callback (which drops incoming pushes
without forwarding to the worker), and kick_send (which becomes a
no-op). The letters the server pushes during disconnect are simply
not acked; they'll come back on next pull. The reconcile, wipe,
adopt, and export verbs all work regardless of the flag. Import
forces the flag on.

Export reads all pages (skipping disabled), their blocks, and all
backlinks, under the write lock so nothing changes mid-read, writes
the JSON, and triggers a browser download. No server interaction. No
disconnect required. Export doesn't touch the queue.

Import reads a JSON file the user selected, wipes the local tables
(pages, every_block_in_existence, backlinks, client_love_letter,
seen_server_letters), rebuilds each page with from_hard_copy, writes
the block rows, writes the backlinks, and forces the disconnect flag
on. Any currently-open page reloads. The server is untouched. Import
is all-or-nothing - if it fails mid-way, the user is in a bad state,
so it should be transactional under the write lock.

Wipe server sends a new request variant. The server deletes every
row from pages, love_letters_from_clients, and unsent_love_letters,
mints a new generation, and replies with the new generation. The
client clears its local client_love_letter and seen_server_letters,
stores the new generation, sets the disconnect flag, and pops a
message. Client's pages and blocks are not touched - they stay
diverged from the now-empty server.

Adopt builds the hard copy from current local state, sends it as one
payload (a new request variant carrying pages and backlinks), and
the server wipes its own pages, letters, and unsent tables, rebuilds
each page from the hard copy, writes the backlinks, mints a new
generation, and replies Ok. The client then wipes its local tables,
rebuilds from the same in-memory hard copy, stores the new
generation, clears the disconnect flag, and reconnects. Both sides
end up with fresh yrs docs whose content matches but whose clocks
differ - that's fine; yrs updates are self-describing and the two
converge on future edits. The currently-open page reloads. Adopt is
the only flow that both sends and wipes in one user action.

Reconcile pulls GetAllYrsPages, and for each page id in the union of
local and server:

- only on the server: rebuild locally from the server's hard copy. No
  letters.
- only on the client: send CreateNewPage + one CreateNewBlockWithId
  per block, directly, bypassing client_love_letter.
- on both, and hard copies match: nothing.
- on both, and hard copies differ: present the page to the user with
  a left/right pick. If the user picks the server's version, the
  client rebuilds the page locally from the server's hard copy and
  does nothing on the server. If the user picks the client's version,
  the client sends RemoveBlock for every block the server has that
  the client doesn't, CreateNewBlockWithId for every block the client
  has that the server doesn't, EditBlock for every block that exists
  on both but differs in content, and MoveBlock for any block whose
  frac differs - all directly, bypassing the queue. Then it also
  rebuilds the page locally from the client's own state.

Reconcile requires no new letter types. The whole flow is
GetAllYrsPages and direct sends of existing letters. Direct sends
are not durable against a crash - if the user reloads mid-reconcile,
some letters may have gone and some not, and the next reconcile will
detect the difference again and redo them. That's acceptable for v1.
Reconcile ends by clearing the disconnect flag and reconnecting, and
by pulling GetMyUnsentLetters once to pick up anything the server
pushed during the reconcile.

## Decisions made on purpose, so the smart AI can veto

The socket stays up while disconnected. Gating is at the receive
callback and the sender, not by closing the connection.

Reconcile uses existing letters. No new letter types. The
alternative - a ReplacePageWithHardCopy letter - would be cleaner but
is a protocol change.

Import forces disconnect. Otherwise a user imports and immediately
syncs imported data to a server they didn't intend to touch.

Export triggers a browser download of a Blob. This is web-specific;
Android will need a filesystem version.

Currently-open page reloads after adopt and import.

## Open questions the smart AI should resolve

How the reconcile UI presents the block-level difference (full text
side by side vs. a summary and an expand).

Whether the write lock is enough to guarantee export is atomic given
that export spans multiple get_data calls.

Whether the server's adopt / wipe path should share code with
apply_letter for rebuilding a page, or duplicate the tiny bit of
logic. Instinct: duplicate.

## Files to read

    cat /home/zakke/ProgStuff/ZakkeN/zakken_server/shared/server_protocol/src/request.rs /home/zakke/ProgStuff/ZakkeN/zakken_server/shared/server_protocol/src/message.rs /home/zakke/ProgStuff/ZakkeN/zakken_server/shared/server_protocol/src/lib.rs /home/zakke/ProgStuff/ZakkeN/zakken_server/server_side/server_core/src/allowed_requests/mod.rs /home/zakke/ProgStuff/ZakkeN/zakken_server/server_side/server_core/src/allowed_requests/core.rs /home/zakke/ProgStuff/ZakkeN/zakken_server/server_side/server_core/src/allowed_requests/login.rs /home/zakke/ProgStuff/ZakkeN/zakken_server/server_side/server_core/src/allowed_requests/send_love_letter.rs /home/zakke/ProgStuff/ZakkeN/zakken_server/server_side/server_core/src/socket.rs /home/zakke/ProgStuff/ZakkeN/zakken_server/server_side/db/src/db/operations_key_value.rs /home/zakke/ProgStuff/ZakkeN/zakken_server/server_side/db/src/db/operations_pages.rs /home/zakke/ProgStuff/ZakkeN/zakken_server/server_side/db/src/db/operations_love_letters.rs /home/zakke/ProgStuff/ZakkeN/zakken_server/server_side/db/src/db/apply_letter.rs /home/zakke/ProgStuff/ZakkeN/zakken_server/server_side/db/src/db/schema/mod.rs /home/zakke/ProgStuff/ZakkeN/cqrs_state_synchronization/web_interface/src/server_link/mod.rs /home/zakke/ProgStuff/ZakkeN/cqrs_state_synchronization/web_interface/src/server_link/core.rs /home/zakke/ProgStuff/ZakkeN/cqrs_state_synchronization/web_interface/src/server_link/identity.rs /home/zakke/ProgStuff/ZakkeN/cqrs_state_synchronization/web_interface/src/server_link/send.rs /home/zakke/ProgStuff/ZakkeN/cqrs_state_synchronization/web_interface/src/server_link/receive.rs /home/zakke/ProgStuff/ZakkeN/cqrs_state_synchronization/web_interface/src/server_link/merge.rs /home/zakke/ProgStuff/ZakkeN/cqrs_state_synchronization/web_interface/src/db/ui_actions.rs /home/zakke/ProgStuff/ZakkeN/cqrs_state_synchronization/web_interface/src/db/db_helpers_for_web_client.rs /home/zakke/ProgStuff/ZakkeN/cqrs_state_synchronization/web_interface/src/db/key_value_storage_db_actions.rs /home/zakke/ProgStuff/ZakkeN/cqrs_state_synchronization/web_interface/src/db/write_lock.rs /home/zakke/ProgStuff/ZakkeN/client_table_blueprints/src/tbl_pages.rs /home/zakke/ProgStuff/ZakkeN/client_table_blueprints/src/tbl_every_block_in_existence.rs /home/zakke/ProgStuff/ZakkeN/client_table_blueprints/src/tbl_backlinks.rs /home/zakke/ProgStuff/ZakkeN/client_table_blueprints/src/tbl_client_love_letter.rs /home/zakke/ProgStuff/ZakkeN/client_table_blueprints/src/tbl_seen_server_letters.rs /home/zakke/ProgStuff/ZakkeN/love_letter/src/lib.rs /home/zakke/ProgStuff/ZakkeN/yrs/src/yrs_wrapper.rs /home/zakke/ProgStuff/ZakkeN/artifacts/oct_27_plan.md

## Prompt for the smart AI

I want to build a set of user-triggered operations for moving,
resetting, and reconciling my sync data across my web client and my
server. Web first; Android is deferred and only needs to be kept
compatible, not implemented. I've written a pre-plan that names the
verbs, the data shapes, the new request variants, and the decisions
already made. Read it. Then produce a concrete implementation plan.

The pre-plan is the shape, not the letter. It intentionally does not
specify every file or every function. Where the pre-plan says "new
module" or "new request variant," you decide the exact file path, the
exact struct shape, and the exact function signatures. Where the
pre-plan flags an open question, resolve it in your plan and say what
you chose and why. Where the pre-plan makes a decision you disagree
with, say so and propose the alternative - do not silently overrule
it.

Two things the pre-plan takes for granted. First,
BossOfYrs::from_hard_copy is a to-be-built primitive; it is out of
scope for your plan but everything assumes it exists and works.
Second, the client's write lock is a per-wasm-instance AtomicBool,
not cross-tab, so anything the pre-plan says runs "under the write
lock" is one tab's lock and does not defend against two tabs.

Your plan should be readable in one sitting. If it's longer than a
page or two of decisions, you're over-planning. Do not write code. Do
not open files that aren't in the cat line - the pre-plan's file list
is the whole context you need.
