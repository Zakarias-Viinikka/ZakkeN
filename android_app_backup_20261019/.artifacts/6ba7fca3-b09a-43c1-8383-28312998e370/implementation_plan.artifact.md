# Implementation Plan - Rust Fixes & Logging Lab

Fix the breaking changes introduced by the updated Rust libraries and implement a modular logging system with isolated `ViewPage` helpers.

## User Review Required

> [!IMPORTANT]
> The `LoveLetterSketch.EditBlock` now requires a `blockId`. I will update all call sites in `DocEvents` to pass this ID.

> [!NOTE]
> For the logging system, I'll use a `CoroutineScope` with `Dispatchers.IO` to handle background logging, which is more idiomatic in modern Android/Kotlin than manually spawning `Thread` objects, but it achieves the same "off-main-thread" goal.

## Proposed Changes

### [Rust Fixes]

#### [MODIFY] [EditTextInBlock.kt](file:///home/zakke/ProgStuff/ZakkeN/android_app/app/src/main/java/z/zndroid/DocEvents/EditTextInBlock.kt)
- Update `LoveLetterSketch.EditBlock` instantiation to include the `blockId`.

### [Logging System]

#### [NEW] [ZLog.kt](file:///home/zakke/ProgStuff/ZakkeN/android_app/app/src/main/java/z/zndroid/log/ZLog.kt)
- Implement a thread-safe logging utility.
- Add `i`, `w`, `e` methods.
- Use a background scope to write logs (mocking persistence for now or writing to a local file).

### [ViewPage Refactor]

#### [NEW] [ViewPageHelper.kt](file:///home/zakke/ProgStuff/ZakkeN/android_app/app/src/main/java/z/zndroid/MainPages/ViewPage/helpers/ViewPageHelper.kt)
- Extract state mapping and "Ensure Title" logic from `ViewPage.kt`.

#### [MODIFY] [ViewPage.kt](file:///home/zakke/ProgStuff/ZakkeN/android_app/app/src/main/java/z/zndroid/MainPages/ViewPage/ViewPage.kt)
- Use the new helpers.
- Integrate `ZLog` for initialization and error tracking.

## Verification Plan

### Automated Tests
- Run existing Integrity Suite (`./gradlew test`) to ensure CRDT/SQLite sync isn't broken by the library update.

### Manual Verification
- Deploy to device/emulator.
- Verify that editing text still works (checking if the `blockId` fix in `LoveLetterSketch` is correct).
- Check Logcat for `ZLog` outputs from `ViewPage`.
