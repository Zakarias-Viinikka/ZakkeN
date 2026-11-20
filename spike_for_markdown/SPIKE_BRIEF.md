# SPEC

## Question the spike answers
Can CodeMirror 6 be the "translator" between what the user sees and a raw-text authority held in Rust (yrs)?

## Core idea
Raw text is the only truth, e.g. `see [[Cats|p123]] and **bold**`. CodeMirror's document is exactly that raw string. Decorations only change how it is drawn (hide `**`, draw the link token as a clickable "Cats"). CodeMirror reports edits in raw positions, so there is NO visual<->raw position mapping code anywhere. Flow: user types -> CM -> bridge -> Rust converts to a TextEdit list -> applies to yrs -> compare. External/remote text: Rust -> bridge -> CM dispatch marked as from-Rust. Rust never receives or reasons about visual positions.

## Stack
- Trunk + Leptos CSR (copy the version from web_interface/Cargo.toml), wasm-bindgen, web-sys, `yrs` directly (no uniffi), serde, serde_json. Match the toolchain in web_interface/rust-toolchain.toml.
- JS: one esbuild bundle from `js/src/bridge.js` to `assets/cm_bridge.js` (IIFE exposing `window.cmBridge`). Packages: @codemirror/state, view, commands, language, lang-markdown, autocomplete. No basicSetup. Run the npm build as a trunk pre_build hook; load the script as a classic script before the wasm.
- Cross the boundary with JSON strings (serde_json), not serde_wasm_bindgen.

## Decoration rules (one ViewPlugin, rebuilt on docChanged, selectionSet, viewportChanged, titles-changed effect)
- Headings: line decoration with level class always on (layout stable); hide the `#` marks only when no selection range touches the heading.
- Bold, italic, inline code: style classes always on content; hide the marker characters only when no selection range touches the element (inclusive of edges).
- Use only these lezer node types: ATXHeading1-6, HeaderMark, StrongEmphasis, Emphasis, EmphasisMark, InlineCode, CodeMark. Ignore all others (lezer can parse `[x]` as a Link; do not render those).
- Wikilink `[[Title|id]]`: found by regex, NOT the syntax tree. Title is anything except `]` and newline (may contain `|`; split at the LAST `|`). Id has no whitespace, `]` or `|`. Replace with a widget, registered in EditorView.atomicRanges, never revealed. Widget text = live title from a titles map (id -> {title, disabled}) supplied by Rust, else the cached title in the token styled as "missing". Click calls onLinkClick(pageId). The widget handles its own click and tells CM to ignore its events.
- Tree-derived decorations must skip anything inside a wikilink range (no overlapping replace decorations).

## Edit translation (the hard part)
- Update listener: skip updates whose transactions carry a "from Rust" annotation (also mark those remote and addToHistory false, so undo never reverts them and no echo loop happens). Otherwise use update.changes.iterChanges and send ONE array [{from,to,insert}] where from/to are UTF-16 offsets in the OLD document. Send nothing else.
- Rust holds the pre-edit authority string. For each change, convert to a TextEdit with the same shape as the real project (yrs/src/yrs_wrapper/types.rs): Insert{text,position}, Delete{text,position}, Replace{old_text,new_text,position}. Pure insert -> Insert, pure delete -> Delete (text sliced from the old string), both -> Replace. Because all positions are old-doc coordinates, APPLY IN DESCENDING POSITION ORDER so earlier positions stay valid.
- UNITS: first write a native test to learn what index unit yrs uses by default for XmlText insert/remove_range (I believe bytes; verify with emoji and CJK). Then build the authority Doc with whichever offset kind lets CM's UTF-16 offsets pass straight through (yrs Options offset_kind Utf16), and make Delete/Replace lengths count in the same unit. Put this finding at the top of FINDINGS.md; the real project's BossOfYrs and edit_block_text (which uses chars().count()) may have the same issue.
- Authority storage mirrors the real shape: Doc -> Array "blocks" -> Map with "id" and "text" (an XmlTextRef). One doc per block is fine.
- Mismatch policy: after applying, compare the authority string with cmBridge.getText. On mismatch log loudly, show a red mark, and do NOT auto-heal; only the Resync button replaces the CM text from authority.

## External / remote text -> CM
- cmBridge.applyExternalText(id, newText): compute common prefix and suffix against the current doc (never splitting a surrogate pair), dispatch ONE minimal change with the from-Rust annotation so CM keeps the caret.
- Remote simulation in Rust: create a second yrs Doc (peer) from the authority snapshot, edit it, encode the diff against the authority's state vector, apply it to the authority, read the new string, call applyExternalText. Buttons: edit at start, middle, end, inside a link token (it will break into plain text; document that), delete a whole token.

## Intents (lower priority, after the above works)
A Prec.high keymap that consumes and reports via onIntent JSON, without editing: ArrowUp at doc start and ArrowDown at doc end (empty selection, no modifiers) -> nav; Enter on non-empty doc -> split with before/after strings; Backspace at position 0 with empty selection -> merge_up. Rust logs them and moves focus between the 3 blocks (expose cmBridge.focus(id, "start"|"end")). This proves the app's existing textarea key behaviours can be re-hooked.

## Optional last
@codemirror/autocomplete with an override source triggered after `[[`, options from the titles map, inserting `[[Title|id]]` (strip `[`, `]`, newlines from the title).

## Other editor setup
history() plus defaultKeymap. Verify undo emits ordinary changes to Rust. lineWrapping, no fixed height, styled to look like a plain textarea. A ResizeObserver per editor shows its height in the debug panel (the real app's reorder layout measures block heights with a ResizeObserver).

## Link parsing shared by JS and Rust
Write pure Rust `parse_links(raw) -> Vec<{title, page_id, ...}>` (the real app will use it for backlinks) and a shared `test_vectors/links.json` of {input, expected [(title,id)]}. Both a `cargo test` and a node script must pass the same vectors. Cases: title containing `|`, emoji title, two adjacent tokens, missing id, id with a space (not a link), `[[a|b]]]`, nested brackets.

## Harness page
3 blocks seeded with: (1) heading + bold + italic + code; (2) two link tokens, one pointing at a page missing from the titles map; (3) emoji, CJK, a ZWJ family emoji, accented letters, and a long paragraph for wrapping.
Debug panel per block: CM text, authority text, in-sync mark, last 10 edit batches as TextEdit debug output, height, link-click log, intent log.
Buttons: the remote edits above, rename page (titles map change), disable page, Resync, Fuzz, and Copy report (puts a plain-text report of everything above on my clipboard).
Fuzz: Rust generates random multi-change non-overlapping transactions (insert/delete/replace on char boundaries incl. emoji), sends them through cmBridge.dispatchChanges (a non-remote dispatch, simulating paste or multi-cursor), then compares authority and CM after each. Run 500 times; report the first mismatch with its seed.

## Pitfalls to avoid
- Closures given to JS must stay alive in a struct owned by the component. Call cmBridge.destroy and drop them in on_cleanup.
- Callbacks from JS run synchronously inside CM's dispatch. Do minimal work, never hold a RefCell borrow or signal update while calling back into JS, and release borrows before calling applyExternalText or dispatchChanges.
- Rust must never receive or reason about visual positions.

## Deliverables
The project; README.md with a text diagram of the data flow and a read-this-first file order; FINDINGS.md covering the yrs offset finding, surprises, anything awkward, and what integrating into the real app would replace (the textarea, get_diff, map_caret_across_edit); SPIKE_PROGRESS.md kept current.

## Non-goals
Persistence, batcher, server, drag/reorder, real page picker, styling polish.
