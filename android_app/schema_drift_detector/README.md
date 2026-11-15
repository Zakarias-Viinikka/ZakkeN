# schema_drift_detector

Host-side tool that fails the Android build when the Rust schema (in
`client_table_blueprints`) and the Kotlin side (frozen snapshot +
`TableDescriptors.kt`) disagree. The exact failure names the table and
column — `logs.details_type: added in Rust`, `every_block_in_existence.page_is_disabled:
type changed. Frozen='TEXT' Rust='INTEGER'`, and so on.

## What it checks

A. Frozen `Version<N>.kt` (N from `SchemaVersion.LATEST`) vs the live Rust
   blueprints. Full fidelity: tables, columns, per-column flags, column
   order, foreign keys, FK order.
B. `TableDescriptors.kt` vs live Rust: table coverage both ways, column
   names / types / order. Prints a pasteable `ColumnDescriptor` line for
   each column the Kotlin side is missing.
C. Version plumbing: `LATEST` parses and its bootstrap body references
   `Version<N>.tables` (catches "bumped LATEST, forgot to repoint bootstrap").
D. Native lib freshness: `client_table_blueprints/android_output/.../libclient_table_blueprints.so`
   must be newer than its sources; the app's copy must match it.
E. Rust coverage: every `pub fn new_table_*` / `pub fn get_foreign_def_*` in
   `client_table_blueprints/src/` has an entry in `src/registry.rs`.

## How to read a report

Every line names the table, the column, and what changed. Groups are
labelled `A.`, `B.` etc. Each group ends with a `Fix:` line explaining
the workflow step.

## How to change the schema (the workflow)

1. Edit `client_table_blueprints/src/` as needed.
2. `android_app/schema_drift_detector/check.sh emit-snapshot <N+1>` — prints
   the new `Version<N+1>.kt`.
3. Save as `app/src/main/java/z/zndroid/db/migrations/schemas/Version<N+1>.kt`.
4. In `SchemaVersion.kt`, add a `V<N+1>(<N+1>)` entry whose bootstrap goes
   through `Version<N+1>.tables`, write the `V<N>.updateOnce` that
   migrates old DBs to V<N+1>, and set `LATEST = V<N+1>`.
5. Update `TableDescriptors.kt` per the check-B list.
6. `ZakkeN/build_all_android.sh` — rebuild native libs, copy them in, then
   the tool confirms clean.

## How to add a table

1. Add the Rust blueprint (`tbl_<name>.rs` with `new_table_<name>()`).
2. Add it to `src/registry.rs` in creation order.
3. Run `check.sh` — the report will say the Kotlin side is missing it and
   list exactly what to add (a `FrozenTable(...)` entry plus a descriptor
   object).

## Flags

- `check` (default): run all checks, exit 0 clean / 1 drift / 2 tool error.
- `emit-snapshot <N>`: print a fresh `Version<N>.kt` from live Rust.
- `--warn-only`: print the report, always exit 0 (used by `update.sh`).
- `--no-stale-check`: skip check D.
- `--android-root <dir>` / `--blueprints-root <dir>`: override paths.

Env vars: `SKIP_SCHEMA_DRIFT=1` bypasses the tool with a loud warning.
`SCHEMA_DRIFT_CHECKED=1` is set by build scripts so the Gradle hook
doesn't re-run when it's already been run for the same build.

## Verification

- `cargo test` (in this directory): scanner unit tests, Kotlin parser unit tests, differ unit tests, plus `tests/mutation.rs` which runs the real binary against mutated copies of the real Kotlin files and asserts each mutation fires the right kind of drift. Also round-trips `emit-snapshot` output back through the parser.
- `./check.sh` at the repo root should report `clean`. If it doesn't, that's real drift in the tree, not a tool bug.

- Rust-side **table renames** are undetectable — table names aren't part
  of blueprint output. If you rename a table in Rust, add the new name to
  the registry by hand and remove the old.
- App code that spells a column name as a string literal (`"is_title"`)
  rather than using a descriptor's `.name` still compiles if the column
  is renamed. The `db/reads` package uses descriptors; other spots
  should too, when convenient.
- The tool doesn't simulate `MigrationManager.migrate`; a schema walk
  that reaches the wrong final state passes checks A-E. `MigrationTest`
  on the device covers that path.
