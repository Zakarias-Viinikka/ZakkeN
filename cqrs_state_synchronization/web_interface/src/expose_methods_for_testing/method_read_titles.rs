//! Listens for `read_titles` on `window`. Reads every title row from
//! the local every_block_in_existence table and stores the JSON array
//! in `window.__read_titles__`. The tester reads that global afterwards.
//!
//! From the tester:
//!
//!     window.dispatchEvent(new CustomEvent('read_titles'))
//!     window.__read_titles__  // after a tick

#[macro_export]
macro_rules! method_read_titles {
    () => {
        let handle = leptos::prelude::window_event_listener_untyped(
            "read_titles",
            move |_| {
                leptos::task::spawn_local(async move {
                    use wasm_bindgen::JsCast;
                    let window = match web_sys::window() {
                        Some(w) => w,
                        None => return,
                    };
                    match $crate::db::test_ops_for_db::read_titles().await {
                        Ok(titles) => {
                            let json = leptos::serde_json::to_string(&titles).unwrap_or_else(|_| "[]".to_string());
                            let _ = js_sys::Reflect::set(
                                &window,
                                &wasm_bindgen::JsValue::from_str("__read_titles__"),
                                &wasm_bindgen::JsValue::from_str(&json),
                            );
                            let _ = JsCast::dyn_into::<web_sys::CustomEvent>(
                                wasm_bindgen::JsValue::NULL,
                            );
                        }
                        Err(e) => {
                            $crate::leptos_components::small_components::popup::show_popup(
                                format!("read_titles: {e:?}"),
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
