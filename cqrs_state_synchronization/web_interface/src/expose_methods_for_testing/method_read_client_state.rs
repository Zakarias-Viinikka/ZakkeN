//! Listens for `read_client_state` on `window`. Writes a JSON blob of
//! the client's current state to `window.__client_state__`.
//!
//! From the tester:
//!
//!     window.dispatchEvent(new CustomEvent('read_client_state'))
//!     window.__client_state__  // after a tick

#[macro_export]
macro_rules! method_read_client_state {
    () => {
        let handle = leptos::prelude::window_event_listener_untyped(
            "read_client_state",
            move |_| {
                leptos::task::spawn_local(async move {
                    let window = match web_sys::window() {
                        Some(w) => w,
                        None => return,
                    };
                    match $crate::db::test_ops_for_db::read_client_state().await {
                        Ok(json) => {
                            let _ = js_sys::Reflect::set(
                                &window,
                                &wasm_bindgen::JsValue::from_str("__client_state__"),
                                &wasm_bindgen::JsValue::from_str(&json),
                            );
                        }
                        Err(e) => {
                            $crate::leptos_components::small_components::popup::show_popup(
                                format!("read_client_state: {e:?}"),
                            );
                        }
                    }
                });
            },
        );
        leptos::prelude::on_cleanup(move || {
            handle.remove();
        });
    };
}
