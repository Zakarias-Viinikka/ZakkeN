# Migrate Database Functionality from DbTesting to Zndroid

This plan outlines the steps to copy the Rust-backed database functionality and UI from the `DbTesting` project to the `Zndroid` project. All components will be refactored to use the `z.zndroid` package, and the `DbGui` will become the primary entry point.

## Proposed Changes

### Build Configuration

#### [MODIFY] [libs.versions.toml](file:///home/zakke/ProgStuff/ZakkeN/android_app/gradle/libs.versions.toml)
- Add `androidx-navigation-compose` version and library definition.
- Add `jna` version and library definition.

#### [MODIFY] [build.gradle.kts](file:///home/zakke/ProgStuff/ZakkeN/android_app/app/build.gradle.kts)
- Add Navigation and JNA dependencies.

### Native Infrastructure

#### [NEW] [jniLibs/arm64-v8a/libdb_wrapper.so](file:///home/zakke/ProgStuff/ZakkeN/android_app/app/src/main/jniLibs/arm64-v8a/libdb_wrapper.so)
#### [NEW] [jniLibs/arm64-v8a/libprotocol.so](file:///home/zakke/ProgStuff/ZakkeN/android_app/app/src/main/jniLibs/arm64-v8a/libprotocol.so)
#### [NEW] [com/z_db/android_mascot/db_wrapper.kt](file:///home/zakke/ProgStuff/ZakkeN/android_app/app/src/main/java/com/z_db/android_mascot/db_wrapper.kt)
#### [NEW] [uniffi/protocol/protocol.kt](file:///home/zakke/ProgStuff/ZakkeN/android_app/app/src/main/java/uniffi/protocol/protocol.kt)

### Application Core & Schema

#### [NEW] [MainApplication.kt](file:///home/zakke/ProgStuff/ZakkeN/android_app/app/src/main/java/z/zndroid/MainApplication.kt)
- Custom Application class to initialize `DbManager`.

#### [NEW] [DbManager.kt](file:///home/zakke/ProgStuff/ZakkeN/android_app/app/src/main/java/z/zndroid/DbManager.kt)
- Singleton for database lifecycle management. Refactored to `z.zndroid`.

#### [NEW] [TestTable.kt](file:///home/zakke/ProgStuff/ZakkeN/android_app/app/src/main/java/z/zndroid/schema/TestTable.kt)
- Initial database schema definition. Refactored to `z.zndroid.schema`.

### User Interface

#### [NEW] [DbGui.kt](file:///home/zakke/ProgStuff/ZakkeN/android_app/app/src/main/java/z/zndroid/DbGui.kt)
- Main database dashboard screen. Refactored to `z.zndroid`.

#### [NEW] [InspectTable.kt](file:///home/zakke/ProgStuff/ZakkeN/android_app/app/src/main/java/z/zndroid/InspectTable.kt)
- Screen for viewing and inserting data into specific tables. Refactored to `z.zndroid`.

#### [MODIFY] [MainActivity.kt](file:///home/zakke/ProgStuff/ZakkeN/android_app/app/src/main/java/z/zndroid/MainActivity.kt)
- Replace "Greeting" boilerplate with `NavHost` pointing to `DbGui` and `InspectTable`.

### Manifest & Maintenance

#### [MODIFY] [AndroidManifest.xml](file:///home/zakke/ProgStuff/ZakkeN/android_app/app/src/main/xml/AndroidManifest.xml)
- Set `android:name=".MainApplication"` in the `<application>` tag.

#### [NEW] [update.sh](file:///home/zakke/ProgStuff/ZakkeN/android_app/update.sh)
- Maintenance script updated for the `Zndroid` project paths.

## Verification Plan

### Automated Tests
- Run `./gradlew assembleDebug` to verify compilation and dependency resolution.

### Manual Verification
- Deploy to an arm64-v8a device/emulator.
- Verify `DbGui` loads the `test_table`.
- Insert a record into `test_table` and verify it appears in the list.
- Delete a record and verify it is removed.
- Run `update.sh` to ensure it correctly downloads and places files.
