# SPIKE_REWRITE.md  (read this first for the markdown spike task)

Written by AI for AI with zero memory. Keep it current IN PLACE. If code and this file disagree, code wins. `/home/zakke/ProgStuff/ZakkeN/19nov_progress.md` is about the ANDROID task, not this one.

## Task
Plug a JavaScript markdown editor (CodeMirror 6) into a Leptos web app cleanly. The first attempt mixed JS and Rust so badly it was reverted from the app (commit 7751f0e "removed the markdown conversion stuff"). This is a from-scratch rewrite using the old spike only as reference. Goal: JS is a THIN wrapper; event and caret logic live in Rust/Leptos.

Scope: ONLY `/home/zakke/ProgStuff/ZakkeN/spike_for_markdown`. Do not touch the real app (`cqrs_state_synchronization`), Rust crates, `zakken_server`, `z_db`.

## STATUS (2026-11-22): spike is FEATURE COMPLETE (written). Zakke decided to stop testing minute details and will test when he integrates.
- Steps 1, 2 verified in browser by Zakke. Step 3 partly verified (typed `Changes` events and non-edge keys OK).
- Steps 4, 5, 6 (everything below marked WRITTEN) were written in one pass. Build was requested with `npm --prefix js run build && ./local_cargo.sh check`; result NOT yet seen by the AI. Not run in a browser. Unit tests (`./local_cargo.sh test`, in `src/keys.rs` and `src/edit.rs`) not run.
- If the build output shows errors: fix them first, that is the only open work.

## Layout
- `old/` = full copy of the previous spike (REFERENCE ONLY, never edit). Things NOT ported that live there:
  - `old/src/link_parser.rs` + `old/test_vectors/links.json` + `old/js/test_links.mjs`: pure Rust `parse_links` (for backlinks) with shared test vectors.
  - `old/src/authority_doc.rs`, `edit_translation.rs`, `fuzz.rs`, `harness_app.rs`: yrs authority + TextEdit translation + fuzz harness.
  - `old/FINDINGS.md`: yrs `Doc::new()` defaults to OffsetKind::Bytes while CodeMirror is UTF-16 and the real app's `text_edit.rs` uses `chars().count()`. Emoji/CJK can corrupt in the real app unless yrs uses `OffsetKind::Utf16` and lengths are counted in UTF-16 units. Still true, still relevant at integration.
  - ResizeObserver height reporting (old component had it as a no-op).
- `js/src/editor.js`: thin CM6 wrapper -> esbuild -> `assets/cm_bridge.js` (IIFE, `window.cmBridge`). `npm --prefix js run build`; Trunk.toml runs it as pre_build hook and ignores `assets` in watch.
- `js/src/decorations.js`, `js/src/link_parser.js`: copied unchanged from old (hide `**`/`#`/backticks unless selection touches, `[[Title|id]]` wikilink widgets, titles map). Rendering only.
- `js/src/theme.js`: textarea-like styling. `js/src/wikilink_complete.js`: `[[` autocomplete from the titles map.
- `src/cm.rs`: the ONLY Rust file that knows the bridge. `extern "C"` fns + pub wrappers (`set_text`, `get_text`, `get_selection`, `set_selection`, `focus`, `set_editable`, `set_placeholder`, `set_titles`), private `CmEvent` wire enum, `CmEditor` component (`on_edit`, `on_key -> bool`, `on_link_click`, `on_ready`, `initial_titles`), `PageTitle`.
- `src/edit.rs`: pure `apply_changes(old, &[Change])` (UTF-16, old-doc coordinates) + tests.
- `src/keys.rs`: pure `decide(key, anchor, head, doc_len, ranges, text) -> Option<Decision{intent, consume}>` + tests. Intents: NavUp, NavDown, Split, MergeUp, EnterOnEmpty.
- `src/markdown_conversion.rs`: `<MarkdownConversion />`, props mirror `MyBlockContent`.
- `src/main.rs`: demo with title + 2 body blocks, wikilinks, buttons (fake server update, rename page, disable page, ghost toggle) and an event log.

## Interop design
- Rust -> JS: plain `#[wasm_bindgen] extern "C"` fns on `cmBridge.*`, declared only in `cm.rs` (starter rj1 pattern).
- JS -> Rust: ONE window CustomEvent `cm`, detail = JSON `{id, kind, ...}`; kinds `changes {changes, head}`, `key {key, anchor, head, docLen, ranges}`, `link_click {pageId}`. Parsed in `cm.rs` (starter rj2/rj3 pattern). No Closure objects given to JS.
- The event is cancelable: for `key`, Rust calls `prevent_default()` to consume the key; JS `emit()` returns false and the keymap returns true. JS never decides key meaning.
- Each editor filters events by its own id.
- Offsets are UTF-16 units everywhere (CodeMirror native); `cursor_pos` in `MarkdownInputEvent` is UTF-16.
- JS fires events synchronously inside CM dispatch. Rust handlers do not call back into dispatching JS (`get_text` reads are fine).
- Rust -> editor text uses ONE minimal surrogate-safe change tagged "from Rust" + remote + no history, so no echo event, caret stays, undo does not revert it.

## How `<MarkdownConversion />` works
- `content: ArcRwSignal<String>` is truth. User edit -> `apply_changes(old, changes)` in Rust, cross-checked against `cm::get_text` (mismatch logs an error and the editor text wins) -> `content.set` -> `on_input(MarkdownInputEvent{block_id, old_text, new_text, was_enter, cursor_pos})`.
- Outside change to `content` -> Effect -> `cm::set_text` if different (no echo).
- Keys -> `keys::decide`: NavUp/NavDown -> `on_navigate((id, down))`; Split -> `on_split((id, before, after))` (not for `is_title`); MergeUp -> `on_delete_request(id)` if content empty else `on_merge_with_above((id, text))` (not for `is_title`); EnterOnEmpty -> not consumed, sets `was_enter` for the next `on_input`.
- `focus_at_end/start`: `ArcRwSignal<Option<String>>`, set to Some(block_id); component focuses and resets to None.
- `ghost` -> read-only, `placeholder`, `titles: Signal<HashMap<page_id, PageTitle{title, disabled}>>`, `on_link_click(page_id)`, `class`.
- Title blocks: Enter/Backspace-at-0 are consumed but do nothing (same as the old spike).

## Integrating into the real app (for whoever does it)
1. Copy `js/` (without node_modules), `src/cm.rs`, `src/edit.rs`, `src/keys.rs`, `src/markdown_conversion.rs` into the web interface; add `mod`s. Deps: `serde` (derive), `serde_json`, `js-sys`, `wasm-bindgen`, web-sys features `CustomEvent`, `Element`, `Window`, `Document`, `HtmlElement`.
2. Either keep the esbuild pre_build hook (Trunk.toml, `[watch] ignore = ["assets"]`) or commit the built `assets/cm_bridge.js` and skip npm. Load it as a classic `<script>` before the wasm, and `copy-dir`/`copy-file` it in index.html.
3. Replace the `<textarea>` in `my_block_content.rs` with `<MarkdownConversion>` (props match). This removes `get_diff`, `resize_textarea_to_content`, `map_caret_across_edit` usage for this path.
4. Check the yrs offset-kind issue from `old/FINDINGS.md` before trusting emoji/CJK text.
5. `cargo test` for `keys.rs`/`edit.rs` should run natively; wasm check: `cargo check --target wasm32-unknown-unknown`.

## Decisions log
- 2026-11-22: old spike copied to `old/`, root files replaced. Kept Cargo.lock, Trunk.toml, rust-toolchain.toml (nightly), js/package.json, node_modules.
- 2026-11-22: JSON strings stay as the JS<->Rust wire format but are confined to `cm.rs`/`editor.js`; app code sees typed values.
- 2026-11-22: skipped on purpose (Zakke: "just finish the spike"): yrs authority, fuzz, Rust `parse_links`, height observer.

## Working with Zakke
He only runs commands I give him, does not read code or debug. Short replies. One `cat` line with absolute paths to get files. Run cargo via `./local_cargo.sh` in the spike folder (`check` = wasm check, `run` = trunk serve, `test` = native tests). Never claim verified without pasted output.
