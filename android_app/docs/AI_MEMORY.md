# AI Memory - Zndroid Project

## Architecture Highlights
- **Authoritative Source**: Yrs CRDT blobs in the `pages` table (`blobbed_page` column).
- **Queryable Source**: SQLite table `every_block_in_existence`.
- **Sync Strategy**: Intent-based. `uncommitted_diffs` stores `LoveLetterSketch` (serialized intent) + Yrs diff snapshot.
- **Dual-Layer Storage**: `StorageAccess` uses `FastStorage` (RAM) first for whitelisted keys, then falls back to SQLite.
- **Buffered Batching**: Edits are collected in `BlockUiState.diffBuffer` and "squashed" using `text_diff.combineGetdiffResults` before persistence (500ms debounce).
- **Editing Logic**:
  - **Split**: `Enter` (without Shift) splits a block at the cursor. Text after moves to a new block.
  - **Merge**: `Backspace` at position 0 merges current block into previous one. Structural changes automatically cancel pending debounce flushes to prevent race conditions.
  - **Re-use**: `ViewPage` reuses `BlockUiState` to preserve cursor/selection during UI refreshes.
- **AI Lab**: Experimental UX testing ground in `lab/` package.
  - **Modular**: Experiments in `lab/experiments/` (e.g., `scroll`, `selection`, `undoredo`).
  - **Core**: `LabNavigator` handles routing; `LabContainer` wraps the app for overlays.
  - **Rules**: Governed by `docs/AI_LAB_RULE.md`.
- **Logging**: Modular system in `log/` package.
  - **Engine**: `Logger.kt` processes logs in background `CoroutineScope`.
  - **Helpers**: `ViewPageLogs.kt` for feature events; `LogDateHelpers.kt` for Unix conversions.
- **Schema**:
  - `every_block_in_existence` uses `is_title` ("true"/"false") boolean flag.
  - `LoveLetterSketch.EditBlock` requires `blockId`.

## AI Documentation Index
All architectural notes and rules are located in the `docs/` folder:
- [AI_MEMORY.md](file:///home/zakke/ProgStuff/ZakkeN/android_app/docs/AI_MEMORY.md): (This file) Architecture highlights and learnings.
- [AI_READ_THIS.md](file:///home/zakke/ProgStuff/ZakkeN/android_app/docs/AI_READ_THIS.md): Library guide and API reference.
- [AI_LAB_RULE.md](file:///home/zakke/ProgStuff/ZakkeN/android_app/docs/AI_LAB_RULE.md): Guidelines for the Lab.
- [UX_INTENT.md](file:///home/zakke/ProgStuff/ZakkeN/android_app/docs/UX_INTENT.md): Philosophy and behavior of the editor UX.
- [FORME.txt](file:///home/zakke/ProgStuff/ZakkeN/android_app/docs/FORME.txt): Personal notes.
