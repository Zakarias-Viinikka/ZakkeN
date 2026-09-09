use futures::executor::block_on;
use testing_wasm32_template::thing_to_test::start_timer;

#[test]
fn native_timer_returns_expected_string() {
    let result = block_on(start_timer());
    assert_eq!(result, "hello world");
}
