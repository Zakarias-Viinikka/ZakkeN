# cascade_physics / feel_labs Android - AI stored context

Written by AI for AI. Update a section when its content changes and
bump its date. If code and this file disagree, the code wins.

## What this is - 2026-10-31

Android port of `messing_around/feel_labs`. Kotlin + Jetpack Compose
(foundation only, no Material3) plus one Rust crate `cascade_physics`
for the block-cascade physics. The plan lives at
`android_app/docs/android_port_of_feel_labs.md`; read that first.

## State - 2026-10-31

Step 1 of the plan's order of work: scaffold, theme primitives, and a
hello-world crate wired end to end. Labs are not ported yet.

## Toolchain - 2026-10-31

- rustc 1.101.0-nightly, cargo 1.101.0-nightly.
- Android targets installed: aarch64-linux-android, x86_64-linux-android.
- cargo-ndk 4.1.2.
- NDK at `/opt/android-ndk` (ANDROID_NDK_HOME).
- SDK at `/home/zakke/Android/Sdk` (via `local.properties`).
- JDK 21. AGP 9.3.3, Kotlin 2.2.10, Compose BOM 2026.02.01.
- uniffi-bindgen comes from the crate's own `uniffi-bindgen` bin
  target, not from a system CLI.

## Build pattern - 2026-10-31

Same pattern as `android_app` and `rust_to_android_template`:

- `.so` files live pre-built in `app/src/main/jniLibs/{arm64-v8a,x86_64}/`.
- Generated `.kt` lives pre-built in `app/src/main/java/cascade/`.
- `./build_android.sh` at the project root runs `cargo ndk` (both
  targets) then `cargo run --bin uniffi-bindgen` and copies the
  outputs into the Android tree.
- `./gradlew :app:assembleDebug` after that just builds the APK from
  whatever is already on disk.

Gradle does not shell out to cargo. No Exec tasks, no `sourceSets`
blocks, no `kotlin { sourceSets { ... } }`. If you're tempted to add
any of those, look at `android_app` again.

## Bugs hit - 2026-10-31

Three bugs caused the same symptom: blocks collapsed onto each other
and cascades never animated. Logs on device were what found them, not
reading the code.

1. `runFrames` reset `lastNanos` and `first` inside the inner
   `while (!closed)` loop. Each `withFrameNanos` call saw `first=true`,
   so `dt = 0` every frame. The physics stepped with zero dt, nothing
   moved, and `moving` stayed true forever, burning frames. Fix: hoist
   `lastNanos` and `first` out of the inner loop, one pair per kick.

2. `CascadeColumn` had a `render` log reading `yState.floatValue` in
   the composable body. That read registers a snapshot observation, so
   every per-frame write to `yState` recomposed the whole column. The
   fast path is: read `yState.floatValue` only inside
   `graphicsLayer { }` (or `drawBehind`, `offset { }`, `layout { }`),
   never in the composable body.

3. `CascadeColumn` iterated `blockIds.forEach` without `key(id)`.
   Inserting a block shifted slot positions, so Compose disposed and
   recreated the boxes for existing blocks. That fired their
   `onSizeChanged` again with a real height that differed from the
   estimate, which called `set_height` mid-cascade and retargeted
   blocks that were already animating. Fix: `key(id)` around the Box
   so existing boxes are reused.

A fourth bug in the same area: `set_height` re-committed the layout
with `snap_ids` including blocks that were mid-cascade, so a block
whose height was updated while another block was moving would snap the
mover to its target. Fix in `Core::set_height`: skip the reflow when
any block is non-Idle. The height still updates; the layout catches up
the next time `set_height` fires while everything is settled.

## Gotchas - 2026-10-31

- `uniffi` as a crate dep needs the `cli` feature for the
  `uniffi-bindgen` bin target to link.
- The generated file lands at `<out-dir>/cascade/cascade_physics.kt`
  with `package cascade` (from `uniffi.toml`). It must be copied to
  `app/src/main/java/cascade/` — anywhere else and `import cascade.*`
  fails to resolve.
- `uniffi.toml` uses the new-style `[crate-roots]` +
  `[crates.cascade_physics.bindings.kotlin]` format. The old flat
  format is rejected by uniffi 0.32.

## Pending - 2026-10-31

- Step 2: real crate port (layout / step / commit / rapier static
  index) plus cargo tests.
- Then the labs in the plan's order.