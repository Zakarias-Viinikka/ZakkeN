# Getting started

You have zero context on this codebase. This file tells you which files matter for the task in front of you. Read only what you need.

## One paragraph

Android app for note-taking over yrs CRDT pages. Rust crates behind uniffi (SQLite mirror + yrs docs + websocket + diffing), Kotlin Compose UI on top. Local-first with server sync. Every block is a CodeMirror 6 editor in its own WebView today (2026-11-25).

## Read in this order

1. `AI_READ_THIS.md` — reference for the whole codebase. Skim. Come back to it as a lookup.
2. `MISSING_AND_STALE.md` — known gaps. Especially before touching migrations or sync. The "2026-11-25 CM6 editor: bugs found on device" section is not fixed yet.
3. `INVENTED_ON_ANDROID.md` — every place Android intentionally differs from web. You will hit one of these and try to "fix" it otherwise.

## Task → relevant files

"Entry" = read these first. "Touch" = these need edits for that task area. "Careful" = there is a trap, see the referenced doc entry.

### Editor: typing, Enter/Backspace/arrow behavior, block text

Entry: `editor/PageSession.kt`, `MainPages/ViewPage/EditableBlock.kt`, `markdown_conversion/keys.kt`.

Touch: `markdown_conversion/keys.kt` (editing rules), `markdown_conversion/edit.kt` (`applyChanges`), `MainPages/ViewPage/EditableBlock.kt` (dispatch and `handleKeyIntent`).

Careful: `markdown_conversion/cm.kt` talks to JS. Only relevant if you're changing the bridge. `editor/PageBatcher.kt` is below everything; read it before touching a flush path. The device-found bugs about fast split/merge and caret twitch are in `MISSING_AND_STALE.md` under 2026-11-25.

### Editor: markdown rendering, wikilinks, decorations

Entry: `assets/markdown_web_output/cm_bridge.js` (web-built, do not edit — copy from web), `assets/markdown_web_output/editor_host.html`.

Touch: `editor_host.html` only. The JS bridge is regenerated from `cqrs_state_synchronization/web_interface/markdown_web_output/cm_bridge.js`. Behavior changes go in the web markdown_js source and rebuild, not in this repo's copy.

### Link clicks / page navigation from a wikilink

Entry: `markdown_conversion/main_for_on_link_click/core.kt` (`mainOnLinkClick`), `MainActivity.kt` (the `view_page` route wires `onOpenPage`).

Touch: `main_for_on_link_click/<command>.kt` for new commands; one line in `core.kt`'s `when`.

Careful: `page_nav.kt` is the only real command; the others are placeholders. See `INVENTED_ON_ANDROID.md`.

### Special menu (double-tap overlay)

Entry: `MainPages/special_menu/core.kt` (`SpecialMenu`, `interceptDoubleTap`), `LinkToExistingPage.kt`, `internally_shared.kt`.

Wiring: `MainPages/ViewPage/ViewPage.kt` (double-tap + overlay state), `EditableBlock.kt` (`onFocused` callback reports "this block is focused"). Insert uses `markdown_conversion/cm.kt::Cm.insertAtCursor`.

Careful: `searchPageTitles` in `db/reads/Reads.kt` is the fallback-only search. No FTS5 on Android. See `MISSING_AND_STALE.md`.

### Schema, migrations, table descriptors

Entry: `db/migrations/SchemaVersion.kt`, `db/schema/TableDescriptors.kt`, `schema_drift_detector/src/registry.rs`.

Touch: new `db/migrations/schemas/VersionN.kt` when adding a version; `SchemaVersion.kt` for the enum + bootstrap + `updateOnce`; `TableDescriptors.kt` for column readers.

Careful: the `schema_drift_detector` runs as `preBuild` and in `build_and_install.sh`. It compares Rust blueprints, `Version<N>.kt`, and `TableDescriptors.kt`. Break one, the build fails with a specific table/column name. Read `MISSING_AND_STALE.md` "backlinks CRDT table removed" before touching table sets. Never edit `TableDescriptors.kt` in isolation — the detector will catch the divergence.

### Sync: login, send, receive, apply, reconcile

Entry: `sync/owner_of_interacting_with_server/Core.kt`, `ConnState.kt`, `SendLoop.kt`, `ReceiveLoop.kt`, `ApplyLetter.kt`, `Reconcile.kt`, `Recovery.kt`, `SeenLetters.kt`, `SpontaneousInbox.kt`.

Touch: one file per behavior, mirroring web's `cqrs_state_synchronization/web_interface/src/owner_of_interacting_with_server/`.

Careful: `ConnState.NeedsReconcile` is planned (import will set it) but not added yet. `syncMayRun()` reads `Connected` only today. See `MISSING_AND_STALE.md` for other sync gaps. Do not touch Rust crates from this folder — server protocol is shared.

### Sync recovery screen

Entry: `MainPages/RecoveryPage.kt`, `sync/owner_of_interacting_with_server/Recovery.kt`.

Careful: not covered by e2e. Fixing a recovery bug means testing it manually on the phone.

### Page lifecycle: create / rename / disable

Entry: `actions/PageActions.kt`, `actions/MoveBlock.kt`, `actions/Kick.kt`.

Touch: `PageActions.kt` for page-level actions. `executor/` mirrors the Rust builder and table writes — do not edit unless the builder changed.

Careful: `createNewPage` emits two love letters (CreateNewPage + EditBlock); web emits one. Known, not a bug.

### Nav / main menu

Entry: `MainActivity.kt` (routes), `MainPages/NavPage.kt`, `NavRow.kt`.

Careful: the eye button on a nav row triggers a 1.5s drop animation then `disablePage`. Not immediate. See `INVENTED_ON_ANDROID.md`.

### DB reads

Entry: `db/reads/Reads.kt`.

Touch: add a function here, use `ColumnDescriptor` unwrap through `colOf`. Never read columns by position in new code (there are existing positional reads; see `MISSING_AND_STALE.md`).

### Theme / colors

Entry: `ui/theme/Theme.kt`, `FeelColors.kt`.

Careful: always dark currently. `LocalFeelColors.current` is the palette. `LocalReducedMotion.current` gates animation plays. Do not use Material colors directly in new UI — use `LocalFeelColors`.

### Build / native libs / update.sh

Entry: `build_and_install.sh`, `update.sh`, `../build_all_android.sh`.

Careful: `update.sh` overwrites generated Kotlin under `rustlib/` and `com/z_db/`. Never edit those directly. Rust crates are outside this repo; build them separately. See `OUTSIDE_ANDROID_APP.md`.

### Dev tools / tests

Entry: `Tests/DevToolsPage.kt`, `Tests/DevCheckRegistry.kt`, `Tests/OpsChecks.kt`.

Careful: `./gradlew test` hangs on this PC — the JVM host tests need x86_64 libs that this project does not ship. Use `./gradlew :app:compileDebugKotlin` for compile-only checks and `./build_and_install.sh` for the real gate. See `TESTING_ON_PC.md`.

### Lab (experimental)

Entry: `lab/`, `android_labs/` (outside this repo). Not mounted in the app. See `AI_LAB_RULE.md`.

## Conventions

- Errors carry three fields (what went wrong, method, file). Popups go through `GlobalPopupManager.showError`. Never swallow silently.
- Every commit that changes a schema bumps a version — no in-place column edits.
- `WriteLock` is not re-entrant. Do not take it inside a section that already holds it.
- Actions take `WriteLock` then one `DbManager.withTransaction`, nothing else.
- Read the file before editing. Verify what is there — do not guess at anchors.

## Known open bugs (2026-11-25)

Two device-only bugs on the CM6 editor, no web equivalent, documented with reproduction and investigation steps in `MISSING_AND_STALE.md`:
- "save failed couldn't remove 3 items from an array" during fast split or merge.
- Caret jumps to the wrong location for a split second after a split.

Neither is fixed. Do not assume the editor is stable on a phone.
