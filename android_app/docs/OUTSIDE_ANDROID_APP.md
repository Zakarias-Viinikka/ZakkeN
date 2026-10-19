# Outside Android App

Context the Android app needs that it can't see from inside its own tree.
Read this before making decisions about how Android should behave.
Anything in here is a constraint, not a suggestion.

## What ZakkeN is

An offline-first Notion-style notes app. The domain is pages and blocks.
A page is a list of blocks; a block is text plus metadata. Every page's
state lives in a yrs CRDT doc; the sqlite mirror is queryable but not
authoritative.

Edits are "love letters" — serialized intents the client sends to the
server so both sides agree on what changed. The client stores its own
copy in uncommitted_diffs; the server tracks which registered clients
have received which letters.

You are the Android client. cqrs_state_synchronization/web_interface is
the reference implementation for behaviour. Android does UI differently
(Compose, mobile), but the underlying logic should match. When in doubt,
look at the web version and mirror it.

## The crates you depend on

All of these come across the uniffi boundary, as Kotlin in the rustlib.*
or uniffi.* package namespaces.

yrs (rustlib.my_yrs_lib) — CRDT wrappers. BossOfYrs owns one page's doc.
YrsActivePages owns the "which pages are disabled" doc. YrsBacklinks
owns backlinks. The crate's lib name is my_yrs_lib even though the
package name is yrs. BossOfYrs::new takes a unix_time string as its
second argument.

love_letter (rustlib.love_letter) — the edit contract. LoveLetterSketch
is the intent (EditBlock, CreateNewBlock, RemoveBlock, DisablePage,
CreateNewPage). LoveLetter is the same plus a snapshot blob. bincode
serialized. No dependency on yrs.

text_diff (rustlib.text_diff) — character-level diff and squashing.
getDiff(old, new) -> DiffResult. combineGetdiffResults(list) squashes.

client_table_blueprints (rustlib.client_table_blueprints) — the schema.
newTableX() returns a List<ColumnDef> for table X. newXRow(...) returns
a List<ColumnValue> ready to insert. Also exports getForeignDefX() for
foreign keys.

protocol (uniffi.protocol) — payload types, Row, Col, ColumnDef,
ForeignKeyDef, the various In/Out structs.

db_wrapper (com.z_db.android_mascot) — LiveForever, the sqlite
connection. Methods: createTable, createForeignTable, listTables,
getData, insertData, editColInRow, editColInRowWhere, deleteRow,
getDataOrdered, and so on.

data_builder_for_operations_that_need_to_be_correct
(rustlib.data_builder_for_operations_that_need_to_be_correct) — the
pure builder. Produces the data for one user action (create page, edit
block, disable page). Runs anywhere. Returns a ctx struct with
everything the executor needs.

executor_of_what_the_builder_built_because_the_builder_shouldnt_touch_the_db
— the executor. In Rust it is wasm-only (it touches web_internal_db).
For Android you write your own executor in Kotlin that mirrors its
behaviour. DbManager is where that work happens.

## What changed since the app was last touched

A lot. The Kotlin code currently calls functions and references columns
that no longer exist under those names, and calls row builders with the
wrong number of arguments.

### Column-list function renames

uniffi converts snake_case to camelCase.

  Old Kotlin call              New Kotlin call
  pagesColumns()               newTablePages()
  backlinksColumns()           newTableBacklinks()
  everyBlockInExistenceColumns()  newTableEveryBlockInExistence()
  uncommittedDiffsColumns()    newTableUncommittedDiffs()
  keyValueStorageColumns()     newTableKeyValueStorage()

newPageRow and the other row builders kept their names, but their
signatures changed.

### Column changes

pages — unchanged. id, page_id, blobbed_page, page_status, version,
is_main_menu_page.

every_block_in_existence — gained two columns:
  is_part_of_main_menu_page (Text) — inserted after is_title
  page_is_disabled (Text) — at the end
Full list: id, is_title, is_part_of_main_menu_page, content,
my_id_as_given_by_yrs, id_of_page_i_belong_to, position,
page_is_disabled.

backlinks — unchanged. id, page_that_holds_link_id,
page_being_linked_to_id, disabled, version.

uncommitted_diffs — unchanged. id, snapshot_of_edit,
love_letter_sketch, session_id, target_id.

key_value_storage — gained an id column. Rust's schema is now
  id (INTEGER, primary key, autoincrement)
  key (TEXT, unique)
  value (TEXT)
It used to have key as the primary key with no id. This is a real
schema change; it needs a migration.

incoming_love_letters — new. id, love_letter, target_page_id. Not
needed yet; that is server-side work.

logs — new. id, timestamp, level, category, source, session_id,
message, details, details_type. Needs to be created and written to.

### Row builder signature changes

new_page_row(page_id, is_main_menu_page, blobbed_page, version) — four
arguments now. The old call passed three.

new_every_block_in_existence_row(is_title, is_part_of_main_menu_page,
content, my_id_as_given_by_yrs, id_of_page_i_belong_to, position) — six
arguments.

new_log_row(timestamp, level, category, source, session_id, message,
details, details_type) — eight arguments. timestamp is passed in, not
computed inside.

new_incoming_love_letter_row(love_letter, target_page_id) — two
arguments.

## The schema-drift problem and the parallel structure

Rust's schema crate uses SchemaColumn<T> to encode each column's type.
T is String, Vec<u8>, f64 or i64. That gives compile-time type safety
on the Rust side.

uniffi does not support generics. Android cannot use SchemaColumn<T>.
So Android has no equivalent to Rust's type-checked columns.

You compensate with a parallel structure. You define your own column
objects — one per column, per table — with three fields: name, type,
and the unwrap function that turns a Col into the correct Kotlin type.
Group them into table objects.

Then write a canary test: a test that asserts each column has the type
you expect, with an error message and a comment saying what else needs
changing if it fires. The test is not comparing to anything real — it is
a hardcoded assertion. When the schema drifts, the test fails, you read
the failure, and fix the whole chain of code that depended on the old
type.

This is a temporary solution. A drift-detection tool that parses the
Rust source and reports the real schema is planned but not built.

You can still call newTableX() and newXRow() — those create tables and
build rows. What you cannot use is a generic SchemaColumn for type
checking.

## The builder/executor pattern

Every user action that touches the db goes through two steps.

Builder — pure. Takes the user's intent and produces a "ctx" struct
with everything that needs to be written. No db access. Runs anywhere.

Executor — takes the ctx and performs the db writes. Order matters.
Wraps in a transaction.

Rust already has this for create_page, edit_block and disable_page.
Android mirrors it: call the Rust builder to get the ctx, then write a
Kotlin executor that inserts the ctx.

The Rust executor lives in
executor_of_what_the_builder_built_because_the_builder_shouldnt_touch_the_db.
It is wasm-only. Your executor lives in Kotlin and uses DbManager.

For operations Rust does not cover yet — add block, remove block, split
block — you design your own builder and executor following the same
shape. Log what you invent (see below).

## The squash-first rule

Text edits are buffered (500ms debounce) and flushed in a batch.
Structural actions (split, merge, delete) must not run while text edits
are pending.

The rule: any action that is not a plain text keystroke must squash the
pending buffer first, and the flush must complete before the action
runs.

This applies to Enter (split), Backspace at position 0 (merge), tapping
another block, moving a block, deleting, and anything else that could
be affected by text not yet written.

The reason: if you merge a block while a pending edit has not been
flushed, the merge operates on the pre-edit content and the edit flushes
against a block that no longer exists. The current Android code discards
the pending buffer instead of flushing it. That is wrong.

## What you are allowed to invent

Not everything has a Rust counterpart. Structural operations do not have
a Rust builder yet. Design them in Android, mirror the shape of the
existing ones, and log them.

Log them in docs/INVENTED_ON_ANDROID.md. Every entry says what Android
does and where the equivalent web or Rust code would need to be written
so it can be checked later. When the web side catches up, someone reads
that file and verifies Android's choices are correct.

## What not to change

The web project. It is the reference. Do not touch it.

The Rust crates. If a Rust crate is wrong, say so and stop — do not edit
Rust to make Android compile.

The Android Lab's isolation rule. Lab code copies production code if it
wants to modify it; it never imports and modifies production behaviour.
If a Lab experiment breaks because production changed, fix the Lab's
copy, not production.

## The migration system

Android has its own migration system in Kotlin: MigrationManager,
SchemaVersion, and frozen snapshots in db/migrations/schemas/.

SchemaVersion.V0 is a frozen copy of the schema at version 0. LATEST
points at V0 today. assertLatestMatchesBlueprints() compares the latest
frozen version against the live Rust blueprints and throws if they
disagree.

Since the Rust schema has changed, this test fails. The correct fix is:
freeze a new version (V1) that matches the current live Rust blueprints,
add a migration step from V0 to V1 that alters the tables as needed, and
bump LATEST to V1. V0 stays as it is — it is history.

This is also the drift-detection mechanism for schema shape, as opposed
to column types (which the canary test covers).

## The db serialization model

On web, db access goes through a worker. A JS message queue serializes
every call. begin_all_or_nothing, everything_went_perfectly and
regret_everything tell the worker when a transaction is open.

On Android, DbManager does the same job with a Mutex and a
coroutine-context flag (DbTransactionElement). Nested withTransaction
calls are re-entrant. This is correct; do not change it.

On the Rust side, start_transaction_if_not_in_one!() and
end_transaction_if_i_own_the_transaction!() macros check whether a
transaction is already open. Android's equivalent is checking the
coroutine context for a DbTransactionElement.

## Logging

Logger.kt writes to Logcat only. It needs to also write to the logs
table.

The rule: while a transaction is open, log lines do not write to the
db. They go into an in-memory buffer. When the transaction commits, the
buffer is dropped. When the transaction rolls back, the buffer is
flushed to the db, outside the transaction.

On success, nothing extra is written. The buffer only exists so failure
can be logged without being rolled back with the transaction that just
failed.

Everything outside a transaction goes to the db as a normal insert.

The source parameter (file/class/function) is passed explicitly at every
call site. Per-feature helpers (ViewPageLogs and friends) fill it in;
direct calls pass it by hand. Stack-trace introspection is not used —
it breaks under R8 obfuscation.

## The nav menu title

The nav menu shows a page's title. The title is the content of the title
block — the block whose is_title column is "true" in
every_block_in_existence for that page. It is not page_id.

A page's page_id never changes. It is the stable identity. If the user
"renames" a page from the nav menu, that is an edit to the title block,
not a change to page_id.

If the title block is empty, show "Untitled" in a muted colour.

## Docs to update

After making changes, update these where they have drifted:
  docs/AI_READ_THIS.md — the library guide
  docs/SHARED_MEMORY.md — architecture highlights
  docs/UX_INTENT.md — only if UX behaviour changed

## The e2e test harness (2026-10-17)

An external Python/Appium test suite lives at `../e2e_android_tests/`. It is
not part of the Android build. It runs against the app already installed on a
physical phone, over adb. It does not build or install the APK — the human
rebuilds and installs via Android Studio or `android_app/build_and_install.sh`
before running tests.

Two things the app must provide for the harness to work:

- `TestTransactionReceiver` (`src/debug/`): a debug-only broadcast receiver
  that calls `DbManager.beginExternalTransaction()` and
  `endExternalTransaction(commit)`. When active, every DB write the app makes
  during a test is inside the transaction; teardown rolls it back and the DB
  on disk is unchanged. Removing or gating this receiver breaks the harness
  silently — it will time out waiting for the transaction to be acknowledged
  in logcat.

- Popup logging: `GlobalPopupManager.show()` and `showError()` both write to
  logcat with tag `GlobalPopup`. The harness reads popup messages from there,
  because Compose snackbars render behind modal `Dialog` windows and are not
  visible in the a11y tree when a dialog is open. Do not remove these log
  calls.

Details in `AI_READ_THIS.md`, sections "End-to-end testing" and "Popup
severity".

## What changed again — 2026-10-19

A second batch of Rust-side renames and additions, on top of the ones
above. The Kotlin tree will not compile until these are fixed. The older
section still applies — this is additional.

### uncommitted_diffs is now client_love_letter

The table, the file, and both helpers are renamed. Nothing about the
columns changed.

  Old                                   New
  uncommitted_diffs (table)             client_love_letter
  tbl_uncommitted_diffs.rs              tbl_client_love_letter.rs
  newTableUncommittedDiffs()            newTableClientLoveLetter()
  newUncommittedDiffRow(...)            newClientLoveLetterRow(...)

In the builder crate, `UncommitedDiffsInsertCtx` became
`ClientLoveLetterInsertCtx`, and the field that carried it is now named
`client_love_letter` on `EverythingToInsertForNewPage`,
`EverythingForEditBlock`, `EverythingForDisablePage` and the new
`EverythingForAddBlock` / `EverythingForDeleteBlock`.

### newEveryBlockInExistenceRow takes an optional position

The `position` argument changed from `f64` to `Option<f64>`.
`None` means "I don't know the position yet — insert a placeholder and
update the row afterwards". The executor does exactly that for
add_block: it inserts with `None`, reads the new row's `id` back, then
sets `position` to that id as a float.

  newEveryBlockInExistenceRow(..., position: Double?)
  newEveryBlockInExistenceRow(..., position: Option<Double>)

Kotlin sees it as a nullable Double.

### RemoveBlock love letter carries a block id, not a position

    LoveLetterSketch::RemoveBlock { block_id: String, target_page_id: String }
    LoveLetter::RemoveBlock       { block_id: String, target_page_id: String, snapshot_of_edit: Vec<u8> }

`position: u32` is gone. A CRDT position is fragile under concurrent
edits; a block id is not. The server ignores this field either way.

### New builder exports: add_block and delete_block

Both are exported through uniffi and return `BuilderErr` on failure, the
same as create_page / edit_block / disable_page.

  add_block(yrs, block_content, block_meta_data, position,
            is_part_of_main_menu_page, session_id)
    -> EverythingForAddBlock { pages_update,
                               every_block_in_existence_insert,
                               client_love_letter }

  delete_block(yrs, block_id, session_id)
    -> EverythingForDeleteBlock { pages_update, block_id,
                                  client_love_letter }

The Rust executors for them live in the executor crate as
`add_block_requires_four_db_actions` and
`delete_block_requires_three_db_actions`. Those are wasm-only; Android
writes its own executor in Kotlin, mirroring the behaviour.

add_block's executor does four things in one transaction: update
pages.blobbed_page, insert the block row, set that row's position to its
own id as a float, insert the client_love_letter row.

delete_block's executor does three: update pages.blobbed_page, delete
the every_block_in_existence row matching `my_id_as_given_by_yrs`,
insert the client_love_letter row.

### z_db gained a read method (additive, nothing broke)

`get_data_ordered_and_skip_first_x` takes a table name, select
arguments, columns to read, an order_by, and a `skip_first_x: u64`, and
returns the same rows shape as get_data_ordered. It reads everything
after the skip; there is no cap. Nothing existing changed.

## Still open — 2026-10-19

`add_block` and `delete_block` are built on the Rust side as builder functions and uniffi exports. Android can call them once it writes its own executors. Neither is wired to any UI on web either.

Moving a block has no Rust implementation yet — no yrs move method, no love_letter variant, no builder, no executor. Android should not build one either, because there is no reference to mirror yet.

`new_every_block_in_existence_row` takes `position: Option<f64>`. `None` means "insert a placeholder and update the row after insert" — that is what add_block's executor does. Android's executor should follow the same shape.

The db-side ordering scheme is still unresolved. `position` currently holds the row's own id as a float, which is insertion order, not sort order. A real reorder cannot be expressed with it. Whatever Android does here has to match whatever web lands on.
