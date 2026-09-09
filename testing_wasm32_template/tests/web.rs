#![cfg(target_arch = "wasm32")]

use testing_wasm32_template::thing_to_test::start_timer;
use wasm_bindgen_test::*;

#[wasm_bindgen_test]
async fn wasm_timer_returns_expected_string() {
    let result = start_timer().await;
    assert_eq!(result, "hello web");
}
