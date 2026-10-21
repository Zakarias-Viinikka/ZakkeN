# general todo

move a page.

blocks need an ordering key. either main_menu_position on the block row, or a separate table with a map.

verification for generalized methods that check if stuff is in sync. a way to run these checks for various things: whether everything agrees on active pages, on the number of blocks in a page, and on the content of blocks (that's what tests will assert on later).

inside_page_edits.rs is fully commented out. Component renders a Stylesheet and nothing else. Do this after the main pages work - it's for editing blocks inside a real page.

I should be updating the activePages when i make a new page.

the docs are misleading about what active_pages is. fix them.

testing support using testing_wasm32_template (ZakkeN/testing_wasm32_template, its README has how to run the tests).

a way to create fake server uncommitted diffs that the client is supposed to receive. they need to be stored in a table.

proper schema migration. details to be worked out.

the builder shouldn't hardcode values that depend on where it runs. create_page currently makes its yrs boss with a hardcoded user id and an empty time string. instead the builder should request them: the caller (web, a test) passes in the user id and the time, and the builder only uses what it was given. that way each target supplies its real values and tests can supply fixed ones.

two tabs with the same page open isn't supported. add a deliberate check that shows a message like "edits aren't working right now, currently does not support two tabs having same page open".

addblock, editblock and deleteblock need to exist in the builder, the executor and the web interface.

text_diff becomes a dependency of the cqrs crate.

every executor that affects a page first sends a message through a tx saying "batch immediately if still unfinished". The receiving side is the rx that the batcher owns.

timer logic for the batcher.

the batcher is owned by main.rs (or similar) and passed around, so a page swap is safe and doesn't lose data.

a block auto-creates a new block when it accumulates enough linebreaks, but only when the newline is added to the bottom of the block. Adding a newline above or in the middle does nothing.

pages inside pages, and the syntax for it. The special menu opens on z+space, deactivates on esc or space again, and always expects the down arrow to select even while the menu is showing. Needs the component that brings up the menu, and the typed-trigger that replaces the text with the menu. The planned details of the typed-trigger are written down somewhere, location unknown - find that first.

on-leave behaviour: a general way to express "if the page changes and it's not x page, clean out y". Implemented with normal Leptos signals, not rx/tx. Write handles are named by when they write rather than by what they are; read-only side keeps conventional names.

the loading system - preload, offline, server - is described in ZakkeN/cache_and_preload.md. Read that instead of repeating it here.

the worker OPFS should not clear unless it has to. Track how old a db entry is so the oldest can be evicted.

page state must be visually clear: connected and up to date, currently pushing changes, offline, and so on, so the user always knows where a page stands.

a method that checks whether yrs and the table are in sync. Android's CheckIfTablesInSync is the reference.

flush on close and on visibilitychange, so the last batch isn't lost when the tab goes away.

everything built here gets reflected in the Android docs, so Android knows what it can use as a reference and what it still needs to implement.

add_block and delete_block are built in the builder and executor, plus uniffi exports. Not wired into the web interface yet, because the page UI for it hasn't been designed - that's still in a lab and the lab needs auditing first.

pure_web_ui_components crate exists at cqrs_state_synchronization/pure_web_ui_components. Exports PagesUiThatOnlyOwnsDragReorder with PageInput / BlockInput / MoveEvent. Came from the drag_reorder final_test lab, split into modules (drag_state, rebuild, heights, sync, apply_move, mouse, plus the untouched cascade / layout / rapier_world). Visual confirmation stripped - that was testing-only. Stage background removed, parent owns it. Nothing in web_interface calls it yet; the real page UI hasn't been designed.

Moving a block within a page works end to end on web (yrs method, love letter variant, builder, executor, ui_action, on_move wired). After a drag the page does not re-order until reload, because the block signal is re-read from db only on load.

Moving a block to another page works on web. ui_actions_move_block_operations::move_block_to_another_page runs delete_block on the old boss and add_block(Some(block_id), ...) on the new boss, both executors in one transaction. Two letters, not one: RemoveBlock deletes the every_block row on the old page, CreateNewBlockWithId recreates it on the new page under the same block id. At-least-once delivery makes this safe.

Back button: navigating back to a previous route doesn't re-render without a manual refresh. Cause not yet diagnosed.

Preload flow lives in complicated_signal_flow/core.rs. Page is the one struct for a page's live state: page_id, boss_of_yrs, blocks, page_state, preload_request, preload_response. PageLoadState is the enum, with PreloadLoaded(usize) carrying the count. Page::new builds one; MainContent builds one per page and Copy hands it into that page's effects.

The loading sequence is two effects per page. The state effect matches on page_state, calls preload_pages / load_offline / load_from_server for that step, and the resulting state change fires it again for the next step. The response effect watches preload_response; when it flips to Some it sets the blocks and moves to PreloadLoaded(count). preload_pages writes preload_request, and - because no preloader exists yet - also seeds preload_response with an empty Vec. That fallback goes away when the preloader exists.

Batcher notification has no home right now. When the batcher exists it needs a signal for "this page is about to change", and the executor crate has to reach it, which isn't solved.

BlockRow is the one block type. It lives in complicated_signal_flow/core.rs. There is no BlockInput anymore.

dev_actions::insert_three_pages seeds three pages with varied titles and bodies.

## Focus the new block when split creates one

When three newlines in a row split a block, focus should land at the
start of the new block. Currently the focus effect runs and the new
textarea is `document.activeElement`, but typing still goes into the
old block. `tmp_focus_test` at `ZakkeN/tmp_focus_test` reproduces the
split-and-focus mechanic correctly, so the cause is structural in the
real project. Suspects: the async gap between the input handler and
the DB-backed block insert, and the two-signal layer
(`page.blocks` -> `boxes` -> `For`) keyed by `internal_id`.

## tmp_focus_test is a reproduction harness

`ZakkeN/tmp_focus_test` is a minimal Leptos project that replicates
the split-and-focus mechanic. Currently green. Use it to isolate the
focus-on-split bug by incrementally adding real-project mechanics
(async split, two-signal layer, global focus static, DB reload before
focus). Delete when the bug is fixed and the fix is verified in the
real project.
