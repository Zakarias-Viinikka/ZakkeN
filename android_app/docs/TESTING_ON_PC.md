# Testing on the PC

Status: TODO. Not part of the 2026-11-25 CM6 editor port. Recorded so the
2026-11-25 decision ("out of scope for this work") is not lost.

## Two different things people mean by "simulate Android on the PC"

**JVM unit tests.** `./gradlew test` runs `app/src/test/` on the PC's own JVM
(x86_64 Linux here). The tests import the uniffi-generated Kotlin, which
calls `Native.register(...)` and asks the JVM for arm64-v8a `.so` files in
`app/src/main/jniLibs/arm64-v8a/`. The PC's JVM cannot load arm64 code, so
anything that touches the DB or yrs fails or hangs.

**Android emulator.** An x86_64-linux-android image. Also needs x86_64
`.so` files, but built against the NDK, not the PC's glibc. Different target
triple from the JVM case above, often confused, not the same problem.

A 2026-11-25 note says: "all the android builds need to be changed to do the
actual build for the tester thingy too". Reads as both. Verify before
starting work.

## Current state

`update.sh` copies only `arm64-v8a` from each crate's `android_output` (or
`android_output_folder`; the output dir varies per crate). The generated
Kotlin calls `System.loadLibrary("...")` through JNA, which asks
`java.library.path` for the name. Nothing in `app/build.gradle.kts` touches
`java.library.path` today.

`build_and_install.sh` refuses an emulator unless `ANDROID_SERIAL` names one,
then installs an arm64-only APK that would die at `MainApplication.onCreate`
because the uniffi `.so` files cannot load on an x86_64 image.

`./gradlew test` hangs at `:app:testDebugUnitTest > Executing test
z.zndroid.Tests.BuildVerificationTest` for many minutes ("1 test completed"
never advances). At least one test suspends forever on `DbManager.awaitReady()`,
which is a `CompletableDeferred` completed only by `DbManager.init(context)`
inside `MainApplication.onCreate`. That never runs on a JVM test.

## To get JVM unit tests passing

1. Build every crate for the PC's native triple (`x86_64-unknown-linux-gnu`),
   additively to its existing Android build script. Crates: `yrs`,
   `love_letter`, `client_table_blueprints`, `text_diff`, `smart_frac_index`,
   `data_builder_for_operations_that_need_to_be_correct`, `server_link`, plus
   `z_db` (`protocol` and `db_wrapper`).
2. Load order: JNA's `Native.register` uses `findLibraryName` which returns a
   bare name like `"my_yrs_lib"`, then lets `java.library.path` resolve it.
   Set `testOptions.unitTests.all { it.systemProperty("java.library.path",
   "<dir>") }` in `app/build.gradle.kts`, OR set `jna.library.path` (JNA
   prefers it), OR write the `.so` into `build/` and configure the test JVM to
   search there. Pick one, keep it in Gradle so no shell env is needed.
3. `DbManager.awaitReady()` needs a test path. Options in order of least
   invasive: (a) `DbManager.init` takes a DB path instead of a Context, and
   each DB-using test calls it with a temp path. (b) Robolectric, then
   `@Before` calls `DbManager.init(RuntimeEnvironment.getApplication())`.
   (c) A TestRule that pre-populates `readyDeferred`.

## To get the app running on the emulator

1. Build every Rust crate for `x86_64-linux-android` via the NDK.
2. `update.sh` copies the resulting `.so` into `app/src/main/jniLibs/x86_64/`.
3. Confirm `app/build.gradle.kts` does not strip the extra ABI (no
   `ndk.abiFilters` set as of 2026-11-25; nothing to do if it stays that way,
   but check before relying on it).
4. `librapier2d_wrapper.so` (cascade engine, copied by hand from
   `android_labs/special_menu/app/src/main/jniLibs/`) is arm64 only. The
   emulator would crash on first page open without an x86_64 build of
   `android_labs/rapier2d_wrapper`. This is likely the harder half; needs its
   own `build_android.sh` variant.
5. `build_and_install.sh` already accepts an emulator when `ANDROID_SERIAL`
   names one, so no change on that side once the libs are in place.

## Non-goals

- Shipping x86_64 to a real phone. Emulator is dev only.
- Old emulator images. minSdk is 31.
- Removing the arm64-only assumption from the release APK.

## What "done" looks like

- `./gradlew test` returns in seconds with real pass/fail instead of
  hanging at 1 test.
- `./build_and_install.sh` against an emulator installs and the app opens;
  settings, nav, and opening a page all work.
- A physical arm64 phone still installs and runs as before.
