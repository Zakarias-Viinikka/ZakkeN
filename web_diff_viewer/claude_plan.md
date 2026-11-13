# Claude's plan for the reconcile page port

**SUPERSEDED.** This file was the plan for Step B. During
implementation the design changed twice more:

- The viewer grew id-carrying props (`server_page` / `client_page` /
  `kept_side`) and emits `ViewerOutput { kept_blocks, sketches }`
  directly. There is no caller-side reattach-by-text step.
- Frac assignment (LIS + `smart_frac_index`) moved inside
  `block_diff_viewer_leptos`, so only the minimal set of `MoveBlock`
  sketches is emitted.
- `drift_guard_and_ops/client::reconcile_planner` was rewritten
  around `turn_all_edits_into_love_letters`. The old LCS planner was
  deleted. `BossOfYrs::set_frac_position` was added so moves write
  the frac verbatim instead of recomputing from neighbours.

`web_diff_viewer/STORED_AI_CONTEXT.md` is now the source of truth for
the viewer contract and the sketch-to-letter flow. This file stays as
a record of the plan; treat it as historical.

Revised 2026-11-13. The original plan (saved in git history) had the
caller match viewer text back to block ids after each session. That
was wrong. The correct shape, decided with the user since:

- The viewer carries ids through. Its props are pages, not strings.
- The viewer owns frac recomputation via `smart_frac_index`.
- The viewer produces love-letter sketches directly.
- The caller turns sketches into full `LoveLetter`s (the scratch-doc
  snapshot step).

So `block_diff_viewer_leptos` grows a public surface. It gains deps on
`generate_love_letters`, `love_letter`, `smart_frac_index`, `yrs`. The
"do not modify it" rule from the original plan is dropped.

---

The original writeup follows, updated in place where it contradicts
the new shape. Where something changed, the new text says so.

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

0a. Public structs. Add two types the caller hands in and gets back. Names are yours; use cqrs-style long names. Suggested:
 - `PageBlocksForTheViewer { page_id: String, is_main_menu_page: bool, blocks: Vec<Block> }` where `Block` is re-exported from `generate_love_letters`.
 - `ViewerOutput { kept_blocks: Vec<Block>, sketches: Vec<LoveLetterSketch> }` where `LoveLetterSketch` is re-exported from `love_letter` (via `generate_love_letters`).
Caller-facing fields, documented in lib.rs.

0b. Props. `DiffViewer` takes:
 - `server_page: Signal<Option<PageBlocksForTheViewer>>` (None means client-only)
 - `client_page: Signal<Option<PageBlocksForTheViewer>>` (None means server-only, which the reconcile flow doesn't use today but the API supports)
 - `output: WriteSignal<ViewerOutput>`
 - `handle: FinishHandle`
 - `on_finished: Option<Callback<()>>`
 - Keep the remount-on-input-change behaviour for the non-client-only path. For client-only, don't render the diff area; just produce a ViewerOutput.

0c. Internal sketch generation. On finish:
 - If client-only (server None, client Some): kept_blocks = client blocks; sketches = vec![LoveLetterSketch::CreateNewPage { page_id, is_main_menu_page }]. No LIS, no fracs.
 - If both present: kept_blocks = current right-side blocks in visual order (each entry retains the id from the row it came from); sketches = the result of running the LIS frac assignment and generate_sketches described below.
 - Write to output, fire on_finished.

0d. LIS frac assignment (moved into this crate from the planner). Port the algorithm described in the original Step B below. It runs against the two pages the viewer already has. It calls smart_frac_index. Result: the alive-and-with-correct-frac `Vec<Block>` list ready for generate_sketches. This is a pure helper in the crate.

0e. Call generate_sketches(page_id, server_blocks, alive_with_fracs). Map GenErr to an "error line" string in ViewerOutput — for simplicity, treat a GenErr as a programming bug and panic inside the crate; the caller's self-check at the letter stage is the real safety net, and a GenErr means the viewer's inputs are malformed (duplicate ids), which can't happen given the props are built from the server's own page.

0f. Update example_app and example_2 to the new prop shape. example_2's `placeholder_data.rs` becomes `Vec<PageBlocksForTheViewer>` pairs. example_2's Done flow reads `output.kept_blocks` and `output.sketches`. Delete the old text-based flow (reattach, fuzzy matching) — it's gone. The user explicitly does not want example_2's UI shape changed; only the data types flowing through it change.

0g. Native unit tests: LIS frac assignment (move one block to top -> one move sketch, no other moves; swap two -> one move; identical -> zero sketches; blank server blocks untouched -> zero sketches). Port the test cases from the deleted planner step.

STEP A - generate_love_letters (web_diff_viewer/generate_love_letters)
A1. Fine-grained edits WITHOUT text_diff::get_diff (get_diff is unsound for different-length replacements; do not use it and do not add a text_diff dep). Write a pure helper that computes the common prefix length in chars, then the common suffix length in chars capped so prefix+suffix <= min(old_len,new_len). old_mid and new_mid are what's left. old_mid empty -> TextEdit::Insert{text:new_mid, position:prefix}. new_mid empty -> TextEdit::Delete{text:old_mid, position:prefix}. both non-empty -> TextEdit::Replace{old_text:old_mid,new_text:new_mid,position:prefix}. Identical -> no edit. Positions are in chars, same convention get_diff used. Use it in the EditBlock branch in place of the hardcoded full Replace.
A2. Blank rule refinement: today "server id present in user with empty text -> RemoveBlock". Change to remove only if the server's text is NOT empty. Blank on both sides is unchanged (no letter). Leave the other rules alone (id missing from user -> RemoveBlock; new non-empty id -> CreateNewBlockWithId; skipped when blank; frac differs -> MoveBlock).
A3. Add native unit tests: cat->dogs, abcXdef->abcYYdef, aa->aaa, abcabc->abc, "a\n\n"->"a\n\n\n", emoji text, blank-both-sides is a no-op. Run the workspace tests (`c t`) and `cargo check -p example_2 --target wasm32-unknown-unknown`. Update STORED_AI_CONTEXT.md (generate_love_letters rules + the viewer now owns frac assignment and sketch generation; the old id-reattach problem is gone because the viewer carries ids).

STEP B - caller-side letter builder (drift_guard_and_ops/client/src/reconcile_planner, must stay pure: no leptos/network/DB)
Add dep to drift_guard_client: love_letter (already there) and yrs (already there). The planner no longer needs generate_love_letters, smart_frac_index, or text_diff. Remove the ones added earlier if unused. Re-run cargo tree -d after, confirm love_letter and my_yrs_lib still appear exactly once.

B1. main_for_turning_page_sketches_into_love_letters: `create_love_letters_out_of_page_sketches(server_page: Arc<BossOfYrs>, sketches: &[LoveLetterSketch]) -> Result<Vec<LoveLetter>, String>`.
 - Open the scratch copy exactly like today (unique actor).
 - Apply the sketches in the order given:
   - EditBlock -> edit_and_capture with the sketch's own text_edit.
   - RemoveBlock -> delete_and_capture.
   - MoveBlock -> NEW move_and_capture. `BossOfYrs::move_block` takes (block_id, above_id: Option<String>, below_id: Option<String>) and computes frac internally. The sketch carries `new_frac_position`; the letter needs it. So: apply the move to scratch, then read `scratch.get_frac_position_of_block(block_id)` and put THAT into the sketch's `new_frac_position` field before building the letter. Above/below ids for the move come from the sketch itself — the viewer produced them when it decided the move.
   - CreateNewBlock -> insert_and_capture (as today). Position comes from the sketch's `position_to_insert`.
   - CreateNewPage -> build the letter directly: `build_a_love_letter(CreateNewPage{page_id, is_main_menu_page}, server_page.snapshot()?)`. If the server page for this call is None (client-only), pass an empty snapshot — CreateNewPage's payload is the client's own data, which the caller provided via the sketch (is_main_menu_page) plus the client page's blob (passed in alongside; adjust the API to also take `client_page_blob: Option<Vec<u8>>` for this path, and use it for CreateNewPage letters).
   - DisablePage or CreateNewBlockWithId -> Err.
 - Self-check at the end: for pages that had a server side, scratch texts in order must equal the target list (kept_blocks passed alongside). For client-only pages, the check is trivial (nothing to compare). Otherwise Err including both lists. Adjust signature to also take `target_texts: &[String]`.
B2. `move_and_capture` lives in `turn_one_change_into_a_love_letter.rs` alongside the existing three. Add it. Reuse the bookmark -> apply -> generate_diff_snapshot -> build_a_love_letter pattern.
B3. Delete main_for_turning_a_reconcile_choice_into_love_letters.rs and align_two_block_lists.rs and the text_edit_for helper only after grep shows nothing else uses them. Update mod.rs.
B4. Native test: `create_love_letters_out_of_page_sketches` round-trip for one page (edits + a move + a create) against a BossOfYrs built from a small page; assert the self-check passes and the resulting love letters apply cleanly to a fresh copy.

STEP C - reconcile_page data flow (web_interface/src/leptos_components/reconcile_page)
phase.rs: MismatchedPage.client_blocks / server_blocks become `PageBlocksForTheViewer` (or `Vec<Block>` plus separate page_id/is_main_menu_page fields — pick one, keep it consistent with STEP 0). load_and_classify.rs: read_local_pages and the server-blob decoding produce the new shape via one small conversion helper at the bottom of the file. Use `BossOfYrs::get_entire_page` + `BossOfYrs::get_frac_position_of_block` per id to build `Vec<Block>`. Everything else (fingerprints, classification, client_only) stays. Client-only pages get `server_page = None`.

STEP D - building letters (web_interface, reconcile_page folder)
D1. main_for_building_letters_for_one_reconciled_page: input is the Arc<MismatchedPage> plus the `ViewerOutput` for that page (kept_blocks + sketches). Decodes the server blob via doc_from_snapshot, calls `create_love_letters_out_of_page_sketches`, returns summaries (one line per sketch, port `summary` from example_2) plus encoded letters.
D2. main_for_building_the_whole_reconcile_submission: loops over all mismatched pages, calls D1 for each (skipping pages with no viewer output yet — those were never picked), appends the client-only letters, returns `ReconcileSubmission { per-page summaries, client_only page ids, letters: Vec<Vec<u8>> }`. Any page error -> Err naming the page.
The Done button's preview uses D1 for just that page (using the ViewerOutput the viewer just produced). Submit uses D2 (using each pair's last stored ViewerOutput).

STEP E - UI (replace ready_view internals; delete mismatched_card.rs, diff_overlay.rs, resolve_choice.rs)
Keep in ready_view.rs: rp-meta line and the "N mismatched pages, plus M new pages" summary line. Under them is ONE dark rounded box (bg #1c2433, border rgba(228,233,242,.09), radius 10px) holding one of three states. Port example_2's components with cqrs-style long names, e.g. numbered_page_chooser_row.rs, pick_which_side_wins_screen.rs, inline_diff_viewer_host.rs, nothing_selected_panel.rs, pair_state.rs (PickedSide {Mine, Server} + PairState). Copy example_2's CSS consts nearly verbatim but rename every class to an rp- prefix (.chooser -> .rp-chooser etc.) to avoid collisions. The layout classes (container, viewer-region, nothing-selected, default-picker, default-btn, no-diff-open, submit-btn, sketch-panel...) are NOT in the .rs files: read them from example_2's index.html/scss and port them. The reconcile page's .rp-page-inner is capped at 960px, which is too narrow for a side-by-side diff: widen it (about 1400px) and give the viewer region a viewport-relative height with the host using flex:1;min-height:0 so the viewer body scrolls internally. The block_diff_viewer scss must already be linked in the web_interface index.html (the old overlay used it); verify.

The UI must look and behave like example_2:
 - Horizontal numbered chooser row; green when the pair has an effective pick, accent border when selected; add a title tooltip with the page_id.
 - Pick screen: 3-column header grid (numbered close-box + page_id left, two framed buttons "Use mine" / "Use server's" top-center using the .btn-6 "Framed" style from ZakkeN/cool_button.md, dull by default, glow on hover), side-by-side plain preview below.
 - Viewer host: single-row toolbar (number-box that saves+closes, page_id, counts, hint, legend hidden below 900px and shown above with the viewer's own .dv-legend hidden, Reset only when no default set, Done primary), DiffViewer, preview strip "Would generate:" under the diff that appears on Done and stays open.
 - Direction: Mine picked -> left = server (discarded), right = client (kept). Server picked -> left = client, right = server.
 - "Default for everything" (Ask each time / Always mine / Always server's) lives in the nothing-selected box, in memory only (no persistence). Pair's explicit pick wins over the default.

State design (important, do not copy example_2 here):
 - The set of mismatched pages never changes during a mount. Build a plain Vec of PairState once, each holding Arc<MismatchedPage> plus its OWN signals: picked: RwSignal<Option<PickedSide>> and last_output: RwSignal<Option<ViewerOutput>> (the last ViewerOutput the viewer produced for this pair). Do not use one RwSignal<Vec<PagePair>>. Use Arc, not Rc: leptos closures must be Send+Sync.
 - The closure that renders the three-state region must track ONLY `selected` and the selected pair's effective side (derive it through a Memo). Everything else (last_output, previews) must be read with get_untracked inside it. Anything tracked there remounts DiffViewer and wipes in-progress edits/drags. Example_2 does remount on Done; the real page must not.
 - Viewer props are built from the pair: server_page = Some(server blocks as PageBlocksForTheViewer) — None if this is a client-only page; client_page = Some(client blocks). When the user has picked, the viewer's left/right are set by passing the two pages in that order (Mine -> left=server, right=client; Server -> left=client, right=server). Wrap that ordering in the viewer_host component, not in the pair.
 - On Done and on X: read the ViewerOutput from output.get_untracked(), store it as last_output for this pair. Done additionally computes the preview via D1 (using the same output). X closes, selection = None. Reset clears picked and last_output and returns to the pick screen.
 - Submit is enabled when every mismatched page has an effective pick (keep the "n of t resolved" note); with zero mismatched pages show the old "Nothing to resolve..." text and Submit is enabled. Submit runs D2 and shows a review panel in the box: per page the summaries ("no letters" for empty), a separate "New pages that will be added" section listing client-only ids, a Back button (clears the review), and "Send and finish" (hidden if D2 errored; show the error instead). Selecting a pair clears the review. Disable chooser and Send while finishing.

STEP F - finish_reconcile.rs
Shrinks to: verify_nothing_changed_since_mount (unchanged) -> send_reconcile_letters(target_drift_version, target_reconcile_version, submission.letters) if non-empty, with the same DriftVersionMismatch error mapping -> wipe_local_and_pull_from_server. The caller (Send button) keeps today's behaviour: Ok -> reload_the_page(), Err -> show_popup(...) and finishing=false.

STEP G - verify and report
cargo check the web_interface for wasm32, cargo test drift_guard_client and generate_love_letters natively, `c t` in web_diff_viewer, `c c` for example_2. Update STORED_AI_CONTEXT.md (both web_diff_viewer's and ZakkeN's External: web_diff_viewer section). In your final report list every deviation from this plan and anything you could not verify (especially: whether `BossOfYrs::move_block` produces the frac the sketch expected when read back, whether the client-only CreateNewPage letter needs the client blob alongside the sketch, whether DiffViewer's output length can differ from its input length, and whether the workspace now has duplicate love_letter/my_yrs_lib copies after adding deps to block_diff_viewer_leptos).

DECISIONS ALREADY MADE (do not re-ask)
The viewer carries ids via its props and output. It owns the LIS frac assignment and produces love-letter sketches. The caller turns sketches into full love letters. The crate deps do not scare anyone. Default picker is in-memory only. Client-only pages are not in the chooser (they appear only in the review panel); the viewer handles them via server_page = None. The verify-nothing-changed guard runs at send time. Chooser is horizontal. Merged = the viewer's output for the effective kept side; a pick with no edits sends that side unchanged. Blank blocks: blanking a block deletes it, an untouched blank block is left alone, client-only blank blocks are not created. Block metadata on existing blocks is not edited (known gap).