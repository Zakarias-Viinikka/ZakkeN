# context

Things to read to understand what cqrs_state_synchronization is about.

## Other docs in this project

- ZakkeN/android_app/docs/SHARED_MEMORY.md
- ZakkeN/android_app/docs/AI_READ_THIS.md
- ZakkeN/android_app/docs/UX_INTENT.md

## Crates it depends on

- yrs — the CRDT. The authoritative state.
  ZakkeN/yrs
- client_table_blueprints — the tables the actual projects use to store
  the data that needs to be synced.
  ZakkeN/client_table_blueprints
- z_db — a sqlite wrapper plus its protocol crate (payloads, Row/Col,
  errors). Used here for two different things:

  1. The browser's internal crud solution. web_internal_db is an
     extension to it. Lives inside z_db.

  2. Later: z_db also has more use cases. It will be used to test server
     and client talking. In that case a shared protocol defines the
     payloads and whatnot. That is separate from the payloads in z_db.

  ~/ProgStuff/z_db
  ~/ProgStuff/z_db/web_internal_db
- text_diff — batches yrs edits. The clients use it to batch edits, so
  the tester has to as well.
  ZakkeN/text_diff
- love_letter — so the server and client agree on how to describe
  "i want to do x" to the yrs docs.
  ZakkeN/love_letter

- error_stuff is so the workspace has a shared error even though there are multiple error types.
- data_builder_for_operations_that_need_to_be_correct is basically the reference for how any operation should be done.
- executor_of_what_the_builder_built_because_the_builder_shouldnt_touch_the_db is the one that executes it.
- they're split because executing requires specific context, and in this one it's wasm.

## Reference code in the android app

- every_block_in_existence — the mirror table.
- pages.blobbed_page — the authoritative yrs blob.
- uncommitted_diffs — intent + yrs diff snapshot.
- DocEvents/AddBlock.kt — how position is computed on insert.
- DocEvents/EditTextInBlock.kt — how edits apply to yrs and the mirror.
- DocEvents/CheckIfTablesInSync.kt — the check.
- Tests/PageDataSyncTest.kt — the closest thing to what this crate is for.

## create_page and is_main_menu_page

The button in the main menu calls create_page with is_main_menu_page hardcoded to true. That's correct — that's how creating a page in the main menu is supposed to work. If you create a page from inside a page, that would pass false. That's way down the line.
