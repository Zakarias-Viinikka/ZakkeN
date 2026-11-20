# Findings

## 1. yrs Offset Units (CRITICAL for the real project)
- **Finding:** `yrs::Doc::new()` defaults to `OffsetKind::Bytes` (UTF-8 byte offsets and lengths), verified by `authority_doc::tests::discover_default_yrs_offset_unit_is_bytes` in `src/authority_doc.rs`.
- **Why this spike works cleanly:** Constructing the authority `Doc` with `Options { offset_kind: OffsetKind::Utf16, .. }` lets CodeMirror's UTF-16 `from`/`to` offsets pass straight through to `XmlTextRef::insert` and `XmlTextRef::remove_range`, provided `Delete`/`Replace` lengths also count UTF-16 code units (`s.encode_utf16().count()`).
- **Real-project bug warning:** `yrs/src/yrs_wrapper/boss.rs` uses `Doc::new()` (`OffsetKind::Bytes`), while `yrs/src/yrs_wrapper/text_edit.rs` passes `text.chars().count() as u32` to `remove_range` and `web_interface/.../my_block_content.rs` slices with UTF-16 indices. Any emoji or CJK text in the real app is exposed to offset/length unit mismatches across those three layers.

## 2. Surprises & Awkward Spots
- **Remote edits inside a `[[Title|id]]` token:** Because wikilinks are parsed by regex from raw text (not stored as structured CRDT nodes), a concurrent/remote edit that inserts a newline, `]`, or whitespace into the id (or newline/`]` into the title) breaks the token back into plain text. Non-breaking edits inside the title preserve the link with the modified cached title (until overridden by the live `titlesMap` if the id exists).
- **Sync-mismatch philosophy:** Never auto-healing on mismatch makes bugs immediately visible in the harness; `updateListener` only reads `cmBridge.getText` after Rust finishes mutating `yrs` and never dispatches back inside `onChanges`.
- **Closure lifetimes & re-entrancy:** `dispatchChanges` (fuzzing) synchronously triggers CM's `updateListener` -> Rust `onChanges` while inside the call stack. Rust must not hold any `RefCell` borrow across `dispatchChanges` or `applyExternalText`, and `EditorBridgeHandle::drop` must call `cmBridge.destroy` before dropping the `Closure`s.
- **Native vs wasm32 testing:** Splitting pure logic (`edit_translation`, `authority_doc`, `link_parser`, `fuzz`) away from DOM calls allows `cargo test` to verify 500-round fuzzing and `yrs` UTF-16 behavior natively without a headless browser.

- **Trunk `pre_build` watch loop:** Writing `assets/cm_bridge.js` in a `pre_build` hook causes `trunk serve` to re-trigger endlessly unless `[watch] ignore = ["assets"]` is set in `Trunk.toml`.

## 3. JS / npm / esbuild Dependency Note
- Rebuilding `assets/cm_bridge.js` from `js/src/` requires Node/npm + esbuild (currently wired as a Trunk `pre_build` hook per the spike spec).
- Once built, `assets/cm_bridge.js` is a standalone IIFE bundle with all CodeMirror 6 packages inlined. In the real app, you can commit `assets/cm_bridge.js` and omit the `pre_build` hook so contributors only need Node/npm if they edit the JS bridge itself.

## 4. What Integrating into the Real App Replaces
Replacing `<textarea>` in `cqrs_state_synchronization/web_interface/src/drag_reorder_container/my_block_content.rs` with this CodeMirror 6 bridge removes:
- **The `<textarea>` element and `resize_textarea_to_content`:** CM6 handles line wrapping naturally and `ResizeObserver` reports height changes for the reorder layout.
- **`get_diff` / `text_diff` on user input:** Instead of diffing `old_text` vs `new_text` after every `on:input`, CM6 reports exact `[{from, to, insert}]` deltas directly from `update.changes.iterChanges`.
- **`map_caret_across_edit`:** External/remote updates use `applyExternalText` with minimal UTF-16 prefix/suffix diffing, so CodeMirror preserves the user's selection/caret natively.
- **Re-hooked behaviours:** `Prec.high` keymap preserves `ArrowUp` at start, `ArrowDown` at end, `Enter` split (`before`/`after`), and `Backspace` at pos 0 (`merge_up`), plus `cmBridge.focus(id, "start"|"end")`.