# Linking to another page

Write this in any block:

    [[PageNav|the page id]]

The block shows "PageNav" as a clickable link. Clicking it opens that page.

## Finding the page id

Open the page you want to link to. The address ends in `/MainContent/<page id>`. Copy the part after the last slash.

## Rules

- The first part is the command. It must be exactly `PageNav` (capital P, capital N).
- Everything after the first `|` is the page id. Spaces around it are ignored.
- The `|` is required. `[[PageNav]]` stays plain text.
- An empty id logs an error and nothing happens.
- A wrong id: I haven't checked what the page shows.
- The old `[[Title|id]]` form no longer works. It reads "Title" as a command, so clicking logs an unknown-command error.
- Typing `[[` may open the autocomplete. It still uses the old format and currently offers nothing.

## Where it lives

`src/markdown_conversion/main_for_on_link_click/`. `page_nav.rs` is the PageNav command and `core.rs` holds the list of commands.

## Status

Built and compiles. Clicking a link in the browser has not been confirmed yet.
