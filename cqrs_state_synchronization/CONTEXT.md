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

## Reference code in the android app

- every_block_in_existence — the mirror table.
- pages.blobbed_page — the authoritative yrs blob.
- uncommitted_diffs — intent + yrs diff snapshot.
- DocEvents/AddBlock.kt — how position is computed on insert.
- DocEvents/EditTextInBlock.kt — how edits apply to yrs and the mirror.
- DocEvents/CheckIfTablesInSync.kt — the check.
- Tests/PageDataSyncTest.kt — the closest thing to what this crate is for.
