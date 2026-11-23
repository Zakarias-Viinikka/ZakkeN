TASK: plug the markdown editor from the spike into the real web app, and add a CaretOwner that owns all caret placement.

Context
- Spike (reference material, NOT a spec): /home/zakke/ProgStuff/ZakkeN/spike_for_markdown. It contains a CodeMirror 6 editor behind a thin JS bridge (js/src/editor.js -> assets/cm_bridge.js, window.cmBridge). src/cm.rs is the only Rust file that talks to JS. src/markdown_conversion.rs is a blackbox <MarkdownConversion /> whose props mirror MyBlockContent. src/keys.rs and src/edit.rs are pure logic with tests. Read SPIKE_REWRITE.md first.
- Real app: /home/zakke/ProgStuff/ZakkeN/cqrs_state_synchronization/web_interface. Today each block is a <textarea> in src/bbox_drag_reorder_container/my_block_content.rs, rendered by DragReorderContainer inside the drag-reorder black box. App callbacks (src/leptos_components/main_content/callbacks/*) decide what split/merge/delete/navigate mean and call focus::at_end / at_start / at_pos (src/complex_state/focus.rs), which write three global signals that every block watches.
- /home/zakke/ProgStuff/ZakkeN/19nov_progress.md is about the ANDROID task. Ignore it. Its ban on touching web_interface does not apply here; Zakke requested this work.
- Zakke only runs commands you give him, does not read code, does not debug. Keep replies short. Ask for files with ONE cat line using absolute paths (use `t` for trees). Never claim something is verified without pasted output. Do not edit Rust crates, zakken_server, z_db, or the Android/other projects; report problems instead.
- Design freedom: the spike's shape was a first guess. Change its API wherever the real app wants a different shape. Rule that stays: JS stays thin; only cm.rs talks to JS; JS never decides what a key means.

Design

A. CaretOwner (the main new idea)
- Types: CaretPlace = Start | End | At(utf16 offset). A caret request = (block_id, CaretPlace).
- Global state in the same style as complex_state/ui_signals.rs: ONE request signal that replaces FOCUS_BLOCK_ID / FOCUS_BLOCK_AT_START / FOCUS_BLOCK_AT_POS, and ONE registry (block_id -> handler taking a CaretPlace; the registry must be reactive).
- focus.rs keeps at_end / at_start / at_pos with the same signatures, but they now just write the request. Callbacks in main_content/callbacks should need no changes. Delete the three old statics once grep shows no users.
- <CaretOwner /> is a component with no visible output, mounted once in MainContent. It holds one Effect that watches the request and the registry. When the request names a registered block, it runs that block's handler on the NEXT animation frame (not synchronously), then clears the request. Reason: callbacks update the row text and immediately ask for a caret; the new text takes a hop to reach the editor. For At(pos): if pos is beyond the editor's current length, retry each frame up to about 10 frames, then clamp.
- If the target is not registered yet (a freshly split block is still mounting), do nothing. When the block registers, the registry changes and the effect re-runs. Drop a request that is still unhandled after about 2 s, and any new request overwrites the old one.
- Key interpretation stays in keys.rs (pure, tested). The owner only places carets.

B. Converter reshape (src/markdown_conversion/ in web_interface)
- Remove the focus_at_end / focus_at_start props. When the editor is ready, the converter registers a handler under its block_id. The handler: (1) if the editor text differs from the content signal, set_text it first, (2) set the caret / focus, scrolled into view. It unregisters on cleanup, but ONLY if the registered entry is still its own (blocks may be re-mounted with the same id; a late cleanup must not remove the newer registration).
- cm.rs/editor.js need "focus without moving the caret" or "focus at position N". Check what exists; add the smallest thing if missing.
- Use the app's BlockInputEvent directly instead of the spike's duplicate struct (same fields).
- MyBlockContent becomes a thin adapter: take BlockRenderInput, render the converter with block_id, content, ghost, is_title, placeholder (the old code used "Untitled" for title, "" otherwise), and pass the callbacks through. Delete the old helpers: handle_arrow/enter/backspace, autosize, value sync, the three focus effects, map_caret_across_edit. DragReorderContainer should not need changes.
- First pass: leave wikilink titles and on_link_click unwired. That is a follow-up.

C. Bringing the bridge in
- Copy js/ (no node_modules), cm.rs, edit.rs, keys.rs, markdown_conversion.rs into web_interface under one new module folder; add dependencies (serde derive, serde_json, js-sys, web-sys features CustomEvent/Element/Window/Document/HtmlElement).
- Load cm_bridge.js as a classic script before the wasm, via index.html. Either keep the esbuild pre_build hook or commit the built bundle. If git is usable, look at commit 7751f0e (the earlier attempt that was reverted) to see how the build was wired. Prefer whichever needs the least setup for Zakke.
- Milestone 1: the app builds and loads with window.cmBridge defined, no blocks changed yet. Ask Zakke for the right build command; do not guess.

Risks to check, in this order (read the files first)
1. Heights. The drag-reorder black box measures block heights for its layout (read drag_reorder_black_box/heights.rs, sync.rs, rebuild.rs). The textarea resized itself; CodeMirror grows by itself, but the container must notice: on mount, when a line wraps while typing, and when text arrives from the server. If it does not already observe the block wrapper's size, add a size-change notification from the converter.
2. Rebuilds. Check whether block views are re-created on moves/reorders. If editors are destroyed and re-created, focus and caret are lost, so the owner must cope (see the unregister rule above).
3. UTF-16. CodeMirror offsets are UTF-16; yrs Doc::new() defaults to byte offsets, and the app's text_diff/get_diff and text_edit may count chars. Read spike old/FINDINGS.md. Test emoji and CJK. If the fix is in a crate, report it to Zakke rather than editing. Leave on_block_input computing the diff from old/new text for now.
4. Key parity. Compare keys.rs against the old handle_* code (Enter on a title, Enter on an empty block setting was_enter, Delete on an empty block acting like Backspace, merge only when not a title). Keep the real app's behaviour; list any difference to Zakke.

Order of work (give Zakke one build command per step and wait for output)
1. Milestone 1 above.
2. Add CaretOwner, registry and the single request signal; convert focus.rs; remove old statics. Build.
3. Reshape the converter (register handler, app's event type). Build.
4. Swap MyBlockContent to the thin adapter. Build.
5. Work through risks 1 to 4.
6. Give Zakke a short manual checklist: type; Enter splits and the caret lands at the start of the new block; Backspace at start merges and the caret lands at the join; arrows move between blocks; deleting an empty block focuses the one above; long wrapped lines keep layout right; emoji/CJK; a server update arriving while typing; dragging a block.
7. Update spike_for_markdown/SPIKE_REWRITE.md in place with the final shape and what is verified vs not run.