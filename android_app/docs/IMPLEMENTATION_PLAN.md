# Implementation Plan

Read docs/OUTSIDE_ANDROID_APP.md first. It explains why each of these
is needed and what the outside world looks like. This file is only the
order of work.

## Before you start

Any step that requires the human's approval - running a chain that
needs review, running a binary, anything gated - must be requested
BEFORE you start work, all in one batch. The human goes AFK. If you
request approval in the middle of work, you stall. Get every approval
you need up front, then do the rest without asking.

If you need to run the Android build or tests, do it yourself. The
project's build system is available to you.

If you need the human to make a decision that isn't covered by these
docs, ask for the decision up front too, before starting work.

## The order

Each step builds on the previous. Do them in order. After each step,
the project should still compile.

### 1. Rename the column-list functions

Mechanical. Every call to pagesColumns, backlinksColumns,
everyBlockInExistenceColumns, uncommittedDiffsColumns,
keyValueStorageColumns becomes the newTableX form.

Files affected:
  DbManager.kt
  Storage/StorageAccess.kt
  Storage/StorageInitializer.kt
  db/migrations/MigrationManager.kt
  db/migrations/SchemaVersion.kt
  SchemaGui.kt
  DocEvents/AddBlock.kt
  DocEvents/EditTextInBlock.kt
  DocEvents/RemoveBlock.kt
  Tests/SchemaSyncTest.kt
  Tests/TransactionTest.kt
  any Lab file that imports them

### 2. Fix the row builder calls

Everywhere that calls newPageRow with three arguments, update to four.
newEveryBlockInExistenceRow needs the new is_part_of_main_menu_page
argument. Check the exact signatures in
OUTSIDE_ANDROID_APP.md.

### 3. Fix StorageAccess.setValue

It currently zips the two-column result of newKeyValueItem against the
three-column schema (id, key, value). Indexes are off; the value never
lands in the right column. Compare with StorageInitializer which gets
it right using SafeRowMapper with skipId = false. Make setValue do the
same.

### 4. Schema drift canary tests

Build the parallel column structure: one Kotlin object per table, each
holding a list of column descriptors with (name, type, unwrap). Group
them per table.

Then a canary test that asserts each descriptor's type is what you
expect. The assertions are hardcoded. The test does not call Rust. If a
column type changes upstream, the assertion fails; the error message
and the comment say what else needs to change.

This is the Android substitute for Rust's generic SchemaColumn<T>. It
is temporary - a real drift-detection tool is planned.

Also rebuild ExpectedSchema (or replace it) so it reflects the current
live schema. It's missing columns on every_block_in_existence and
missing the logs table entirely.

### 5. Migration V1

Freeze a new version matching the current live Rust blueprints. That
means every column of every table, plus foreign keys, in
db/migrations/schemas/Version1.kt.

Add a migration from V0 to V1 that alters tables to match. The changes
to apply:

  every_block_in_existence gains is_part_of_main_menu_page (Text,
  not null) and page_is_disabled (Text, not null).

  key_value_storage gains id (INTEGER, primary key, autoincrement).
  The old table had key as primary key. Migrating means recreating the
  table, moving rows, then dropping the old.

  logs is created (new table).

  incoming_love_letters is created (new table) but is not used yet.

Bump SchemaVersion.LATEST to V1.

Update assertLatestMatchesBlueprints to check against V1 and to include
the new tables.

Leave Version0.kt alone. It is a historical snapshot.

### 6. Squash-first rule

Every action that is not a plain text keystroke must flush the buffer
first, and the flush must complete before the action runs.

Current state:
  splitBlock - already calls flushBuffer first. But it calls
  debounceJob.cancel() before, and cancelling may not stop an in-flight
  flush. Also splitBlock does the flush and then EditTextInBlock.execute
  as separate operations; if the second fails, the split is
  half-applied. Make the flush and the split one transaction.

  mergeWithPreviousBlock - does state.diffBuffer.clear(), which
  discards pending edits. Replace with a flush.

  The Backspace handler in EditableBlock calls the same merge function,
  so fixing the function fixes both.

  Add flush-before to: tapping another block, deleting a block, any
  other structural action.

### 7. Fix syncUiStates clobbering pending edits

ViewPageHelper.syncUiStates copies b.text over existing.textFieldValue
whenever they differ. If the block has pending buffered diffs, the boss
hasn't seen them yet, so the UI text is reverted.

The fix: skip the sync when the block has a non-empty diffBuffer.

### 8. Nav menu title

NavPage currently displays page_id as the title. Change it to read the
title block's content from every_block_in_existence where is_title =
"true" and id_of_page_i_belong_to = the page's id.

If the title block is empty, show "Untitled" in a muted colour.

Rename-from-menu currently changes page_id. Change it to edit the title
block's content. It must go through the builder/executor path.

### 9. Logging to db

Logger.kt currently only writes to Logcat. Extend it to also insert into
the logs table.

The shape:
  Outside a transaction: insert directly.
  Inside a transaction: buffer in memory.
    On commit: drop the buffer.
    On rollback: flush the buffer to the logs table (outside the
    failed transaction).

Every call site passes source explicitly (file/class/function). Per-
feature helpers (ViewPageLogs and future ones) fill it in. Do not use
stack-trace introspection - R8 obfuscation breaks it.

### 10. DbManager.renamePage

Rename it and reimplement. What it does today - changing page_id - is
wrong. It should call the builder to edit the title block's content.

### 11. Update references to renamed things

After all the above, scan for stale imports and old function names
throughout. Fix each.

### 12. The Lab

Check whether any Lab experiment imports from production and uses
something that was renamed. If yes, fix the Lab's copy. Do not change
production to accommodate the Lab.

### 13. Docs

Update:
  docs/AI_READ_THIS.md - reflect the current state of the library guide
  docs/SHARED_MEMORY.md - reflect the current architecture
  docs/INVENTED_ON_ANDROID.md - create it if it doesn't exist, list every
    decision you made that has no Rust or web counterpart

INVENTED_ON_ANDROID.md is important. Anything you designed without a
reference - structural ops, the shape of the canary tests, anything
in the logging buffer - goes in there. Each entry says what Android
does and where in web/Rust the equivalent should eventually live so it
can be verified when the web side catches up.

## Things to verify before declaring done

Every one of these is a check you run yourself. Do not ask the human.

  Search the whole Kotlin tree for the old function names. Zero
  results.

  Search for any reference to page_id as a title. None should exist.

  Search for direct calls to diffBuffer.clear(). None should exist.

  Every structural action flushes first. Read each one.

  Every newTableX and newXRow call matches the current Rust signature.

  The canary tests run and pass.

  assertLatestMatchesBlueprints passes.

  The Lab compiles.

## What NOT to do

Do not change any Rust crate. Do not change the web project. Do not
edit any file outside android_app/.

Do not redesign the architecture. This is a repair job, not a rewrite.
The bugs are known; the plan is known. Follow it.

If you find something that's broken but not mentioned here, log it in
INVENTED_ON_ANDROID.md and move on. Do not fix it - the plan is the
plan.
