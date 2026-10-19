# Implementation Plan - Transaction Support in DbManager

The Rust database library (`z_db`) has been updated to support transactions via `begin_all_or_nothing`, `everything_went_perfectly`, and `regret_everything`. This plan details how to expose this functionality in the Kotlin `DbManager` while maintaining thread safety and avoiding deadlocks.

## User Review Required

> [!IMPORTANT]
> The implementation uses a `CoroutineContext` element to track transaction state. This allows `DbManager.executeNative` and other public API methods to be called safely from within a `transaction` block without causing a deadlock on the `dbMutex`.

## Proposed Changes

### [zndroid Component](file:///home/zakke/ProgStuff/ZakkeN/android_app/app/src/main/java/z/zndroid)

#### [MODIFY] [DbManager.kt](file:///home/zakke/ProgStuff/ZakkeN/android_app/app/src/main/java/z/zndroid/DbManager.kt)

- **Define `DbTransactionElement`**: A private class to store the `LiveForever` instance in the coroutine context during a transaction.
- **Refactor `executeNative`**:
    - Add a private `executeNativeInternal` that performs the actual block execution and exception handling.
    - Update `executeNative` to check for the presence of `DbTransactionElement` in the `coroutineContext`. If present, it bypasses the `dbMutex` to prevent deadlocks.
- **Add `withTransaction` method**:
    - Acquires `dbMutex`.
    - Calls `db.beginAllOrNothing()`.
    - Executes the provided block within a new coroutine context containing `DbTransactionElement`.
    - Calls `db.everythingWentPerfectly()` on success.
    - Calls `db.regretEverything()` on failure before re-throwing or returning the error.

#### [MODIFY] [StorageInitializer.kt](file:///home/zakke/ProgStuff/ZakkeN/android_app/app/src/main/java/z/zndroid/Storage/StorageInitializer.kt)

- No changes required, but it will now be safe to call from within a `DbManager.withTransaction` block.

---

## Verification Plan

### Automated Tests
- I will create a temporary scratch test to verify that:
    - Multiple operations in a transaction are atomic.
    - An exception inside a transaction triggers a rollback.
    - Nested calls to `DbManager` methods within a transaction do not deadlock.

### Manual Verification
- Verify the app still initializes correctly and storage tables are created as before.
- (Optional) Wrap `DbManager.init` logic in a transaction to ensure atomic database setup.
