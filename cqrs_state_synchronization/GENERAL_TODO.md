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