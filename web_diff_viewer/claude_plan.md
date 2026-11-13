# Claude's plan for the reconcile page port

**HISTORICAL PLAN, AS-BUILT NOTE AT TOP.** This file is the plan the
reconcile port was handed off against. Read the "As built" section
below first; the long writeup further down is the original plan and
has drifted from the code in several places (noted inline where I
caught them).

## As built (2026-11-13)

The reconcile page at cqrs's `/reconcile` route runs the full port:

- `block_diff_viewer_leptos::DiffViewer` props are `server_page:
  Signal<Option<PageBlocksForTheViewer>>`, `client_page:
  Signal<PageBlocksForTheViewer>`, `kept_side: Signal<PickedSide>`,
  `server_page_boss: Signal<Option<Arc<BossOfYrs>>>`,
  `output: WriteSignal<ViewerOutput>`, `handle: FinishHandle`,
  `on_finished: Option<Callback<()>>`.
- `ViewerOutput { kept_blocks: Vec<Block>, love_letters: Vec<LoveLetter> }`.
  When `server_page_boss` is `Some`, the viewer builds full love
  letters via `turn_all_edits_into_love_letters`. When `None`, letters
  come back empty (diff-only mode). Client-only pages produce one
  `CreateNewPage` letter.
- The viewer runs the whole chain on finish: alive filter → LIS frac
  assignment → `generate_sketches` → `turn_all_edits_into_love_letters`.
  Callers never see sketches. `make_sketches_out_of_two_pages` is the
  pure sketch half.
- `drift_guard_and_ops::reconcile_planner` is called from inside the
  viewer, not from cqrs. Entry: `turn_all_edits_into_love_letters(
  server_page, edits, target_blocks)`. Two passes (survivors then
  creates in target order). `BossOfYrs::set_frac_position` was added
  so moves write the viewer's frac verbatim, instead of
  `move_block` recomputing it from neighbours.
- Title rule: server block at index 0 is never removed. Blanking it
  becomes an `EditBlock` with empty text. Non-title blocks still
  remove.
- The old LCS planner (`align_two_block_lists.rs`,
  `main_for_turning_a_reconcile_choice_into_love_letters.rs`) was
  deleted. Same for the reconcile page's `diff_overlay.rs`,
  `mismatched_card.rs`, `resolve_choice.rs` — the UI was ported to
  `example_2`'s shape (chooser / pick screen / inline viewer / Submit
  with review). New files under `reconcile_page/`:
  `inline_diff_viewer_host.rs`, `pick_which_side_wins_screen.rs`,
  `nothing_selected_panel.rs`. Styles in
  `web_interface/css/reconcile_page.scss`.
- `finish_reconcile` shrank to verify → encode → send → wipe. It does
  not build letters.
- One thing the plan got wrong: it said the caller should aggregate
  per-page letters at Submit. The final shape has the viewer produce
  them per page (Done / X); cqrs just concatenates and sends. This is
  because the viewer needs the boss to snapshot, and snapshotting is
  exactly the work `turn_all_edits_into_love_letters` does.
- Another: the plan didn't call out that any `pairs_signal.update(...)`
  while the viewer is open remounts it, wiping edits. The final
  `ready_view` writes `pair.last_output` directly (not via
  `pairs_signal`) for exactly this reason.

## The original plan writeup follows

It has been lightly updated in place where it contradicted the
as-built, but treat the section above as the source of truth.

---

# Claude's plan for the reconcile page port

**SUPERSEDED body.** During implementation the design changed several
times:

- The viewer grew id-carrying props (`server_page` / `client_page` /
  `kept_side` / `server_page_boss`) and emits `ViewerOutput {
  kept_blocks, love_letters }`. No caller-side reattach-by-text step,
  no caller-side sketch-to-letter step either — the viewer builds
  letters itself when it has a `server_page_boss`.
- Frac assignment (LIS + `smart_frac_index`) moved inside
  `block_diff_viewer_leptos`, so only the minimal set of `MoveBlock`
  sketches is emitted.
- `drift_guard_and_ops/client::reconcile_planner` was rewritten
  around `turn_all_edits_into_love_letters`. The old LCS planner was
  deleted. `BossOfYrs::set_frac_position` was added so moves write
  the frac verbatim instead of recomputing from neighbours.

The original writeup follows. Where something was revised in place it
is marked "(Revised: ...)".

---

I found several places where the pre-plan would break in practice, so
I changed them. The hand-off prompt follows the list.

What I changed from the pre-plan

get_diff can’t produce the fine-grained edits. It only represents a pure insert, a pure delete, or an equal-length replace. For a different-length replacement like “cat” → “dogs” it returns an Insert of the whole new string, which corrupts the block. The generator gets its own prefix/suffix edit function and doesn’t depend on text_diff.
Sketch positions can’t be trusted for creates. Two consecutive new blocks between the same server neighbours get identical anchors and end up tied or reversed. Letters are built by an applier that works on the scratch doc and recomputes each insert position from the scratch’s current state. Because the client is wiped and re-pulled after sending, client ids never need to survive, so CreateNewBlockWithId is never put in a letter. This also avoids the unverified server handler.
Moves are minimal via longest increasing subsequence (LIS), not “monotonic with neighbours”. The LIS runs over each surviving block’s server index, and client fracs are ignored entirely. (Revised: the LIS now runs inside `block_diff_viewer_leptos`, in the step that builds sketches.)
Id reattachment is no longer needed. The viewer's props and output carry ids. No fuzzy matching, no fresh-id fallback. (Revised: this bullet replaces the original "chained reattachment" bullet.)
Blank-block fix. A blank block that was already blank must not become a RemoveBlock, otherwise an untouched “Server” pick would emit letters.
Submit is a two-step review/confirm, and letters are built at Submit. Errors show before anything is sent.
The viewer must never remount on Done. Example_2 does remount it, because its closure tracks pairs.

You are implementing a feature in the ZakkeN repo. Read ZakkeN/coding_style.md FIRST and follow it for every file and name you create (long descriptive names, main_for_* for caller-pov entry files, "Caller pov:" header comment on every file, main entry at the top of the file, mod.rs/lib.rs are plumbing only). Also read web_diff_viewer/STORED_AI_CONTEXT.md. You may change block_diff_viewer_leptos, generate_love_letters, drift_guard_client's reconcile_planner, and the reconcile_page folder in cqrs_state_synchronization/web_interface. You may also update example_app and example_2 to compile against the new block_diff_viewer_leptos API.

GOAL
Port example_2's UI shape (web_diff_viewer/example_2) into the real /reconcile page. Grow block_diff_viewer_leptos's public surface so it carries ids, owns fracs, and produces love-letter sketches. Replace the caller's letter generator with the sketch-driven one. Preserve: route, Phase (Loading/NotLonewolf/Ready/Error), load_and_classify's fingerprint classification, NotLonewolfView, error rendering, target versions, mount_fingerprints, the verify-nothing-changed guard, DriftVersionMismatch error mapping, send-batch -> wipe -> reload, client-only pages sent as CreateNewPage letters.

THE INVARIANT TO REASON BY
After the batch is sent, the client is wiped and re-pulled from the server. So only the SERVER's final state matters. Client block ids and client fracs never need to survive. A page is correct iff the scratch doc (server snapshot + our letters applied) ends with block texts, in order, equal to the target list. Ids, moves and text-matching only decide how minimal the letters are. The self-check at the end of building letters is the backstop; if any heuristic guesses wrong, the self-check either passes (harmless) or errors (nothing is sent). Use this to debug: when a self-check fails, print both lists and look at ordering, then at the alive-filter.

STEP 0 - extend block_diff_viewer_leptos (web_diff_viewer/block_diff_viewer_leptos)
Add deps: generate_love_letters (path ../generate_love_letters), love_letter (path ../../love_letter), smart_frac_index (path ../../smart_frac_index), yrs (path ../../yrs).

0a. Public structs. Add two types the caller hands in and gets back:
 - `PageBlocksForTheViewer { page_id: String, is_main_menu_page: bool, blocks: Vec<Block>, blob: Vec<u8> }` where `Block` is re-exported from `generate_love_letters`. (Revised: `blob` was added when the viewer started building love letters — CreateNewPage needs it.)
 - `ViewerOutput { kept_blocks: Vec<Block>, love_letters: Vec<LoveLetter> }` where `LoveLetter` is re-exported from `love_letter`. (Revised: was `sketches: Vec<LoveLetterSketch>`. The viewer builds letters now, not sketches.)

0b. Props. `DiffViewer` takes:
 - `server_page: Signal<Option<PageBlocksForTheViewer>>` (None means client-only)
 - `client_page: Signal<PageBlocksForTheViewer>`
 - `kept_side: Signal<PickedSide>` (Revised: was missing from the pre-plan; added so the viewer knows which side is editable.)
 - `server_page_boss: Signal<Option<Arc<BossOfYrs>>>` (Revised: added so the viewer can build love letters itself.)
 - `output: WriteSignal<ViewerOutput>`
 - `handle: FinishHandle`
 - `on_finished: Option<Callback<()>>`

0c. Finish behavior. On finish: kept_blocks = current right-side blocks in visual order (each entry retains the id from the row it came from). If `server_page_boss` is `Some`, also build `love_letters` via `turn_all_edits_into_love_letters` from `drift_guard_ops::reconcile_planner`. If `server_page` is `None` (client-only), emit one `CreateNewPage` love letter built from the client blob. Write to output, fire on_finished.

0d. LIS frac assignment. Lives inside this crate, called from the same finish effect, before generate_sketches. Pure helper with native tests.

STEP A - generate_love_letters
A1. Fine-grained edits without text_diff::get_diff. Common char prefix, then common char suffix capped so prefix+suffix ≤ shorter length, then Insert / Delete / Replace on just the middle at the prefix offset. Not `get_diff` — it's unsound for different-length replacements.
A2. Blank rule: remove only if the server's text is not empty AND it's not the title (block at index 0). Blank on both sides is a no-op. Title blanking becomes an EditBlock.
A3. Unit tests for cat->dogs, abcXdef->abcYYdef, aa->aaa, abcabc->abc, trailing newlines, emoji, blank-both-sides, blanked-title-is-edit, blanked-non-title-is-remove.

STEP B - sketch-to-letter (drift_guard_and_ops/client/src/reconcile_planner)
B1. `turn_all_edits_into_love_letters(server_page, edits, target_blocks) -> Vec<LoveLetter>`. Two passes: (1) edit / delete / move in the order given, (2) creates in target order with the position recomputed from the scratch's current state (previous surviving block, not the sketch's anchor). The two-pass shape is because creates must be placed relative to blocks that already sit at their final positions.
B2. Moves use `BossOfYrs::set_frac_position` (new). `move_block` would recompute the frac from neighbours, unsafe in a batch.
B3. Creates use `insert_new_block_with_preexisting_id`, so the target id becomes the doc id. Sketch stays `CreateNewBlockWithId`.
B4. Old LCS planner (`align_two_block_lists.rs`, `main_for_turning_a_reconcile_choice_into_love_letters.rs`) deleted.

STEP C - reconcile_page data flow
`MismatchedPage.client_blocks` / `server_blocks` are `Vec<Block>` with fracs populated via `get_entire_page` + one `get_frac_position_of_block` per id.

STEP D - folded into STEP B
The pre-plan had a separate cqrs-side letter builder. That got folded into the viewer's finish effect. cqrs's `finish_reconcile.rs` only verifies, encodes, sends, and wipes now.

STEP E - UI
Ported example_2's shape into `reconcile_page/`: horizontal numbered chooser, pick screen (three-column header, framed buttons at the top-center), inline viewer with preview strip on Done, nothing-selected panel with default picker and two-step Submit (review then send), one dark rounded box holding the active state. CSS lives in `web_interface/css/reconcile_page.scss`. Three files (`diff_overlay.rs`, `mismatched_card.rs`, `resolve_choice.rs`) deleted; three new (`inline_diff_viewer_host.rs`, `pick_which_side_wins_screen.rs`, `nothing_selected_panel.rs`).

STEP F - finish_reconcile.rs
Verify-nothing-changed → encode the pre-built letters → send → wipe. Same as before except it doesn't build letters anymore.

DECISIONS ALREADY MADE (do not re-ask)
The viewer carries ids via its props and output. It owns the LIS frac assignment and produces full love letters (when it has a boss). Default picker is in-memory only. Client-only pages are not in the chooser; they appear only in the review panel and go straight to Submit as CreateNewPage letters. The verify-nothing-changed guard runs at send time. Chooser is horizontal. Merged = the viewer's output for the effective kept side. Blank blocks: blanking a block deletes it (except the title); an untouched blank block is left alone; client-only blank blocks are not created. Block metadata on existing blocks is not edited (known gap).