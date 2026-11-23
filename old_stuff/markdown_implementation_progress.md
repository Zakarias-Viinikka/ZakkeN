# markdown_implementation_progress.md

Written by AI for AI with zero memory. Companion to `/home/zakke/ProgStuff/ZakkeN/markdown_implementation_plan.md` (the task and its design). Keep this file current IN PLACE. If code and this file disagree, code wins. Last updated 2026-11-23.

`19nov_progress.md` is about the ANDROID task, ignore it.

## Where we are

Plan steps 1 to 4 are written. Step 5 (risks 1 to 4) is partly done. Steps 6 and 7 (manual checklist, update SPIKE_REWRITE.md) are not done.

All paths below are relative to `/home/zakke/ProgStuff/ZakkeN/cqrs_state_synchronization/web_interface/` unless absolute.

## Layout that now exists

- `markdown_js/` : package.json plus `src/{editor,decorations,link_parser,theme,wikilink_complete}.js`. Build: `cd markdown_js && npm install && npm run build`. Output goes to `markdown_web_output/cm_bridge.js` (built bundle, committed, like `zdb_web_output`). No Trunk hook. If `editor.js` changes, rebuild with `npm run build` before `c c`.
- `index.html` : `copy-dir markdown_web_output` plus a classic `<script src="/markdown_web_output/cm_bridge.js">`.
- `src/markdown_conversion/` : `mod.rs`, `cm.rs` (the ONLY file that talks to JS), `edit.rs` (pure, tests), `keys.rs` (pure, tests), `markdown_conversion.rs` (`<MarkdownConversion />`). `lib.rs` has `pub mod markdown_conversion;`. Cargo.toml got web-sys feature `Document`; serde, serde_json, js-sys were already deps.
- `src/complex_state/caret/` : `caret_request.rs` (ONE request signal: `CaretPlace` = Start | End | At(utf16), `CaretRequest` has a serial; `plexset_caret_request`, `plexget_caret_request(_untracked)`, `plexset_clear_caret_request_if_serial`), `caret_handlers_registry.rs` (reactive map block_id -> handler, tokens so unregister is "only if still mine"), `focus.rs` (`at_end`, `at_start`, `at_pos`, same signatures as before, now only write the request). `complex_state/mod.rs` has `pub use caret::focus;` so callers still write `focus::at_end(..)`.
- `src/leptos_components/main_content/caret_owner.rs` : `<CaretOwner />`, mounted once in `main_content.rs`. One Effect watches request plus registry. Runs the block's handler on the NEXT animation frame, retries `At(pos)` beyond the text up to 10 frames then the handler clamps, drops an unhandled request after 2 s. Clears the request only if it is still the same serial.
- `src/bbox_drag_reorder_container/my_block_content.rs` : now a thin adapter around `<MarkdownConversion />` (placeholder "Untitled" for title, "" otherwise, `class="ps-content"`). `drag_reorder_container.rs` was not touched.
- Deleted: the three statics FOCUS_BLOCK_ID / FOCUS_BLOCK_AT_START / FOCUS_BLOCK_AT_POS and their re-exports, plus the old textarea helpers (handle_arrow/enter/backspace, autosize, value sync, focus effects, map_caret_across_edit) in `my_block_content.rs`.
- Converter API: no focus props. Once the editor is ready it registers a handler under its block_id. The handler (1) set_texts the editor from `content` if they differ, (2) `set_selection` (scrolls into view), (3) `focus_keep_caret`. New JS bridge fn `focusKeepCaret` plus `cm::focus_keep_caret`. Uses the app's `BlockInputEvent` directly.
- `keys::decide` now takes `is_title`. Intent `EnterOnEmpty` was renamed `EnterWithoutSplit`.

## Verified (Zakke pasted output or said so)

- Milestone 1: `typeof window.cmBridge` printed "object" in the browser.
- `c c` green after step 2 (CaretOwner plus registry, with old statics still written), after moving everything into `complex_state/caret/`, and after step 3 (converter reshape).

## Not yet seen / not verified

- `c c` for step 4 (adapter swap, statics deleted) and for the key-parity chain (keys.rs rewrite, min-width css). Zakke has not reported either. Ask him.
- NOTHING has been run in the browser since the converter replaced the textarea. No typing, split, merge, arrows, drag, wrapping, emoji tests done.
- Unit tests (`c t` in web_interface, or native cargo test) for `keys.rs` and `edit.rs` not run. `keys.rs` tests were rewritten for the new signature.
- Possible compile problems I could not check: `BlockRenderInput` fields were confirmed from `types.rs` (block_id: String, content: ArcRwSignal<String>, ghost: Signal<bool>, is_title: bool).

## Risk findings (plan step 5)

1. Heights: no converter change needed. `drag_reorder_black_box.rs` already puts a ResizeObserver on every `.ps-box`, which calls `apply_heights`. Boxes have no fixed height, so CodeMirror growth, wrapping and server text all re-measure. Added `min-width: 0` to `.ps-content` in `drag_reorder_black_box.css` so long unbroken text cannot widen the flex box. Not tested in the browser.
2. Rebuilds: blocks are keyed by `internal_id` in a `<For>`. `full_rebuild` in `sync.rs` reuses the internal_id for an existing block_id (also across pages), so editors survive moves and reorders. Editors are re-created only when a block is truly added or removed or the page changes. The unregister-if-still-mine rule covers a same-id re-mount. Reading only, not tested.
3. UTF-16: PROBLEM, lives outside web_interface, NOT edited. See next section.
4. Key parity (old `handle_*` vs `keys.rs`): two differences found and fixed in `keys.rs`: Enter on a title must not split (old: set was_enter, newline allowed), Enter on a whitespace-only non-title block must not split (old: same). Backspace at 0, Delete on empty, arrows only at edges match. Unverified: Shift+Enter (old code split even with Shift; CodeMirror may treat it differently). Also the old code called `el.blur()` before `on_split`; the new code does not. Old merge only ran for non-titles, same now.

## Risk 3 detail: three different units for text positions

- Editor (CodeMirror), `keys.rs`, `edit.rs`, `focus::at_pos`, `cursor_pos`: UTF-16 code units. Consistent among themselves.
- `text_diff/src/diff_logic.rs` (`get_diff`): counts `chars()`, positions are u32 in chars. `string_from_diff_result` skips/takes chars. Also `diff_batching/combine_list_of_getdiff_results.rs` lines 70 and 73 use `chars().count()`.
- `yrs/src/yrs_wrapper/text_edit.rs`: passes `position` straight into `XmlTextRef::insert` / `remove_range` (units decided by the doc's OffsetKind) and computes delete lengths with `chars().count()`. `apply_edit_to_string` (meta) is chars throughout (self-consistent).
- Every doc is made with `Doc::new()` (no options): `yrs/src/yrs_wrapper/boss.rs:69`, `snapshots.rs:20`, `freeze_doc_main/freeze_internals.rs:25`, `yrs_active_pages.rs:23,29`, `yrs_backlinks.rs:24,30`. I believe yrs defaults to `OffsetKind::Bytes` (not confirmed from the yrs source, confirm). Only Bytes and Utf16 exist, as far as I know.
- Result: ASCII is fine everywhere. å/ä/ö (2 bytes, 1 char) and CJK (3 bytes, 1 char) make chars differ from bytes. Emoji (4 bytes, 1 char, 2 UTF-16) differs in all three. This is the known audit item "an edit after Swedish letters panics yrs". It already existed with the textarea; the converter does not cause or worsen it, because `on_block_input` still diffs old/new strings in Rust, so CodeMirror offsets never reach yrs.
- Proposed fix (needs Zakke's go-ahead, touches yrs, text_diff and probably Android/server): make every doc `Doc::with_options(Options { offset_kind: OffsetKind::Utf16, ..Default::default() })` and make `get_diff` plus `text_edit.rs` count UTF-16 units (`encode_utf16().count()`, and UTF-16-aware slicing in `string_from_diff_result`, without splitting surrogate pairs). Open questions before doing it: does changing the offset kind affect already-stored/encoded docs (check), do server and Android build the same docs (they share these crates, so they must all change together), and `text_diff` has uniffi exports for Android.
- `spike_for_markdown/old/FINDINGS.md` is NOT the findings: it contains only a "failed copy" error message. The earlier FINDINGS text (bytes vs UTF-16 vs `chars().count()`) is only summarized in SPIKE_REWRITE.md.
- Still to test once running: emoji and CJK typing, split, merge, caret landing after merge (`at_pos` with UTF-16).

## Known gaps and decisions

- Wikilink titles (`titles` prop) and `on_link_click` are unwired on purpose (first pass, follow-up).
- Styling: the spike theme (`markdown_js/src/theme.js`) is a white box with a grey border. The app is dark (`.ps-content` css: transparent, light text, big title font). Expect the editor to look wrong until a styling pass. Title font size in the old css applies to `.ps-content`, CodeMirror sets its own font size, so titles probably look small. Not done.
- `on_block_input` still computes the diff from old/new text (plan says leave it).
- `BlockInputEvent` has no derives; `MarkdownConversion` uses it as is.
- Old behaviour kept: Enter on empty block sets `was_enter` for the next input event.

## Next steps

1. Ask Zakke for the `c c` results of step 4 and the key-parity chain. Fix errors first.
2. Zakke runs `cd markdown_js && npm run build` only if editor.js changes; otherwise `c r`, opens a page, types. Ask what he sees before writing more code.
3. Styling pass for the dark theme and title size (tell Zakke before changing anything visual).
4. Report risk 3 to Zakke and decide the fix (separate task, other crates).
5. Plan step 6: give Zakke the short manual checklist (type; Enter splits and caret lands at start of the new block; Backspace at start merges and caret lands at the join; arrows move between blocks; deleting an empty block focuses the one above; long wrapped lines keep layout right; emoji/CJK; a server update arriving while typing; dragging a block).
6. Plan step 7: update `spike_for_markdown/SPIKE_REWRITE.md` in place with the final shape and what is verified vs not run.

