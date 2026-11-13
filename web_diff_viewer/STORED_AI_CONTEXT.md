# web_diff_viewer - AI stored context

Read this before touching `example_2`, `block_diff_viewer_leptos`'s
public surface, or `generate_love_letters`. History and the original
build plan live in `plan.md`; this file describes the current state and
the shape decisions that took several rounds to land. If code and this
file disagree, the code wins — correct the file.

## What this is - 2026-11-13

A Leptos CSR port of zap's egui diff viewer, split into crates so the
viewer is a black box the caller mounts. Standalone from ZakkeN; lives
at `~/ProgStuff/ZakkeN/web_diff_viewer`. Its eventual purpose is to be
the diff surface for the reconcile flow in
`cqrs_state_synchronization/web_interface`.

Workspace members:

- `block_text_diff` — pure logic. Entry:
  `rows_for_two_pages(left: &[String], right: &[String]) -> Vec<Row>`.
  See `plan.md` for the row model, hunk assignment, similarity tuning,
  and move classification. Native tests via `c t` at workspace root.

- `block_diff_viewer_leptos` — the `DiffViewer` component plus pure
  helpers. `plan.md` documents the caller contract, per-block edit
  state, drag math, and styling tokens.

- `example_app` — the original trunk demo (picker + viewer + result).
  Kept as the reference for "how do I mount DiffViewer". Not the
  reconcile shell.

- `example_2` — the reconcile UI prototype. See its section below.

- `generate_love_letters` — pure. Turns the server's page plus the
  user's chosen final page into `LoveLetterSketch` values, keyed by
  block_id. No snapshots, no `yrs` doc — caller builds `LoveLetter`s
  via `build_a_love_letter` as usual.

## block_diff_viewer_leptos — caller contract

The viewer is a black box. Mounting code never reads its internals.

    let server = RwSignal::new(Some(page_blocks_for_the_viewer));
    let client = RwSignal::new(page_blocks_for_the_viewer);
    let kept = RwSignal::new(PickedSide::Mine);
    let boss = RwSignal::new(Some(server_boss_of_yrs));
    let (output, set_output) = signal(ViewerOutput::default());
    let handle = FinishHandle::new();

    <DiffViewer
        server_page=server
        client_page=client
        kept_side=kept
        server_page_boss=boss
        output=set_output
        handle=handle
        on_finished=Callback::new(move |_| { /* read output.get_untracked() */ })
    />
    <button on:click=move |_| handle.finish_and_write_the_result()>
        "Done"
    </button>

- `server_page` is `Signal<Option<PageBlocksForTheViewer>>`. `None`
  means client-only (no server copy); the viewer skips the diff and
  emits a single `CreateNewPage` love letter on finish.
- `client_page` is `Signal<PageBlocksForTheViewer>`.
- `kept_side` is `Signal<PickedSide>` — `Mine` puts server on the
  left and client on the right (client editable); `Server` puts client
  on the left and server on the right. Either way the editable side is
  the right.
- `server_page_boss` is `Signal<Option<Arc<BossOfYrs>>>`. When
  `Some`, the viewer builds full love letters on finish. When `None`,
  `love_letters` in the output comes back empty (diff-only mode).
  Snapshotted at mount, so a change to the boss does not remount.
- Changing `server_page`, `client_page`, or `kept_side` remounts the
  inner viewer; edits and drags never survive a remount.
- `output` is a `WriteSignal<ViewerOutput>`. `ViewerOutput` has two
  fields: `kept_blocks: Vec<Block>` (the editable right-side blocks in
  visual order, each with id, text, metadata, frac) and
  `love_letters: Vec<LoveLetter>` (the full letters that turn the
  server's page into `kept_blocks`; for a client-only page this is
  one `CreateNewPage`; empty in diff-only mode).
- `handle` is `FinishHandle`. Caller stores it, passes it in, and calls
  `handle.finish_and_write_the_result()` from any button. Internally
  it bumps a counter signal; the viewer's effect writes `output` then
  fires `on_finished`.
- `on_finished` is optional. Fires after every finish. Anything that
  reads `output.get_untracked()` inside it sees the fresh value.

`Block`, `LoveLetterSketch`, and `LoveLetter` are re-exported from the
viewer crate. `PageBlocksForTheViewer` is a page id,
`is_main_menu_page`, `Vec<Block>`, and the page's raw `blob: Vec<u8>`.

Two `FinishHandle`s can drive the same viewer — clone it, call the
method on either.

The only caller setup is one stylesheet link in `index.html`:

    <link data-trunk rel="scss" href="../block_diff_viewer_leptos/css/block_diff_viewer.scss" />

Everything inside the viewer is scoped under `.dv-root`.

## example_2 — the reconcile UI shell

This is the shape the real reconcile page should eventually take.
Every place the user has to re-explain "no, the picker stays, the
viewer is inline" is a sign they mean this. Read before touching the
layout.

Files:

- `src/app_main.rs` — layout orchestrator. Owns
  `pairs: RwSignal<Vec<PagePair>>`,
  `selected: RwSignal<Option<String>>`,
  `default_picked: RwSignal<Option<PickedSide>>`,
  `preview: RwSignal<Option<(String, Vec<String>)>>`,
  `submit_result`.
- `src/chooser.rs` — the numbered row of square buttons. Green when
  the pair is picked, accent border when selected.
- `src/pick_screen.rs` — the "which side is the base" screen. Contains
  `PickedSide { Mine, Server }`.
- `src/viewer_host.rs` — the wrapper around `DiffViewer`. Toolbar plus
  the diff plus the preview strip.
- `src/placeholder_data.rs` — `PagePair`, `placeholder_items()`. Three
  pairs; each block carries id + text + metadata + frac_position.
- `src/resolve_choice.rs` — `Mine / Server / Merged(Vec<String>)`.
  Unused by the layout currently, kept for parity with the real
  reconcile page's enum.

### Flow, in the order the user sees it

1. Nothing selected. One dark rounded box holding, top to bottom:
   - the numbered chooser row (`1 2 3`)
   - the "Default for everything" picker (Ask each time / Always mine /
     Always server's) — pick a non-Ask value and clicking a number
     skips the pick screen for that pair
   - either "Ready to submit" with a Submit button (if any pair is
     picked), or "No diff open" (nothing to do yet)
   - after Submit: the aggregated love-letter summaries across all
     picked pairs, under a "Submitted" header

2. Click a number. If the pair has a `picked` side (or a default is
   set), it skips step 3.

3. Pick screen. Same dark box styling. Top row is a 3-column grid:
   numbered close-box + page_id on the left, two framed buttons
   ("Use mine" / "Use server's") dead-center. Below it, a side-by-side
   plain text preview of both pages. Buttons glow on hover; default
   state is dull (`cool_button.md` at the ZakkeN root is the source
   styling, `.btn-6` "Framed").

4. Viewer. Same dark box styling. Toolbar top, diff body, preview strip
   at the bottom (hidden until Done). Toolbar order, left to right:
   - numbered box (blue, 48px square) — closes and saves current edits
   - page_id
   - counts (`N left · M right`)
   - hint (`left: discarded · right: kept (editable, draggable)`)
   - legend (Deleted / Inserted / Edited / Moved) — hidden below 900px,
     shown in the toolbar above 900px. The viewer's own legend is
     hidden at that breakpoint via CSS so there aren't two.
   - Reset (only when no default is set; resets pick + edits for this
     pair and returns to the picker)
   - Done (primary, blue) — computes summaries for the current page,
     writes them to `preview`, keeps the viewer open so the user can
     see what would be sent

5. Done → preview strip appears under the diff. "Would generate:"
   header. Empty list means "kept side matches the server, no love
   letters". Stays open so the user can keep editing. Editing after
   Done and pressing Done again rewrites the preview.

6. X (numbered box) → saves current edits into
   `pair.edited_kept_blocks`, closes, back to chooser. Reopening the
   same pair later starts from `edited_kept_blocks`, not the
   originals.

7. Submit (only in nothing-selected, only when at least one pair is
   picked) → for every picked pair, runs `generate_sketches` and
   aggregates the summaries. Renders as "Submitted — love letters
   that would be sent:" with one section per page. Empty list =
   "no love letters".

### Left/right pairing

Mine picked → server on the left (discarded), client on the right
(kept, editable). Server picked → client on the left, server on the
right. The user picks "whose truth wins" up front, then edits the
winning side against the loser. `PagePair.edited_kept_blocks` holds
the kept side's latest edits so reopening continues from there.

### Green state

A chooser button is green when `pair.picked.is_some()`. Green is an
indicator, not a lock — clicking a green button still opens the pick
screen (or viewer, if a default is set) and lets the user change their
mind. Reset on the viewer clears both `picked` and
`edited_kept_blocks` for that pair.

## generate_love_letters — sketch generation

Entry:

    pub fn generate_sketches(
        target_page_id: &str,
        server: &[Block],
        user: &[Block],
    ) -> Result<Vec<LoveLetterSketch>, GenErr>

`Block` is `{ block_id: String, text: String, metadata: String,
frac_position: String }`. `metadata` is currently opaque — kept so the
API doesn't change when the real thing starts using it.

Rules:

- **The first server block is the title. It is never removed.** A
  blanked title becomes an edit (replace text with empty). Otherwise
  blanking a non-empty block is a deletion.
- A server id is gone from `user` → `RemoveBlock` (unless it's the
  title, in which case nothing is emitted — see above).
- A server id is in `user` with empty text AND the server's text was
  non-empty AND it's not the title → `RemoveBlock`. Blank on both
  sides is a no-op.
- A server id is in `user` with different text → `EditBlock` with a
  fine-grained `TextEdit` computed by a private
  `text_edit_for_block_change(old, new)` helper (common char prefix,
  then common char suffix capped so prefix+suffix ≤ shorter length,
  then Insert / Delete / Replace on just the middle at the prefix
  offset). Not `text_diff::get_diff` — `get_diff` returns a
  whole-string insert for different-length replacements ("cat" ->
  "dogs" -> Insert of "dogs"), which corrupts the block.
- A server id is in `user` with different `frac_position` →
  `MoveBlock { new_frac_position }`. Same id, same text.
- An id in `user` but not in `server` (and non-empty) →
  `CreateNewBlockWithId`.
- Unchanged blocks → nothing.
- Duplicate ids in either list → `GenErr::DuplicateBlockId`.

The sketch's `PositionToInsert` on `CreateNewBlockWithId` is not
consumed by the letter builder — see `make_sketches_out_of_two_pages`
and `turn_all_edits_into_love_letters` in `drift_guard_and_ops`.

Out of scope on purpose:

- Snapshot construction. Emits sketches only. The caller turns
  sketches into `LoveLetter`s with `build_a_love_letter`.

## make_sketches_out_of_two_pages — the viewer's sketch generator

Called inside the viewer on finish, and re-exported so the caller can
generate sketches without opening the viewer (e.g. for auto-picked
pages).

    pub fn make_sketches_out_of_two_pages(
        server_page: &PageBlocksForTheViewer,
        kept_blocks: &[Block],
    ) -> Vec<LoveLetterSketch>

Steps:

- Alive filter. A kept block is alive unless it's a non-title server
  block whose text was blanked. The first server block (the title)
  always survives, even blanked.
- Frac assignment. Only survivors whose order is out of sequence
  relative to the server's own order get a fresh frac; the rest keep
  theirs. The kept survivors are the longest strictly-increasing
  subsequence of their server indices. Each non-stable survivor gets
  a frac minted by `smart_frac_index` between its neighbours in the
  kept order. Result: `MoveBlock` sketches only for the minimum set.
- `generate_sketches` on the alive-with-fracs list.

Panics on duplicate block ids (programming bug, not a runtime case).

Native tests cover the LIS edge cases and the four sketch outcomes
(edit / remove / move / create) plus the title invariant.

## drift_guard_and_ops::reconcile_planner — sketch → letter

Called by the viewer (via `block_diff_viewer_leptos`) on finish, and
re-exported for callers that want to build letters without opening the
viewer. Entry:

    pub fn turn_all_edits_into_love_letters(
        server_page: Arc<BossOfYrs>,
        edits: &[LoveLetterSketch],
        target_blocks: &[Block],
    ) -> Result<Vec<LoveLetter>, String>

Two passes:

- Edits, deletes, moves. `edit_to_love_letter` matches on the sketch
  and calls one of `edit_variant_to_love_letter` /
  `delete_variant_to_love_letter` / `move_variant_to_love_letter`.
  Each applies its change to a scratch copy of the server's page via
  `update_doc_so_next_edits_are_aware` (bookmark → apply → diff),
  then wraps the sketch + snapshot into a `LoveLetter`.
- Creates, in target order. `insert_variant_to_love_letter` uses
  `insert_new_block_with_preexisting_id`, so the viewer's target id
  becomes the doc id. Position is `Some(target_blocks[i-1])` — the
  previous target block. The first block is always the title, so
  there's never a create at index 0; that path panics with a clear
  message if it somehow fires.

Moves use `set_frac_position` (new yrs method), not `move_block`.
`move_block` recomputes the frac from neighbour ids at apply time;
in a batch a later move's neighbour might still be at its old
position. The viewer already computed the frac, so it's written
verbatim.

No self-check. Correctness is covered by tests.

Old files removed: `align_two_block_lists.rs` and
`main_for_turning_a_reconcile_choice_into_love_letters.rs` (LCS-based,
text-only, no reorder awareness, replaced).

Because the viewer calls this on finish, callers never see a sketch.
The viewer's `ViewerOutput` carries full `LoveLetter`s. The reconcile
page in cqrs aggregates those across pages and sends them in one
`reconcile_letters` request.

## Known gaps

- **Server handler for `CreateNewBlockWithId`.** The sketch variant is
  emitted and the letter builder produces the corresponding
  `LoveLetter`; whether the real server accepts it hasn't been
  verified.
- **`example_2` has no persistence and no server.** Every "commit"
  and "submit" is local. Data resets on reload.
- **New-block metadata.** `insert_new_block_with_preexisting_id` is
  called with an empty metadata string. Reconcile doesn't edit
  metadata today, so nothing is lost, but if that changes the sketch
  needs to carry it.

## Build and run

- `c t` at workspace root — native tests across the workspace.
- `c r` at workspace root — serves `example_app`.
- `trunk serve` inside `example_2/` — serves the prototype.
- `cargo check -p example_2 --target wasm32-unknown-unknown` — CI
  compile check for the prototype.
