# Current state

- web_interface is a Leptos csr app, built and served with trunk.
- Routes: `/` and `/Menu` (Menu, a nav page), `/DbGui` (the orm, comes
  from the web_internal_db crate in z_db).
- The internal db is all wired up. z_db's web_internal_db crate has the
  methods to call for doing crud stuff.

- create_page in the builder works. It builds a BossOfYrs, inserts two blocks (title, then a normal one), snapshots the doc, takes a version bookmark, and returns all the prepared data holding three ctx structs.

- the executor takes all the prepared data and does the three inserts in one transaction: pages, every_block_in_existence, uncommitted_diffs.

- there's a method for editing a block, and it's also used for adding a title to a fresh page.

- checkbox system almost finished. the goal is to have a button (which works) that deletes existing db stuff then seeds based on selected checkboxes.

- main.rs creates 3 tables if they don't exist.
