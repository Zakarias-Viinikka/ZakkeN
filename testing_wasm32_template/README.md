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
