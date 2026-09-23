# Current state

- web_interface is a Leptos csr app, built and served with trunk.
- Routes: `/` and `/Menu` (Menu, a nav page), `/DbGui` (the orm, comes
  from the web_internal_db crate in z_db).
- The internal db is all wired up. z_db's web_internal_db crate has the
  methods to call for doing crud stuff.
- action_buttons/ exists as a folder (page_edits.rs, inside_page_edits.rs)
  but has no real content yet.
- Nothing else is built. No yrs, no blocks, no replay.
