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

    let left = RwSignal::new(vec_of_strings);
    let right = RwSignal::new(vec_of_strings);
    let (output, set_output) = signal(Vec::<String>::new());
    let handle = FinishHandle::new();

    <DiffViewer
        left_page=left
        right_page=right
        output=set_output
        handle=handle
        on_finished=Callback::new(move |_| { /* read output.get_untracked() */ })
    />
    <button on:click=move |_| handle.finish_and_write_the_result()>
        "Done"
    </button>

- `left_page` / `right_page` are `Signal<Vec<String>>`. Changing either
  remounts the inner viewer; edits and drags never survive a remount.
- `output` is a `WriteSignal<Vec<String>>` the viewer writes to when
  the caller asks to finish. Value is the right-side blocks in current
  visual order, LeftOnly rows skipped.
- `handle` is `FinishHandle`. Caller stores it, passes it in, and calls
  `handle.finish_and_write_the_result()` from any button. Internally
  it bumps a counter signal; the viewer's effect writes `output` then
  fires `on_finished`.
- `on_finished` is optional. Fires after every finish. Anything that
  reads `output.get_untracked()` inside it sees the fresh value
  because `output` is written before the callback runs.

Two `FinishHandle`s can drive the same viewer — clone it, call the
method on either. Useful when Done and the numbered close-box both
need to trigger a finish.

The only caller setup is one stylesheet link in `index.html`:

    <link data-trunk rel="scss" href="../block_diff_viewer_leptos/css/block_diff_viewer.scss" />

Everything inside the viewer is scoped under `.dv-root`. The stylesheet
uses `--dv-*` custom properties; a host can override any of them
without touching the file.

The viewer returns text only. It does not know about `block_id`,
`metadata`, or `frac_position`. If the caller needs those, it must
reattach them on its side — see "Known gaps".

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

- A server id is gone from `user` → `RemoveBlock`.
- A server id is in `user` with empty text → `RemoveBlock`.
- A server id is in `user` with different text → `EditBlock` with
  `TextEdit::Replace { old_text: server's, new_text: user's,
  position: 0 }`. Full replace, not fine-grained diff.
- A server id is in `user` with different `frac_position` →
  `MoveBlock { new_frac_position }`. Same id, same text.
- An id in `user` but not in `server` (and non-empty) →
  `CreateNewBlockWithId`, anchored via
  `PositionToInsert::Between { above, below }` where above/below are
  the nearest surviving ids in `user`'s order. If neither exists,
  `AtEnd`.
- Unchanged blocks → nothing.
- Duplicate ids in either list → `GenErr::DuplicateBlockId`.

Out of scope on purpose:

- Reorders of unchanged blocks (that's the `MoveBlock` path above and
  it works).
- Snapshot construction. Emits sketches only. The caller (finish
  flow) turns sketches into `LoveLetter`s with `build_a_love_letter`.

## Known gaps

- **Ids through the viewer.** `DiffViewer` returns `Vec<String>` in
  visual order. To key `generate_sketches` by id, the caller must
  reattach ids. Today `example_2` zips by position against
  `kept_blocks`. Correct under edits; wrong under drag (a dragged
  block's text lands at a different index). The real fix is to change
  `block_diff_viewer_leptos` to return `(id, text)` pairs or a paired
  API. Not done.
- **Server handler for `CreateNewBlockWithId`.** The sketch variant is
  emitted; whether the real server accepts it hasn't been verified.
- **`example_2` has no persistence and no server.** Every "commit" and
  "submit" is local. Data resets on reload.

## Build and run

- `c t` at workspace root — native tests across the workspace.
- `c r` at workspace root — serves `example_app`.
- `trunk serve` inside `example_2/` — serves the prototype.
- `cargo check -p example_2 --target wasm32-unknown-unknown` — CI
  compile check for the prototype.
