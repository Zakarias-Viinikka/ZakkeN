The drag work is integrated below. It's the same plan with drag slotted in, plus a few knock-on edits where drag touched other sections.

What changed, and what to veto before pasting:

Drop rule. I used the dragged row's centre, not the pointer position. This is ported from cqrs, where locate_drop is called with the dragged block's centre (d.current + size/2). It gives the same result as "mouse-up over row N" for equal-height rows. For tall rows it avoids a dead zone.
Row rects. Rows are display: contents wrappers, which have no bounding rect. Drop detection reads the right cell's rect instead. The plan calls this out explicitly because it's an easy trap.
Position-dependent presentation is now derived from visual order, not stored.
Hairlines come from neighbour comparison. The logic crate's starts_new_hunk becomes hunk_index: Option<usize>.
The bottom panel radius moves to a footer strip row, so data-last-row is gone.
The right gutter shows the live output position, and "moved to #N" chips follow it.
State is separated from the For loop. Per-row edit state is created once at mount, outside a <For> keyed by stable row id. Drag only permutes an order signal, so textareas are never remounted and the reset/stash state survives drags.
Drag reports a RowMoveEvent (moved, above, below). This is the same shape as cqrs MoveEvent, minus pages. The drag module never writes the order itself; the owner applies the event. That keeps feel_labs' black-box/container split.
Pure helpers. Slot-from-midpoints and apply-event are pure and natively tested.
Consequences of dragging whole rows (your spec; I'm only flagging them):
After a drag, the left column no longer reads in original order. The left gutter numbers are what stay truthful.
A CollapsedDeleteAndInsert row pairs two unrelated texts by stacking, so dragging it carries an unrelated left block along. The fix, if it feels wrong, would be splitting collapsed rows at drag start. I did not build that.
Not built: undo for drags, keyboard reorder, touch (mouse events only, like the existing code). Autoscroll is driven by mousemove only, so there's no timer loop.
Missing files. Your paste had mouse.rs, drag_state.rs, apply_move.rs, types.rs and layout.rs, but not drag_reorder_black_box.rs (the component that renders handles and attaches listeners) in either project, nor rapier_world.rs. I inferred the handler shape from mouse.rs. The prompt tells the cheap AI to read the missing component files first. If you'd rather I verify the handle shape myself, paste those.
my_block_content.rs. The prompt reuses its guarded value-sync and map_caret_across_edit for reset/undo-reset while the textarea is focused. It leaves the split/merge/navigate/focus plumbing out.
Task: Leptos block diff viewer workspace (port of zap's egui diff viewer)

You are turning /home/zakke/ProgStuff/ZakkeN/web_diff_viewer/ (a bare Leptos CSR scaffold) into a Cargo workspace with three crates. The result is a reusable side-by-side diff component for two pages of text blocks. The right side is editable, and whole rows can be reordered by dragging a handle on the right side. This is a spike/template. There is no persistence, no server, and no integration with other projects. Do not touch cqrs_state_synchronization/, zakken_server/, or zap/. You may read the cqrs and feel_labs drag code listed below.

Read first
/home/zakke/ProgStuff/ZakkeN/coding_style.md.
Zap sources for porting: /home/zakke/ProgStuff/egui/zap/zapgui/app_core/src/text_diff/{mod,pair,rows,post_process,similarity,moves}.rs, /home/zakke/ProgStuff/egui/zap/zapgui/every_page_ever/src/ReusableComponents/diff_viewer/{ui,mod}.rs, and /home/zakke/ProgStuff/egui/zap/CHAIN_REVIEW_UI.md.
The scaffold files: Cargo.toml, index.html, local_cargo.sh, rust-toolchain.toml, css/global.scss, src/main.rs, src/app_main.rs.
Drag sources, read before writing anything in the drag section:
cqrs (mature): everything under /home/zakke/ProgStuff/ZakkeN/cqrs_state_synchronization/web_interface/src/drag_reorder_container/. This includes drag_reorder_black_box/{mod,types,drag_state,mouse,apply_move,layout}.rs and drag_reorder_black_box.rs itself, which is the handle component and listener wiring. It was not in the paste the planner saw, so read it yourself.
feel_labs (lab-scale): /home/zakke/ProgStuff/ZakkeN/messing_around/feel_labs/src/drag_reorder_lab/, including its black box component wherever it lives in that folder.
Port feel_labs for the core and cross-check cqrs for anything it added. Take the drag-handle component shape, the mouse-down/move/up flow, DRAG_THRESHOLD = 4.0, and the drop-event report shape (MoveEvent { moved, above, below }).
Leave behind: rapier, cascade, the phase machine (Idle/Waiting/Moving), vacuum, ghost, stored blocks, the two-page concept, the y-interpolation loop, and the "title never moves" rules.
From my_block_content.rs, take only the guarded value-sync effect and map_caret_across_edit. Leave the split/merge/navigate/focus plumbing, which is for a block editor.

Port logic from similarity.rs, pair.rs, and moves.rs, and take visual constants from ui.rs. Do not port egui's three-pass pre-layout. The browser does layout here.

1. Workspace shape
The root Cargo.toml becomes a virtual workspace with resolver = "3" and edition 2024. Put shared dependency versions in [workspace.dependencies].
Keep rust-toolchain.toml at the root and add the wasm32-unknown-unknown target to it.
Logic crate. Pure and target-agnostic, with no leptos or web-sys. It depends only on similar and mirrors zap's app_core::text_diff. Suggested name: block_text_diff. cargo test -p on it must run natively.
Component crate. A library depending on leptos (csr feature), the logic crate, web-sys, and wasm-bindgen. Let the compiler tell you which web-sys features are needed. Expect Element, HtmlElement, HtmlTextAreaElement, MouseEvent, KeyboardEvent, DomRect, NodeList, Window and Document. It ships its own stylesheet at css/block_diff_viewer.scss. Suggested name: block_diff_viewer_leptos. Keep every DOM call out of the pure modules so cargo test -p on this crate still runs natively.
Example crate. The trunk app: index.html, css/global.scss, src/main.rs, src/app_main.rs, and the sample pages. Move the existing scaffold files into it and keep the router, since it's the human's template pattern. Its index.html links the global scss, the component crate's scss via a ../ path, and rel="rust".
Adapt local_cargo.sh: for run, cd into the example crate and exec trunk serve with the remaining args. Everything else goes to cargo at the workspace root. trunk build must work from inside the example crate.
Follow coding_style.md: long descriptive names, main_for_* for caller-pov entry files, lib.rs and mod.rs as plumbing only, one concern per file, entry function at the top of each file, and comments that explain why. Every deviation from zap, cqrs, or feel_labs gets a comment saying why.
2. Logic crate: the hard part
Output model (shape is a suggestion)

The entry returns an ordered list of rows. Each row has an optional left slot, an optional right slot, a hunk_index: Option<usize>, and a relationship enum. hunk_index is None on Equal rows, and all rows of one hunk share the same value. The relationship values are: Equal, Edit, CollapsedDeleteAndInsert (both sides present but unrelated), LeftOnly, and RightOnly.

A slot holds only the block index into the caller's input, not the text.
Left markers are: Unchanged, Deleted, Edited, MovedAway (with partner right index and an identical flag).
Right markers are: Unchanged, Inserted, Edited, MovedHere (with partner left index and an identical flag).
Hard invariant: reading the left slots top to bottom yields every left block exactly once in order, and the same for right. The component relies on this so that, before any drag, the output equals the right page in its original order. Property-test it.
The logic crate knows nothing about row ids or dragging. The component assigns row_id = index in this initial list.
Algorithm
Blank blocks (empty or whitespace-only) are canonicalised to one key, so blank equals blank, as zap's same_text does. Diff the two Vec<String> as item sequences with similar::capture_diff_slices (Myers, the zap precedent). It already merges adjacent delete+insert into Replace. Still treat any run of consecutive non-Equal ops as one hunk, defensively.
Each Equal item becomes an Equal row.
For each hunk you have a left slice (k blocks) and a right slice (j blocks). Run zap's alignment DP: a score table where a diagonal step is allowed only when pair similarity ≥ the pairing threshold, with the similarity as gain. Back-trace into an ordered step list.
Cache the similarity matrix. Compute each pair once and reuse it.
Prefilter with a cheap check. Blocks can be long paragraphs and the LCS is quadratic in tokens. Use dice_over_tokens to skip hopeless pairs before running the LCS.
Lower the cell cap to about 10,000. Zap's 100,000 was tuned for short lines. Beyond the cap, fall back to zap's pair-by-index.
Walk the steps. Each diagonal is an anchor: an Edit row, with both sides in one row. Each maximal run of non-diagonal steps is an unpaired segment. Collect its deleted left blocks and inserted right blocks separately, in order.
Collapse (the load-bearing change). A segment emits max(#dels, #inses) rows. Row r takes dels[r] and inses[r] if each exists. Both present means CollapsedDeleteAndInsert (red left, green right on one row); one present means LeftOnly or RightOnly. Gaps only ever appear at the tail of a segment where one side outnumbers the other.
Why alignment is automatic. An anchor row is always alone in its row, and segments sit wholly between anchors, so Edit pairs can never misalign. There is no separate gap-insertion step. If you find yourself writing one, the model is wrong.
Assign hunk_index to every row emitted from a hunk. The component draws hairlines from it (see section 3).
Similarity tuning for short blocks

Port zap's tokenize, weighted_lcs, similarity, dice_over_tokens, edit_distance, and the inline-span functions largely verbatim. Then fix these block-specific problems:

Single-word blocks score all-or-nothing under token LCS ("color" vs "colour" scores 0). Define "short" as either block having fewer than ~4 tokens or fewer than ~24 non-whitespace chars. For short pairs, score = max(token score, 1 − edit_distance/longer_len) and use a stricter pairing threshold (~0.7). Long blocks keep zap's 0.5.
Use one threshold, and make it conservative. Zap's separate EDIT_THRESHOLD is gone. With collapse, a "paired but not an edit" row looks identical to a delete and an insert sharing a row. A false pairing is costlier than a false non-pairing, because unpaired del/ins already share a row. Put every threshold as a named constant at the top of the similarity file with a comment saying this.
Raise the inline-span max chars from 2,000 to roughly 8,000 for prose, and keep the "mostly covered, so show no spans" guard.
Expected results to assert in tests: "color"/"colour" pairs; "Yes"/"No" does not; "Chapter 1"/"Chapter 2" pairs; "Intro"/"Outro" does not.
Moves (ported, simplified)

Run this as a post-pass over unpaired deleted and inserted items across the whole document. It does not change row layout; it only swaps Deleted→MovedAway and Inserted→MovedHere.

Match exact-equal blocks first via a hash map. This is the common case of a reordered paragraph.
Then fuzzy-match remaining candidates by similarity ≥ ~0.75, only while D×I stays under a cap.
Require size gates: ≥ ~8 non-whitespace chars for exact matches and ≥ ~24 for fuzzy ones. This avoids "Yes"/"}"-style noise.
Match greedily, best score first, one-to-one.
Record identical so the label can say "(edited)", as zap does.
Moves are classified once, from the original pair. User drags never re-run this. See "frozen" in section 3.
Logic tests (named scenarios)
Identical pages produce all Unchanged rows and no hunks.
The human's example: left [bla, bla3], right [bla, bla2] with dissimilar bla3/bla2 gives Equal, then one shared Deleted|Inserted row. With similar texts it gives one Edit row.
4 deleted vs 2 unrelated inserted gives 4 rows, with the first two shared and the last two right-empty. (This is the D1–D4 example.)
Stack-then-edit: left [A, X, Y, E], right [A, E'] gives right-empty rows for X and Y, then E|E' as an Edit row. Also test the inverse, with more inserts than deletes before an edit.
hunk_index is None on Equal rows, shared within a hunk, and different between two hunks separated by an Equal row.
The ordering invariant, run over randomized small inputs using a tiny hand-rolled LCG so no rand crate is needed.
Moves: a reordered long block is detected with correct partner indices, and a short generic block is not.
Blank blocks, one side empty, and both sides empty.
3. Component crate
Caller pov (document this in the crate-level doc comment)
A DiffViewer component takes left_page and right_page as Signal<Vec<String>> (use #[prop(into)]), output_right_side as a WriteSignal<Vec<String>>, a handle, and an optional on_finished Callback<()>.
A small Copy handle type (constructor plus a long-named "finish and write the result" method) wraps one counter signal. The caller creates it, passes it in, and calls the method from any button anywhere. The component runs an Effect on the counter that collects the output, calls output.set, then runs on_finished. That Effect must skip its first run and must read everything untracked. This is an owner+handle split in Leptos form, so file it under shared_owner/ per the style guide.
Output = the current right-side text of each row in the current visual order, top to bottom, skipping rows with no right slot (LeftOnly). Write this as a pure function with a native unit test: untouched order yields the right page; a permuted order yields the permuted texts; LeftOnly rows contribute nothing wherever they sit.
At mount the component sets the output signal to an empty Vec.
Inputs are snapshotted per mount. The outer component re-renders an inner component when either input signal changes, so all per-block and per-row state is fresh, and edits and drags never survive a re-open.
The only caller setup is one stylesheet <link> line, documented in the crate doc.
Row model at mount

The inner component computes the diff once and builds:

A frozen immutable Vec of rows (Arc'd, captured by clone): row_id (index), left/right slots, markers, relationship, hunk_index. Nothing re-diffs, ever. Typing and dragging do not touch it. Markers, classifications and per-row pairings stay as computed at mount.
visual_row_order: RwSignal<Vec<usize>>, starting as 0..n. This is the only thing dragging changes.
Per-row edit state: one RwSignal<BlockEditState> for each row that has a right slot, held in a Vec<Option<RwSignal<…>>> indexed by row_id. Create these once at mount, outside the row list. Do not create them inside the <For> children closure.
live_right_number_by_row: Memo<Vec<Option<usize>>>: for each row, its 1-based position among rows that have a right slot, derived from the order. The right gutter number and the "moved to #N" chip both read it. The partner right block index maps to a row id through a static lookup built at mount.
Rows render through a <For> keyed by row_id (not by position or text), iterating visual_row_order. This keeps the DOM, textareas and state alive across reorders.
Per-block edit state
Make a pure, Leptos-free struct holding original, current, and stashed_edit: Option<String>, with unit tests that run natively. It has five operations:
user_typed(text) sets current and clears the stash.
reset() runs only when current ≠ original: stash current, set current = original.
undo_reset() runs only when a stash exists: current = stash, clear the stash.
is_edited() is current ≠ original.
can_reset() equals is_edited(), and can_undo_reset() is "a stash exists".
Invariant: a stash exists only while current == original.
Test the human's exact scenario sequence, plus: typing the original text back clears "edited"; reset then undo then reset works again; ops that don't apply are no-ops.
Reset and drag are orthogonal. State is keyed by row_id, never by position. Reset changes text, drag changes position, and neither reads the other.
Document every toucher in the doc comment of its shared_owner/ file: literal path, why, when. Touchers include textarea input, buttons, edited dot, live highlights, edited counter, and the finish Effect.
Blocks cleared to empty stay as empty strings in the output. The component never drops blocks.
Layout technique
One CSS grid with two minmax(0,1fr) columns and a 14px column gap. Each row is a display: contents wrapper containing exactly two cells (left, right), and both are always rendered, with a hatched placeholder when a side is empty. Row heights then equalise for free.
Put data-row-id on both cells (and the wrapper), and a class that identifies the right cell. A display: contents element has no bounding rect. Anything that needs a row's geometry (drag) must read the right cell's rect, since the grid equalises heights.
The panes' rounded-panel look comes from per-cell borders and corner radii. A sticky header row takes the top radius. A footer strip row (two thin cells at the end of the grid) takes the bottom radius. Do not use "last row" selectors or attributes, because the last row changes when the user drags.
Hairlines are derived from visual order: draw a hairline above a row when the row has a hunk_index and the previous visual row does not share it. Never draw one on the first visual row. Emit it as a data- attribute computed per row from the order and hunk_index. Before any drag this matches zap's "hairline at the start of each hunk".
Row hover uses a pseudo-element overlay on both cells. It is suppressed while a drag is in progress.
The viewer fills its parent's height with a scrolling body (this is the drag scroll container), so the caller sizes it.
Editable cell
Use the CSS autogrow-textarea technique. A wrapper stacks, in one grid cell, a mirror div and the textarea.
The mirror renders the same text invisibly, with a trailing extra character so a final newline is counted. It sets the height, so there is no JS measuring. This deliberately replaces the scroll_height measuring in my_block_content.rs. Say so in a comment.
The mirror and textarea must have identical font, size, line-height, padding, border, letter-spacing, tab-size, white-space: pre-wrap, and overflow-wrap: anywhere. The textarea has overflow: hidden, no resize, and a transparent background. If heights drift, this equality is the first thing to check.
Live inline highlights.
For rows with relationship Equal or Edit only, a per-row memo computes inline change spans between the left text and the current right text.
The left cell renders text as segments with highlighted spans.
The right cell renders the same segmentation inside the mirror layer, which sits behind the textarea. The mirror text is transparent and only the highlight backgrounds show.
Never highlight CollapsedDeleteAndInsert rows, since those texts are unrelated.
The segmenting function is pure and tested. If span ranges don't fit the text, draw plain and log, as zap's build_job does.
Build this after everything else works. If the overlay won't line up, ship left-side highlights only.
Binding. Port the guarded value-sync from my_block_content.rs (install_value_sync_effect). An Effect watches the state's current text. It sets the DOM value only if it differs from the textarea's value, and if the textarea is focused it restores the caret via map_caret_across_edit (UTF-16 offsets, as selection_start uses). The input event updates state. Resets and undo-resets then update the DOM, and normal typing never moves the caret. A bare prop:value binding is not enough here.
Cell anatomy
Left cell: gutter (40px, dim, right-aligned, the original left block number, which never changes) | kind bar | content.
Right cell: gutter (~52px) holding a drag handle (below) then the live output position number | kind bar | a fixed 14px "edited" lane | content | a reserved ~60px actions lane. The placeholder variant (LeftOnly rows) has no handle and no number.
The left bar is red for Deleted, blue for Edited, and purple for MovedAway. The right bar is green for Inserted, blue for Edited, and purple for MovedHere. No bar when Unchanged. Each bar has a title tooltip.
The edited marker is a small amber circle with a soft ring, which is a different shape from the bars and a free colour since wrap markers are dropped. It appears while is_edited(). The lane is always reserved so text never shifts.
Reset and undo-reset are small 24px icon buttons in the actions lane. They use inline SVG (lucide-style rotate-ccw for reset and rotate-cw for undo) with titles and aria-labels, because unicode arrows render inconsistently.
They are fully revealed on row hover or focus-within.
They stay visible at ~70% opacity whenever the block is edited or has a stash, so state is readable at a glance.
Pristine blocks hide them. Disabled buttons are dim with a not-allowed cursor.
Mousedown on the buttons must prevent_default so they don't steal the textarea's caret.
Move chip. A small purple pill in the content area's top-right: "moved to #N" or "moved from #N", plus "(edited)" when applicable. It is positioned inside the content lane so it never collides with the actions. This replaces zap's extra-height strip, so move rows don't grow. "Moved to #N" shows the live right number of the partner row. "Moved from #N" shows the fixed left number.
Empty-side placeholders are full-height with a faint diagonal hatch.
Drag reorder (new)

Concept. The unit of drag is the row. Dragging is initiated from a handle on the right cell, but the row (both cells) is what moves. If only the right cell moved, the left/right pairings would go visually stale. Say this in a comment on the handle component. One consequence is that the left column no longer reads in original order after a drag. The left gutter numbers stay truthful, so keep them visible.

Which rows. Rows with a right slot (Equal, Edit, CollapsedDeleteAndInsert, RightOnly) get a handle. LeftOnly rows get none, but they are ordinary rows for drop purposes. They just contribute nothing to the output.

Handle. A <button type="button"> with a six-dot grip SVG, aria-label and title, cursor: grab. It is faintly visible at rest, brighter on row hover, and it lives in the right gutter. Only the handle starts a drag, never the textarea. On mousedown call prevent_default (this prevents text selection, and the textarea keeps whatever focus it had).

Suggested file split (names are suggestions, but keep one concern per file, mod.rs plumbing only):

shared_owner/visual_row_order.rs: the order signal and the full list of touchers (the keyed For, drop apply, finish Effect, position memos, hairline derivation).
shared_owner/drag_in_progress_feedback.rs: three coarse signals, described below, and their touchers.
A drag folder with:
an entry file for the handle component plus window listener setup;
a DOM-reading file (snapshot rects, scroll reads, all web-sys error-wrapped);
a non-reactive session struct file;
a pure math file with native tests.

State split.

Non-reactive bookkeeping in a StoredValue<Option<DragSession>>. It holds: dragged row_id, start pointer position, start scrollTop, last pointer client-y, moved flag, and the row-rect snapshot.
Reactive signals that views read, kept coarse so only the affected rows re-render:
dragged_row_id: Option<usize>, which changes on start and end;
dragged_translate_y: f32, which only the dragged row's style closure reads;
drop_indicator: Option<(row_id, Before|After)>, which changes only when the slot changes.

Flow (port the shape of feel_labs mouse.rs; note the coordinate difference in a comment):

Mouse-down on a handle, primary button only, and only if no session exists. Snapshot every right cell in DOM order as (row_id, top, bottom) in scroll-body content coordinates: rect.top − body_rect.top + body.scrollTop. Store the session with moved = false. Set no signals yet, so a click with no movement does nothing, as in cqrs.
Mouse-move. Listen on window (use window_event_listener, removed in on_cleanup), guarded by "session exists". This is deliberate: the existing code listens on the stage, but a release outside the viewer must not strand a session.
Until the pointer has moved more than DRAG_THRESHOLD = 4.0 px on either axis, do nothing. Once it passes, set moved, set dragged_row_id, and set a data-dragging attribute on the viewer root.
Compute dy = (pointer_client_y − start_pointer_y) + (scrollTop_now − scrollTop_at_start). The scroll term keeps the lifted row under the pointer if the body scrolls. Set dragged_translate_y.
Compute the dragged row's centre in content coordinates (snapshot centre + dy). Compute the insertion slot: the number of other rows (dragged excluded, in visual order) whose midpoint is above that centre. This is the "centre of the dragged item decides" rule from cqrs locate_drop.
Set drop_indicator to None if the slot equals the dragged row's current index (a no-op). Otherwise set it to the other row at that slot with Before, or the last other row with After when the slot is past the end.
Autoscroll: if the pointer is within ~48px of the body's top or bottom edge, scroll_by a few px per mousemove event. A stationary pointer won't scroll and the wheel works natively. This is a deliberate no-timer simplification.
Also listen to scroll on the body, and rerun the dy and slot computation with the last pointer position while a drag is active.
Mouse-up (window). If the drag moved and the slot differs from the current index, build a RowMoveEvent { moved_row_id, above_row_id: Option, below_row_id: Option } (the neighbours in the new order, same shape as cqrs MoveEvent minus pages) and pass it to the on_row_moved callback. In every case, clear the session, all three signals and data-dragging. The drag module never writes visual_row_order. The owner (the viewer core) applies the event, as feel_labs' on_move does: remove the moved id, insert after above, or at index 0 when above is None.
Escape or window blur cancel: clear everything and report nothing.

Pure math (native unit tests, in a Leptos-free file):

insertion_slot_for_dragged_center(other_row_midpoints_in_order, dragged_center) -> usize.
Build the RowMoveEvent from the order, the moved row and the slot. It returns None for a no-op.
Apply the event to the order.
Test: move down, move up, to the very top, to the very bottom, no-op slot, LeftOnly rows interleaved (moving past them is legal), variable row heights, and a randomized sequence of moves (tiny LCG) that always leaves the order a permutation of 0..n.

Visual feedback (no physics, no animation loop):

The dragged row stays in flow, and its two cells get transform: translateY(dy). The transform goes on the cells because the display: contents wrapper has no box. The cells also get a higher z-index (below the sticky header), a drop shadow, a faint outline, and pointer-events: none. The vacated slot shows the panel background through it.
The drop indicator is an inset box-shadow line (2px, --dv-drop-indicator) on the top edge of the target row's cells for Before, or the bottom edge for After. It causes no layout shift.
While data-dragging is set: cursor: grabbing everywhere, user-select: none on the root, pointer-events: none on textareas, and hover overlays and action reveals off.
No transitions on position. On drop, the row lands by DOM reorder and everything snaps. Do not build FLIP animation. No transition on the lifted transform either, because it must track the pointer exactly. At most, a ~100ms transition on non-positional properties such as shadow or opacity.

Pitfalls specific to drag:

Reordering a keyed <For> moves DOM nodes. A focused textarea in a moved row may lose focus, which is acceptable.
All web-sys reads (rects, scrollTop, dataset) go through small wrappers that report message, method and file path on failure.
Visual design (make it look good)

A dark, zap-feeling look with a bit more polish.

Token	Value
page bg	
#0f1115
panel bg	
#0a0b0e
panel border	rgba(110,110,120,.28)
hunk hairline	rgba(150,150,160,.22)
text / dim	
#dcdcdc / 
#787878
delete bar	rgb(210,80,80)
insert bar	rgb(80,180,80)
move bar	rgb(170,120,220)
edit bar	rgb(100,160,220)
user-edited	rgb(232,185,90)
drop indicator	rgb(120,190,255)
drag lift shadow	0 10px 28px rgba(0,0,0,.6)
Define these as --dv-* custom properties scoped to the viewer so a host can theme them. Write literal rgba tint tokens rather than using color-mix, to be safe with trunk's sass.
Row tint is 6–8% alpha of the kind colour. Inline highlights are ~28% alpha: blue on both sides for Edit rows.
Use monospace by default (JetBrains Mono, Fira Code, then ui-monospace), exposed as a variable. Use 13.5px, line-height 1.55, and cell padding of about 8×10px.
Panes have 10px radii. The bar is 3px with ~5px vertical inset and rounded ends.
The right pane is slightly lighter, with an inset blue focus ring on focus-within.
Transitions are ~120ms for hover and reveal effects. Use thin dark scrollbars and color-scheme: dark.
Above the panes, a toolbar holds a compact legend (coloured dots for deleted, inserted, edited, moved, and "edited by you") and a live "N edited by you" count. The sticky headers read "Before" and "After — editable".
Optional last polish: on mount, scroll the first changed row into view with a little margin, like zap's scroll_to_change_once.
Dropped from zap / cqrs (note this in comments)
From zap: syntax highlighting and themes, wrap markers, the egui right-click copy menu (the browser's native selection and menu replace it), right_accent, and the three-pass pre-layout.
From cqrs/feel_labs: rapier, cascade, the Idle/Waiting/Moving phase machine, vacuum, ghost, stored blocks, pages, the y-interpolation loop, and the title rules.
4. Example app
Dark global styling. Remove the old centred-body flex layout, since the viewer needs room.
Top bar: a title, a short hint, and a primary Done button that calls the handle.
Below it, the viewer in a container sized to the remaining viewport height. Make the content tall enough, or the container short enough, that the body scrolls, so drag and autoscroll can be tried.
Phase signal: Editing or ShowingResult. on_finished flips it to ShowingResult. The result view lists the returned blocks as numbered cards, in the returned order, with an Open again button that re-mounts the viewer from the original, unmodified inputs.
sample_pages file: two ~14–16-block prose pages (release-notes or README flavour) that exercise: several Equal blocks; a clear edit (blue); the "bla" case (dissimilar swap, collapsed red|green); a 4-deleted vs 2-inserted stack (gives LeftOnly rows); an edit directly after a stack, to show alignment; a reordered long paragraph (purple move); a one-word "color"/"colour" edit; one blank block; one very long paragraph with a small change, to show inline highlights and wrapping.
5. Build order
Workspace skeleton: move the scaffold, confirm trunk build passes with the placeholder app.
Logic crate with all its tests passing before any UI.
Component:
Edit-state machine and tests.
Static row rendering through the keyed <For> over visual_row_order, with styles and no editing. Build the order-signal plumbing now, even though nothing reorders yet.
Editable cells.
Reset / undo-reset buttons.
Handle and finish, with the order-based output function and its test.
Drag reorder: the pure math and tests first, then the DOM wiring, then the visual feedback.
Live highlights.
Polish.
Example app.

Done criteria:

cargo test -p on the logic crate and the component crate passes natively.
trunk build succeeds in the example crate.
cargo run via the wrapper serves the app.
Manually walk the reset / undo-reset / edit-again sequence.
Manually walk drag:
drag a row down and up, and drag one past a LeftOnly row;
drag an edited block, and drag a reset block;
reset after a drag still works;
release outside the viewer, and press Escape mid-drag, and confirm nothing is stranded;
finish and confirm the output order matches the visual order;
reopen and confirm edits and order do not survive.
6. Pitfalls
A Leptos Effect runs once on creation, so the finish Effect must not write output on that run.
similar needs Eq + Hash items, so diff &str.
Hover still works on display: contents wrappers, but their rects are empty. Use the cells.
If row heights look off by a pixel or two, compare mirror and textarea metrics, including box-sizing.
Don't recompute the diff on keystrokes or on drags. Only the span memo reacts to typing, and only the order signal reacts to drops.
Key the <For> by row_id and create per-row state outside it. Otherwise a reorder remounts textareas and loses state.
Window-level listeners must be removed on cleanup, and the drag session must be cleared on every exit path.
Wrap any web-sys calls so errors report the message, method, and file path, per the style guide's three-part convention.
State your final choices (crate names, thresholds you changed, anything you deviated on from the plan or from cqrs/feel_labs) in a short summary at the end.
