# Current state

- web_interface is a Leptos csr app, built and served with trunk.
- Routes: `/` and `/Menu` (Menu, a nav page), `/DbGui` (the orm, comes
  from the web_internal_db crate in z_db).

- create_page in the builder works. It builds a BossOfYrs, inserts two blocks (title, then a normal one), snapshots the doc, takes a version bookmark, and returns all the prepared data holding three ctx structs.

- the executor takes all the prepared data and does the three inserts in one transaction: pages, every_block_in_existence, uncommitted_diffs. The executor fns check whether a transaction is already open; if one is, they don't begin or commit, and the outer call owns it.

- there's a method for editing a block, and it's also used for adding a title to a fresh page.

- disable_page marks a page disabled instead of deleting it. The page row stays, every block for it gets page_is_disabled = "true", and a new active_pages blob goes into pages.page_status. The menu filters disabled pages out on load.

- the menu reads its page list from db on load. LocalPages holds db_id, title, block_id, page_id, is_disabled. There's no page counter anymore.

- the dev sidebar has two buttons: delete everything and insert three pages. Both do their db work, then reload the page.

- the eye button next to each menu title disables that page.

- main.rs creates 3 tables if they don't exist.