# AI Memory - Zndroid Project

## Architecture Highlights
- **Authoritative Source**: Yrs CRDT blobs in the `pages` table (`blobbed_page` column).
- **Queryable Source**: SQLite table `every_block_in_existence`.
- **Sync Strategy**: Intent-based. `uncommitted_diffs` stores `LoveLetterSketch` (serialized intent) + Yrs diff snapshot.
- **Dual-Layer Storage**: `StorageAccess` uses `FastStorage` (RAM) first for whitelisted keys, then falls back to SQLite.

## Project Structure
- `z.zndroid.DocEvents`: Atomic operations like `AddBlock`, `EditTextInBlock`, `RemoveBlock`.
- `z.zndroid.MainPages.ViewPage`: Live editor using Compose state "signals" (`BlockUiState`) for each block.
- `z.zndroid.Tests`: Enforced Integrity Suite. If tests in `RegisterTests` fail, the Gradle build fails.

## Recent Learnings
- **Triple Enter**: Triggers `ManyEntersLeadsToManyBlocks` to split a block.
- **Delete if Empty**: Backspace in an empty block triggers `RemoveBlock`.
- **Session IDs**: Managed by `SessionManager`, unique per launch/navigation, stored in `FastStorage`.
- **Schema Drift**: Prevented by `SchemaSyncTest` comparing `ExpectedSchema` vs Rust blueprints vs live DB.
