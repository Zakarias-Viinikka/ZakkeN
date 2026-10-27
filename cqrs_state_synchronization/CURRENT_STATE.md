# Current state

- web_interface is a Leptos csr app, built and served with trunk.
- Routes: `/` and `/Menu` (Menu, a nav page), `/DbGui` (the orm, comes
  from the web_internal_db crate in z_db).

- create_page in the builder works. It builds a BossOfYrs, inserts two blocks (title, then a normal one), snapshots the doc, takes a version bookmark, and returns all the prepared data holding three ctx structs.

- the executor takes all the prepared data and does the three inserts in one transaction: pages, every_block_in_existence, uncommitted_diffs. The executor fns check whether a transaction is already open; if one is, they don't begin or commit, and the outer call owns it.

- there's a method for editing a block, and it's also used for adding a title to a fresh page.

- disable_page marks a page disabled instead of deleting it. The page row stays, every block for it gets page_is_disabled = "true", and a new active_pages blob goes into pages.page_status. The menu filters disabled pages out on load.

- the menu reads its page list from db on load. LocalPages holds db_id, title, block_id, page_id, is_disabled. There's no page counter anymore.

- the dev sidebar has two buttons: delete everything and insert three pages. Both do their db work, then reload the page.

- the eye button next to each menu title disables that page.

- main.rs creates 3 tables if they don't exist.

## Update - 2026-10-19

Routes: `/`, `/Menu`, `/DbGui`, and `/MainContent/:page_id` (reads the page id from the URL).

`App` in main.rs gates the whole router behind table creation, so a direct URL load doesn't race the tables.

New crate modules in web_interface:

- `complicated_signal_flow.rs` - `Page` (one page's live state: page_id, boss_of_yrs, blocks, page_state, preload_request, preload_response), `PageLoadState` (Loading / PreloadLoaded(usize) / Loaded / Online / Offline / LoadFailed), `BlockRow` (the one block type), `LocalPages`.
- `leptos_components/main_content/` - MainContent, ConnectionIndicator, BackgroundActivityIndicator (placeholder), page_loading_logic.
- `drag_reorder_container/` - DragReorderContainer (wrapper), my_block_content (block UI, human-edited), drag_reorder_black_box (rapier + drag + reorder + vacuum, untouched).

Loading is two effects per page: one matches on page_state and advances one step (preload_pages, load_offline, load_from_server), one watches preload_response and applies blocks. load_from_server is a no-op until the server path exists; preload_pages seeds an empty response because no preloader exists yet.

on_block_input goes through edit_title_for_page into the executor and persists.

## Update 2026-10-20

Order now comes from a fractional position on every block, in both yrs
and the db mirror. Reads sort by (frac_position, block_id). The
fracindex crate generates the strings, wrapped by a new
ZakkeN/smart_frac_index crate so the same logic is available outside
yrs. every_block_in_existence.position became frac_position, Text, and
get_all_blocks_for_page uses get_data_ordered with
ORDER BY frac_position ASC, my_id_as_given_by_yrs ASC.

yrs gains move_block(block_id, block_id_of_above, block_id_of_below),
a frac write rather than a delete+insert, and
insert_new_block_with_preexisting_id for the eventual cross-page path.

love_letter gains MoveBlock { block_id, target_page_id,
new_frac_position }. The builder is move_block, the executor
move_block_requires_three_db_actions. ui_actions::move_block wraps both.
on_move in MainContent calls it for same-page moves only; cross-page
drops log and do nothing yet.

add_block_requires_four_db_actions renamed to
add_block_requires_three_db_actions, because set_block_position_to_id
is gone now that the frac position is known before the row is inserted.

Moving a block will not re-order the page until reload. The drag
component already moved it visually; the block signal is re-read from db
on load.

## Update 2026-10-20 (later)

Cross-page move works on web. `ui_actions_move_block_operations::
move_block_to_another_page` runs `delete_block` on the old page's boss
and `add_block(Some(block_id), ...)` on the new page's boss, both
executors in one transaction. `on_move` calls it for cross-page drops;
same-page drops go through `ui_actions::move_block`.

`PositionToInsert` is now `AtEnd` or `Between { block_id_of_above,
block_id_of_below }`. `SpecificPosition` is gone; numeric array slot
stopped meaning anything once order came from frac position.

`add_block` gained `insert_with_existing_block_id: Option<String>`.
`Some` calls `insert_new_block_with_preexisting_id` and builds a
`CreateNewBlockWithId` letter; `None` is the original path.

Session id, user id, and time are real. `lib.rs` has `get_time()`.
`db/cached_ids.rs` holds `SESSION_ID` and `USER_ID`, both
`Option<String>` signals. `init::init_all` fills them from
`key_value_storage`; session id increments each reload.
`db/key_value_storage_db_actions.rs` has
`read_key_value_from_db` and `write_key_value_to_db`. Route `/setup`
and a `UserGate` in `main.rs` force first-run username setup. Errors
at init, load, edit, and move surface as popups via
`small_components::popup::create_popup`.

dev_actions::insert_three_pages seeds three pages with varied titles and bodies.

## Update 2026-10-20 (end of session)

Batcher is live. `BatcherHandle` in `complicated_signal_flow.rs`:
`push_edit(boss, block_id, diff)` appends and pokes, `flush_now()` sends
a message and awaits a oneshot reply. `Batcher::run` drains the
receiver; a poke starts a 500ms gloo-timers debounce, and either the
timer expiring or a `Flush` message drains. Each block's pending diffs
go through `combine_getdiff_results`, then `ui_actions::edit_block`,
all inside one transaction. On failure the whole batch is rolled back,
`APP_LOCKED` flips to true, and a popup tells the user to copy what
they need and refresh. `MainContent` reads `APP_LOCKED` and changes the
background. Only set once per session.

Every ui_actions entry point that writes now takes a `BatcherHandle` and
calls `flush_now()` first: `add_block`, `create_new_page`,
`disable_page`, `move_block`, `move_block_to_another_page`.
`edit_block` stays flush-free, since it's the flush target.
`page_loading_logic::load_offline` also flushes before replacing the
boss.

Text input goes through `BlockInputEvent` (block_id, old_text, new_text,
was_enter, cursor_pos) from `MyBlockContent` up to `on_block_input`.
`check_if_should_make_new_block` fires only when Enter was pressed, the
block was non-empty, the new text ends with two newlines, and the
cursor was at the end. On that path the `\n\n` is pushed to the batcher
first, then `add_block` runs and the new block is appended to
`page.blocks` locally (no full reload). Focus is requested via
`FOCUS_BLOCK_ID`.

Backspace or Delete in an empty non-title block asks MainContent to
delete the block. Refused when there's only one normal block left.

`Page` and `PageLoadState` moved to `page_state.rs`. `OpenPages` (page1
plus an RwSignal<Option<Page>> page2) is created in `main.rs` and put in
context. `MainContentRoute` resets page1's fields when the URL's
page_id changes, using leptos's `batch()` to avoid firing the load
effect mid-reset.

`text_diff::get_diff` had an underflow when the strings' shared suffix
overlapped the shared prefix (e.g. old "a\n\n" -> new "a\n\n\n"). The
last-deviation position is now capped before subtracting.

`MyBlockContent` no longer shows a body placeholder, has a subtler
border, and auto-sizes on mount by retrying the height measure across a
few frames.

## Update 2026-10-21

Two delete bugs closed. `sync_from_props` in `sync.rs` gained a pure-
delete fast path (`fast_delete`) and a pure-add fast path (`fast_add`).
Both skip the full rebuild and animate the remaining blocks; anything
else still falls back to full rebuild.

`FOCUS_BLOCK_ID` is set by delete (block above, cursor at end) and by
split (new block, cursor at start). `FOCUS_BLOCK_AT_START` was added
alongside it for the split case. Both live in
`complicated_signal_flow/core.rs`.

Backspace at position 0 of a non-empty non-title block merges into the
block above (or deletes it if empty). Cursor lands at the join point.

Split trigger is three newlines in a row. The 8-row minimum was
removed. After a split, `page.blocks` is reloaded from DB rather than
inserted locally: the local insert could drift out of yrs frac order
and produce "no room between" errors on the next split.

`complicated_signal_flow.rs` became a folder (`complicated_signal_flow/`)
with `core.rs` holding the old contents and `mod.rs` re-exporting.
External use paths unchanged.

Black box identity bug fixed: `id_map` is written back in
`sync_from_props`, and `fast_add` / `fast_delete` keep it in sync.

Focus-on-split is the one open bug. The focus effect fires and the new
textarea is `document.activeElement`, but typing goes into the old
block. `tmp_focus_test` (at repo root, temporary) reproduces the
split-and-focus mechanic correctly, so the cause is structural.
Handed off to another AI to isolate.

## Update - 2026-10-24

Focus-on-split is fixed. The bug was `.ps-box.ps-unmeasured` using
`visibility: hidden` while a freshly inserted block waits to be
measured by the ResizeObserver. Chrome sets `document.activeElement`
to a `visibility: hidden` textarea on `.focus()`, but does not route
keystrokes to it — input keeps going to the previously focused
element. The `FOCUS_BLOCK_AT_START` effect cleared the signal on the
first attempt regardless of visibility, so it never retried. Fixed by
changing the rule to `opacity: 0; pointer-events: none`, which hides
the block the same way without blocking focus or input routing.

The reproduction harness is `tmp_focus_test`. It only reproduces the
bug when measurement is delayed past the focus attempt, because the
focus effect fires once, calls `.focus()`, and clears the signal
unconditionally. The CSS rule blocks input routing on the hidden
element; the effect's clear-on-first-attempt is what prevents it from
ever recovering when the block becomes visible a frame later.

## Update - 2026-10-27

Enter in a non-empty, non-title block now splits at the cursor, through
a new `on_split` callback carrying (block_id, text_before_cursor,
text_after_cursor). Enter in an empty block or the title behaves as
before. The three-newlines rule and `check_if_should_make_new_block`
are gone. The textarea blurs on Enter so no caret is left behind during
the split. Shift+Enter also splits now — no distinct shift path.

Blocks appear before the DB has them. The batcher's queue is now an
enum of Edit and AddBlock entries, run in order. `flush` writes the
pending edits first, then the add, all in one transaction; a new
`write_edit_run` helper does the edit writing. `push_add_block` queues
an add and returns immediately. `ui_actions::add_block_unflushed` is a
version of `add_block` that skips `flush_now()` — the batcher's own
flush needs this or it deadlocks.

`on_split` makes the new block id in the UI (`new_local_block_id`, 32
random hex characters). It queues the old block's trim and the new
block's add, updates `page.blocks` in one go with the new row directly
after the old, then sets focus at the start of the new block.
`split_block_at_cursor`, the DB reload after a split, and the
`db_helpers_for_web_client` import in main_content.rs are removed.

Arrow keys: Down at the very end of a block focuses the start of the
block below. Up at the very start focuses the end of the block above.
Otherwise the browser handles the arrow key. Holding Shift, Ctrl, Alt
or Cmd skips this so selection still works. Goes through a new
`on_navigate` callback and the existing `FOCUS_BLOCK_ID` /
`FOCUS_BLOCK_AT_START` signals.

Untested this session: whether yrs accepts the 32-hex local block id,
and whether the queued add works when the executor skips its own begin
and commit inside an already-open transaction (which CURRENT_STATE
says it does). Not compiled as far as I know.
