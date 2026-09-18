# Goal: SQLite Schema Migrations via MigrationManager

Implement a self-contained single-step schema migration pattern in the Android DB layer using a walk-loop and the `key_value_storage` table to track versions.

## Proposed Changes

### 1. Database & Migrations Infrastructure

#### [NEW] [Version0.kt](file:///home/zakke/ProgStuff/ZakkeN/android_app/app/src/main/java/z/zndroid/db/migrations/schemas/Version0.kt)
Freeze the initial schema definitions so they never drift when `client_table_blueprints` changes.
- Define explicit `List<ColumnDef>` for `pages`, `uncommitted_diffs`, `backlinks`, `every_block_in_existence`, and `key_value_storage`.
- Define explicit `List<ForeignKeyDef>` for `backlinks` and `every_block_in_existence`.

#### [NEW] [SchemaVersion.kt](file:///home/zakke/ProgStuff/ZakkeN/android_app/app/src/main/java/z/zndroid/db/migrations/SchemaVersion.kt)
Define an enum representing known database schema versions.
- `VERSION_0`
- `val latest = VERSION_0`
- Implement an `updateOnce(db: LiveForever): SchemaVersion?` method that returns the next version after running DDL steps, or `null` if already at the latest.

#### [NEW] [MigrationManager.kt](file:///home/zakke/ProgStuff/ZakkeN/android_app/app/src/main/java/z/zndroid/db/migrations/MigrationManager.kt)
Handles the walk-loop and version checks.
- Read `schema_version` from `key_value_storage`.
- If missing entirely (fresh DB), bootstrap the database with `Version0` tables and set version to `VERSION_0`.
- If present but higher than the highest known version enum, throw a `LocalDbError.DatabaseDowngraded` exception.
- Otherwise, execute a `while` loop within a single database transaction, walking up by running `updateOnce` until the latest version is reached.
- Write the final updated version back to `key_value_storage`.

### 2. Wiring up the Database Layer

#### [MODIFY] [DbManager.kt](file:///home/zakke/ProgStuff/ZakkeN/android_app/app/src/main/java/z/zndroid/DbManager.kt)
- Integrate `MigrationManager.migrate(newDb)` right inside `init(context)`, immediately after creating the `LiveForever` instance.
- Remove the hardcoded table creation blocks from `DbManager.init` since the migration manager will own creating/upgrading tables.

### 3. Verification & Round-Trip Testing

#### [NEW] [MigrationTest.kt](file:///home/zakke/ProgStuff/ZakkeN/android_app/app/src/main/java/z/zndroid/Tests/MigrationTest.kt)
Add a test suite verifying the round-trip:
- Verify that a fresh DB bootstraps correctly to `Version0` tables.
- Verify version tracking writes/reads to `key_value_storage`.
- Register the test in `RegisterTests.allTests`.

#### [MODIFY] [TransactionTest.kt](file:///home/zakke/ProgStuff/ZakkeN/android_app/app/src/main/java/z/zndroid/Tests/TransactionTest.kt)
- Revert the diagnostic logs injected earlier to clean up the code.

---

## Verification Plan

### Automated Tests
- Run Gradle unit tests to ensure no syntax errors or breaking regressions.
- Execute the app tests via the internal test wrapper to verify the walk-loop behaves properly.
