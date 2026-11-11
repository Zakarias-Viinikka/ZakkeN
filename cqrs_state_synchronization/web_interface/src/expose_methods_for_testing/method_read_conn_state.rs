//! Listens for `read_conn_state` on `window`. Writes the connection
//! state (as a Debug string) to `window.__conn_state__`.
//!
//! From the tester:
//!
//!     window.dispatchEvent(new CustomEvent('read_conn_state'))
//!     window.__conn_state__  // after a tick

#[macro_export]
macro_rules! method_read_conn_state {
    () => {
        let handle = leptos::prelude::window_event_listener_untyped(
            "read_conn_state",
            move |_| {
                let window = match web_sys::window() {
                    Some(w) => w,
                    None => return,
                };
                let state = $crate::owner_of_interacting_with_server::CONN_STATE
                    .get_untracked();
                let text = format!("{:?}", state);
                let _ = js_sys::Reflect::set(
                    &window,
                    &wasm_bindgen::JsValue::from_str("__conn_state__"),
                    &wasm_bindgen::JsValue::from_str(&text),
                );
            },
        );
        leptos::prelude::on_cleanup(move || {
            handle.remove();
        });
    };
}
