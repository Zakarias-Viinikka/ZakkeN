# Current state

- web_interface is a Leptos csr app, built and served with trunk.
- Routes: `/` and `/Menu` (Menu, a nav page), `/DbGui` (the orm, comes
  from the web_internal_db crate in z_db).
- The internal db is all wired up. z_db's web_internal_db crate has the
  methods to call for doing crud stuff.
- action_buttons/ exists as a folder (page_edits.rs, inside_page_edits.rs)
  but has no real content yet.
- Nothing else is built. No yrs, no blocks, no replay.

## AI MADE

- small_components/ added, with happy_little_checkbox.rs. A reusable
  checkbox that takes a read signal, a write signal, and a callback it
  runs at startup if the box starts checked.
- action_buttons/page_edits.rs has AllCheckBoxStates, which calls
  use_local_storage to remember one checkbox across reloads. Not wired to
  the component yet — the HappyLittleCheckbox usage is commented out.
- leptos-use added to Cargo.toml. codee is imported in page_edits.rs but
  not yet in Cargo.toml, so it won't build until that's added.
- page_edits.rs also has button handlers stubbed: create_new_page and
  delete_page just log, move is a todo, and up/down buttons bump a
  selected-title counter.

- Changed the db connection name in zdb_web_output/worker_wrapper.js to "cqrs".
