# CodeMirror 6 + yrs Markdown Spike

Proves whether CodeMirror 6 can render markdown + wikilinks visually while keeping raw text as the single source of truth in `yrs` (no visual-to-raw coordinate mapping).

## Data flow

    User edit in CM6
      -> CM6 `updateListener` (skips `fromRustAnnotation`)
      -> `update.changes.iterChanges`: [{from, to, insert}] in old-doc UTF-16 offsets
      -> JSON string across wasm-bindgen (`bridge_ffi.rs`)
      -> `convert_changes_to_edits_desc` slices pre-edit authority string, sorts DESC by position
      -> `AuthorityBlockDoc::apply_edits` applies `Insert`/`Delete`/`Replace` to `yrs::XmlTextRef` (`OffsetKind::Utf16`)
      -> Compare `authority.read_text()` vs `cmBridge.getText(id)` (red flag on mismatch; manual Resync only)

    Remote / external edit in `yrs`
      -> `AuthorityBlockDoc::simulate_remote_edit` (peer snapshot -> edit -> state-vector diff -> merge)
      -> `cmBridge.applyExternalText(id, newText)`
      -> Minimal UTF-16 prefix/suffix diff (never splits surrogate pairs)
      -> Dispatched with `fromRustAnnotation` + `Transaction.remote` + `addToHistory: false` (keeps caret, no echo, no undo)

## Read-this-first file order

1. `FINDINGS.md` — `yrs` offset-unit discovery, quirks, and what this replaces in the real app.
2. `src/edit_translation.rs` — `CmChange` -> descending `TextEdit` conversion in UTF-16 coordinates.
3. `src/authority_doc.rs` — `yrs` `OffsetKind::Utf16` `Doc -> Array("blocks") -> Map{id, text: XmlTextRef}` + remote diff sim.
4. `src/link_parser.rs` + `js/src/link_parser.js` + `test_vectors/links.json` — shared wikilink grammar & vectors.
5. `js/src/decorations.js` — single `ViewPlugin` (allowed Lezer nodes + regex wikilinks + `atomicRanges`).
6. `js/src/bridge.js` — `window.cmBridge`, update listener, external diff dispatch, intent keymap, autocomplete, `ResizeObserver`.
7. `src/bridge_ffi.rs` — wasm-bindgen FFI + `EditorBridgeHandle` closure lifetime guard.
8. `src/fuzz.rs` — deterministic multi-change fuzz generator.
9. `src/harness_app.rs` — Leptos CSR 3-block debug harness.
10. `api_example/src/markdown_conversion.rs` — standalone blackbox `<MarkdownConversion />` component mirroring `MyBlockContent`.
11. `api_example/src/app_main.rs` — plug-and-play demo testing two-way sync, focus-hopping, title block enter-ignore, and ghost toggle.
