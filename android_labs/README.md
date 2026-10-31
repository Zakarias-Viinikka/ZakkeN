# feel_labs (Android)

Android port of the web playground messing_around/feel_labs. Kotlin +
Jetpack Compose (foundation only) for the UI; one Rust crate
rapier2d_wrapper for the block-cascade physics.

## Status

All ten labs from the plan are ported. Note app (page list + per-page
cascade editor) is working. Audit and polish pending.

## Labs

- Block transition - type a line, hit Add block; the newest block
  arrives in two panes with different easings (a soft rise, and an
  indent settle that springs past its rest point).
- Running into the wall - as you type and a line wraps, three small
  dots trail leftward from that row.
- Search bar - a mock search over a fixed corpus of 14 docs; matches
  rise on enter and fall on exit, with a shimmer bar while debouncing.
  Replay types a query in.
- Deleting a block - three exit variants (red fade, strike and slide,
  drop). The variant is locked per block at click time.
- Block cascade - insert or delete a block; everything below moves.
  Segmented Fall / Spring / Ripple. Runs on the Rust crate.
- Drag reorder - two pages stacked vertically; a block drags between
  pages via a 48dp left handle and snaps on release. Runs on the crate.
- New page popup - a mock workspace with a modal; every newly typed
  letter shakes 320 ms after appearing.
- Main menu - a page-list menu with a pen and eye per row. Three
  tap-latched effects to compare (underline draw, nudge plus pill, wave).
  The pen wiggles on tap, the eye closes and stays closed.
- Popups - a toast stack that spawns on a virtual clock. Tap a toast
  to inspect it as a modal; auto-dismiss pauses while a modal is open.
- Settings modal - a designed settings modal. Six categories, a live
  preview of the appearance settings, no save.

## Build

    ./build_android.sh
    ./gradlew :app:assembleDebug

build_android.sh runs cargo ndk (arm64-v8a, x86_64) into
app/src/main/jniLibs/ and cargo run --bin uniffi-bindgen into
app/src/main/java/cascade/.

## Tests

    cd rapier2d_wrapper && cargo test

Covers the fall / spring / ripple integrators, the release gate, commit
snap rules, set_height reflow, and locate_drop.

## Plan

android_app/docs/android_port_of_feel_labs.md.