# Drag container + markdown editor: stored context for AI

Reference for any AI that touches the drag reorder container, the markdown editor, the caret code or the special menu.
Crate: `web_interface` (Leptos CSR), root `/home/zakke/ProgStuff/ZakkeN/cqrs_state_synchronization/web_interface/`. All paths below are relative to that root.
Read `/home/zakke/ProgStuff/ZakkeN/coding_style.md` first (vocab: handle, wrapper, core, main, shared, guard, bbox, plex, long names, file order).

Last updated after wiring the special menu (Ctrl+E, link to existing page) into the container. Things marked UNVERIFIED were not read or not confirmed on disk. Check with `cat` before relying on them.

## Rules of engagement

- `bbox_drag_reorder_container/drag_reorder_black_box/` and `markdown_conversion/` are bbox (code the user has not reviewed). If a feature needs changing their internals, tell the user first. Refactoring them is not a cost, keeping them maintainable matters more than shipping the feature fast.
- The user reviews diffs in zapgui. Send zap chains with absolute paths. Do not assume a chain applied: silence from the user usually means it worked, but verify with `cat`, `ls` or `rg` before building on top of it. Several times a chain was assumed applied and was not.
- All offsets are UTF-16 code units (CodeMirror, `CaretPlace::At`, `keys::split_utf16`). Never use Rust byte or char offsets for caret positions.
- `lib.rs` / `mod.rs` hold only `pub mod`, `pub use`, `use`.

## Map of the code

### Container: `src/bbox_drag_reorder_container/`
- `mod.rs`: re-exports `DragReorderContainer`, `BlockRenderInput`, `MoveEvent`, `PageInput`.
- `drag_reorder_container.rs`: `DragReorderContainer`, the thin wrapper. It builds `render_block` (renders `MyBlockContent` per block), renders `DragReorderBlackBox`, hosts the special menu, owns the Ctrl+E listener. Props: `pages`, `blocks`, `on_move`, `on_block_input`, `on_delete_request`, `on_merge_with_above`, `on_split`, `on_navigate`. It also does `expect_context::<OpenPages>()`.
- `my_block_content.rs`: thin adapter from `BlockRenderInput` to `MarkdownConversion`. Also builds `LastEditInfo` from the editor's caret events and writes the container's `last_edit` signal.
- `shared_internally.rs`: `LastEditInfo { block_id: String, position_last_selected: usize }`, derives Clone, Debug, Default. Only for files inside this folder. Default (empty block id, position 0) means nothing touched yet.
- `main_for_inserting_link_to_existing_page.rs`: `main_for_inserting_link_to_existing_page_at_position(open, block_id, utf16_position, page_id)`.
- `drag_reorder_black_box/` (bbox, mostly UNVERIFIED): `drag_reorder_black_box.rs` (component), `rapier_world.rs`, `layout.rs`, `mouse.rs`, `sync.rs` (`sync_from_props`), `heights.rs`, `cascade.rs`, `drag_state.rs`, `apply_move.rs`, `rebuild.rs`, `types.rs`, `drag_reorder_black_box.css`. Blocks are absolutely positioned boxes owned by rapier2d, not normal flow. A ResizeObserver reports each box's height. The title block has no drag handle. `DragReorderBlackBox` props: `pages`, `blocks`, `on_move`, `render_block: Callback<BlockRenderInput, AnyView>`.
- `BlockRenderInput { block_id: String, content: ArcRwSignal<String>, ghost: Signal<bool>, is_title: bool }`. `MoveEvent { moved_block_id, from_page_id, to_page_id, above, below }`.

### Editor: `src/markdown_conversion/` and `markdown_js/`
- `markdown_conversion.rs`: `MarkdownConversion`, a blackbox CodeMirror editor for ONE block. `content: ArcRwSignal<String>` is the single source of truth. User typing is applied to it, outside changes to it are pushed into the editor (minimal diff, caret kept, no echo). Callbacks carry the block id. Props include `on_input`, `on_delete_request`, `on_merge_with_above`, `on_split`, `on_navigate`, `is_title`, `ghost`, `placeholder`, `on_link_click`, `on_caret_moved`, `titles`, `class`.
- `cm.rs`: the ONLY file that talks to the JS bridge (`window.cmBridge`). JS to Rust events arrive as window CustomEvent `cm` with a JSON detail, parsed into `CmEvent` (`Changes`, `Key`, `LinkClick`, `Selection`). `CmEditor` is the component.
- `keys.rs`: pure Rust key decisions (`decide`). Reported keys: ArrowUp, ArrowDown, Enter, Backspace, Delete only. Also has `split_utf16`.
- `edit.rs`: `Change`, `apply_changes` (UNVERIFIED, not read).
- `main_for_on_link_click/`: click handling for `[[command|rest]]` links (see Links below).
- `markdown_js/src/`: `editor.js` (CodeMirror wrapper, no app logic), `decorations.js` (link widgets, headings, emphasis), `link_parser.js`, `wikilink_complete.js` (autocomplete after `[[`), `theme.js`.
- BUILD: run `./rebuild_markdown.sh` in web_interface/ after ANY change in `markdown_js/src/`. It runs `npm run build` (esbuild, iife) into `markdown_web_output/cm_bridge.js`, which `index.html` loads as a classic script. Editing the JS without rebuilding changes nothing.

### Caret and app state
- `src/complex_state/caret/caret_request.rs`: the one pending caret request (plex pattern, `plexget_`/`plexset_`).
- `src/complex_state/caret/caret_handlers_registry.rs`: block_id to handler registry. Each `MarkdownConversion` registers one when its editor is ready.
- `src/complex_state/caret/focus.rs`: caller pov, `focus::at_end`, `at_start`, `at_pos(block_id, utf16_pos)`. Re-exported as `complex_state::focus`.
- `src/leptos_components/main_content/caret_owner.rs`: `CaretOwner`, the one place that places carets (next animation frame, retries `At(pos)` beyond the text, clamps on the last try).
- `src/complex_state/rows.rs`: `BlockRow { block_id, page_id, content: String, is_title }`. `LocalPages` is a page-list row (title, page id, disabled), NOT block data.
- `src/page_state.rs`: `Page { page_id, boss_of_yrs, blocks: RwSignal<Vec<BlockRow>>, page_state, preload_request, preload_response }` (Copy), `OpenPages { page1, page2: RwSignal<Option<Page>> }` (Copy), with `page_for_block`, `page_and_boss_for_block`, `boss_for_page`. `OpenPages` is global: `main.rs` does `provide_context(open_pages)`, anyone can `expect_context::<OpenPages>()`.
- `src/leptos_components/main_content/main_content.rs` and `callbacks/` (`on_block_input`, `on_delete_request`, `on_merge_with_above`, `on_move`, `on_navigate`, `on_split`): each has `install(open) -> Callback<...>`. They change rows and push to the batcher. `MainContent` passes page1 only to the container.

### Special menu
- `src/leptos_components/special_menu/`: `mod.rs`, `core.rs` (`SpecialMenu`, `WriteField` keyword picker, `NewPage` which is still a todo), `internally_shared.rs` (`TitleMatch { title, page_id }`, `MenuOption`), `cmp/link_to_other.rs` (`LinkToExistingPageComp`, searches titles through `db::reads_wrapper::search::search_page_titles`, fts5).
- `css/special_menu.css`: fixed, centered, 500x500 minimum, white background, shadow, `z-index: 1000`. Input id is `special_menu_text_field` (shared by both components).
- `SpecialMenu` takes `cb_finish_linking_to_existing_page: impl Fn(TitleMatch)`.

## Data flows

Typing: CodeMirror `changes` event, then `cm.rs` `Edit`, then `MarkdownConversion::on_edit` applies the changes to `content` and runs `on_input(BlockInputEvent { block_id, old_text, new_text, was_enter, cursor_pos })`. Then `callbacks/on_block_input.rs` does `text_diff::diff_logic::get_diff`, `batcher_handle::push_edit(boss, block_id, diff)`, and updates `page.blocks`. The `blocks` signal flows back down (`sync_from_props` into `content` signals) and the editor follows.

Programmatic edit (the pattern to copy, see `main_for_inserting_link_to_existing_page.rs` and `callbacks/on_split.rs`): find the page with `open.page_and_boss_for_block`, compute the new text, `get_diff`, `push_edit`, update `page.blocks`, then `focus::at_pos`. Do not write into the editor directly.

Caret IN (place it): `focus::at_*` sets the request, then `CaretOwner` calls the block's registered handler, which calls `cm::set_selection` and `cm::focus_keep_caret`.

Caret OUT (report it): `editor.js` emits `selection {head}` when (selectionSet or docChanged or focusChanged) AND the view has focus, then `CmEvent::Selection`, then `CmEditor` prop `on_caret_moved`, then the `MarkdownConversion` prop `on_caret_moved`, then `MyBlockContent` builds `LastEditInfo { block_id, position_last_selected }`, then it writes the container's `last_edit: RwSignal<LastEditInfo>`. `MarkdownConversion` never sees `LastEditInfo`. Because editors only report while focused, `last_edit` stays frozen while the menu's text input has focus. That is why the container needs no snapshot.

Special menu flow: Ctrl+E (a `window_event_listener(keydown)` in the container, with `prevent_default` and `stop_propagation`, pattern copied from `/home/zakke/ProgStuff/suggestion_for_leptos_book/leptos_starter/hotkeys/src/main.rs`) toggles `special_menu_is_open`. Closing with Ctrl+E puts the caret back with `focus::at_pos(last_edit...)`. Picking a page closes the menu and calls `main_for_inserting_link_to_existing_page_at_position(open, last_edit.block_id, last_edit.position_last_selected, picked.page_id)`. That shows a popup if the block id is empty ("click into a block first") or no open page has the block, otherwise it inserts the link and puts the caret right after it.

## Links

- Syntax `[[command|rest]]` (`link_parser.js`: command has no `]`, `|` or newline, rest may contain anything except a newline, the `|` is required).
- A click calls `main_on_link_click(command, rest)` in `main_for_on_link_click/core.rs`, which looks the command up in `COMMANDS`: `Install x`, `Install y`, `Install c`, `Install all things` (dummies that log), and `PageNav` (`rest` = page id, calls `plexset_navigation_request("/MainContent/{page_id}")`). Unknown commands log an error.
- The menu inserts `[[PageNav|page_id]]`.
- `decorations.js` `WikiLinkWidget` shows the COMMAND as the visible text and ignores the titles map. So an inserted link displays as the word "PageNav", not the page title.
- `wikilink_complete.js` (autocomplete after `[[`) inserts `[[title|id]]`. That displays the title but a click fails with "unknown link command", because the title is read as the command.

## Decisions the user made (do not relitigate)

- No plexget/plexset for the caret position. It is contained to the container folder and travels by props. `LastEditInfo` lives in `shared_internally.rs` on purpose.
- Passing the `RwSignal<LastEditInfo>` to every block is fine (Copy, only read untracked, one `set` per caret move).
- No `Option` and no fallback to the first block. Empty or stale block id means a popup.
- Editing goes through the global `OpenPages` from inside the container folder, not through a new prop or callback.
- Menu clicks: only the menu's own box blocks clicks to the page behind it. No full-screen backdrop.

## Open items

1. Link display and the autocomplete mismatch (see Links). Option discussed: in `WikiLinkWidget`, when the command is `PageNav`, show the title found in `titlesField` for `rest` (fall back to "PageNav"), and make the autocomplete insert `[[PageNav|id]]`. Needs `MyBlockContent` to pass `MarkdownConversion` a `titles` signal, which it does not today. Not decided by the user.
2. The "new page" menu option (`NewPage`) is a todo.
3. Menu styling: white background only. Buttons are unstyled, the locked-state dark red page (`#4a1a1a` in `main_content.rs`) is not matched, and there is no padding around the field.
4. `DragReorderBlackBox` listens for `mouseup` on the window, so releasing a drag over the open menu still counts. Ignored for now.
5. `page2` exists in `OpenPages` but `MainContent` only gives the container page1's blocks. `main_for_inserting_link_to_existing_page` already handles either page through `page_and_boss_for_block`.
6. `TitleMatch.title` is no longer used by the insertion. It is still in the struct and shown on the buttons.
7. `main_for_on_link_click/install_c.rs` showed a stray `"` line in a paste, probably a paste artifact. If `cargo check` complains there, that is why.
8. The whole feature is untested at runtime. Test: click into a block, press Ctrl+E, pick a page, check that `[[PageNav|id]]` appears at the caret and the caret lands after it. Also press Ctrl+E before clicking anywhere (popup expected), and press Ctrl+E twice (caret returns).

## Quick checks for the next AI

- Names and wiring: `rg -n "special_menu|SpecialMenu|LastEditInfo|on_caret_moved" src css`
- JS bridge is current: `rg -n "selection" markdown_web_output/cm_bridge.js` (no hit means `./rebuild_markdown.sh` was not run).
- Container as it is on disk: `cat` `src/bbox_drag_reorder_container/drag_reorder_container.rs`.