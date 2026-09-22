# context

Things to read to understand what cqrs_state_synchronization is about.

## Other docs in this project

- ZakkeN/android_app/docs/SHARED_MEMORY.md
- ZakkeN/android_app/docs/AI_READ_THIS.md
- ZakkeN/android_app/docs/UX_INTENT.md

## Crates it depends on

- yrs — the CRDT. The authoritative state.
  ZakkeN/yrs
- client_table_blueprints — table schemas and row creation helpers.
  ZakkeN/client_table_blueprints
- z_db — SQLite wrapper and the protocol crate (payloads, Row/Col, errors).
  ~/ProgStuff/z_db
- text_diff — character-level diffing and squashing for text edits.
  ZakkeN/text_diff
- love_letter — intent shapes (LoveLetterSketch) for client-server sync.
  ZakkeN/love_letter

## Reference code in the android app

- every_block_in_existence — the mirror table.
- pages.blobbed_page — the authoritative yrs blob.
- uncommitted_diffs — intent + yrs diff snapshot.
- DocEvents/AddBlock.kt — how position is computed on insert.
- DocEvents/EditTextInBlock.kt — how edits apply to yrs and the mirror.
- DocEvents/CheckIfTablesInSync.kt — the check.
- Tests/PageDataSyncTest.kt — the closest thing to what this crate is for.
