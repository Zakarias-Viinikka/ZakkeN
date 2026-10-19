# general todo

move a page.

blocks need an ordering key. either main_menu_position on the block row, or a separate table with a map.

verification for generalized methods that check if stuff is in sync. a way to run these checks for various things: whether everything agrees on active pages, on the number of blocks in a page, and on the content of blocks (that's what tests will assert on later).

inside_page_edits.rs is fully commented out. Component renders a Stylesheet and nothing else. Do this after the main pages work — it's for editing blocks inside a real page.

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

pages inside pages, and the syntax for it. The special menu opens on z+space, deactivates on esc or space again, and always expects the down arrow to select even while the menu is showing. Needs the component that brings up the menu, and the typed-trigger that replaces the text with the menu. The planned details of the typed-trigger are written down somewhere, location unknown — find that first.

on-leave behaviour: a general way to express "if the page changes and it's not x page, clean out y". Implemented with normal Leptos signals, not rx/tx. Write handles are named by when they write rather than by what they are; read-only side keeps conventional names.

the loading system — preload, offline, server — is described in ZakkeN/cache_and_preload.md. Read that instead of repeating it here.

the worker OPFS should not clear unless it has to. Track how old a db entry is so the oldest can be evicted.

page state must be visually clear: connected and up to date, currently pushing changes, offline, and so on, so the user always knows where a page stands.

a method that checks whether yrs and the table are in sync. Android's CheckIfTablesInSync is the reference.

flush on close and on visibilitychange, so the last batch isn't lost when the tab goes away.

everything built here gets reflected in the Android docs, so Android knows what it can use as a reference and what it still needs to implement.

add_block and delete_block are built in the builder and executor, plus uniffi exports. Not wired into the web interface yet, because the page UI for it hasn't been designed — that's still in a lab and the lab needs auditing first.
