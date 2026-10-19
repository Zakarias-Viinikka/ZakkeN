# Fix Database Latency with Background Threading

The database operations are currently running on the Main (UI) thread, which causes the app to "freeze" or stutter during queries. I will implement a utility to easily offload these tasks to a background thread and update the UI to handle asynchronous loading.

## User Review Required

> [!IMPORTANT]
> **DbManager.init** will now be asynchronous. This means the database might not be ready the exact millisecond the first screen appears. I will add a check to ensure we don't try to use the database before it's ready.

## Proposed Changes

### Core Utilities

#### [NEW] [ThreadUtils.kt](file:///home/zakke/ProgStuff/ZakkeN/android_app/app/src/main/java/z/zndroid/ThreadUtils.kt)
Create a simple utility file containing the requested `doInBackground` (or `io`) helper to wrap database calls in `Dispatchers.IO`.

### Database Management

#### [MODIFY] [DbManager.kt](file:///home/zakke/ProgStuff/ZakkeN/android_app/app/src/main/java/z/zndroid/DbManager.kt)
- Make `init` a `suspend` function or use a background scope.
- Add a `isReady` state or ensure initialization happens off-thread.

#### [MODIFY] [MainApplication.kt](file:///home/zakke/ProgStuff/ZakkeN/android_app/app/src/main/java/z/zndroid/MainApplication.kt)
- Launch the database initialization in the background so it doesn't block app startup.

### User Interface

#### [MODIFY] [DbGui.kt](file:///home/zakke/ProgStuff/ZakkeN/android_app/app/src/main/java/z/zndroid/DbGui.kt)
- Move `listTables()` call to a background thread.
- Add a loading spinner while the table list is being fetched.

#### [MODIFY] [InspectTable.kt](file:///home/zakke/ProgStuff/ZakkeN/android_app/app/src/main/java/z/zndroid/InspectTable.kt)
- Move `refreshData` and `insertData` calls to background threads.
- Add a `CircularProgressIndicator` during data loading.

## Verification Plan

### Manual Verification
1. Open the app and verify it starts up without a long white screen.
2. Navigate to the Database GUI and verify the table list appears smoothly.
3. Click a table and verify a loading spinner appears briefly before the data is shown.
4. Insert a row and verify the UI doesn't "hang" while the operation completes.
