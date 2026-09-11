// ============================================================
// How to run wasm32 tests in this project
// ============================================================
//
// In Cargo.toml:
//   - gloo-timers (wasm32 target only, with "futures" feature)
//   - futures-timer (for native async timer)
//   - wasm-bindgen-test (dev-dependencies)
//
// Tests live in `tests/`:
//   - native.rs → normal #[test], block_on for async
//   - web.rs    → #![cfg(target_arch = "wasm32")] at top,
//                 #[wasm_bindgen_test], run_in_node
//
// Run all: ./run_all_tests.sh
//
// Remember:
//   - integration tests only see pub items
//   - use crate name, not crate::
//   - wasm-pack test --node (browser gave driver issues)
// ============================================================


// ============================================================
// UniFFI + wasm32
// ============================================================
//
// If a crate uses uniffi AND targets wasm32, enable:
//
//   uniffi = { version = "0.32",
//              features = ["cli", "wasm-unstable-single-threaded"] }
//
// Without it, wasm32 builds fail with a misleading internal error:
//
//   uniffi::constructor: internal error:
//   proc-macro map is missing error entry for crate Crate(Id(NNN))
//
// ============================================================
