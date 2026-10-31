//! Listens for `show_popup` on `window`. When fired, shows a popup with
//! the event's `detail` string as the message.
//!
//! From the tester:
//!
//!     window.dispatchEvent(new CustomEvent('show_popup', { detail: 'hi' }))

#[macro_export]
macro_rules! method_show_popup {
    () => {
        let handle = leptos::prelude::window_event_listener_untyped(
            "show_popup",
            move |ev| {
                if let Ok(custom_ev) =
                    wasm_bindgen::JsCast::dyn_into::<web_sys::CustomEvent>(ev)
                {
                    let detail = custom_ev.detail();
                    if let Some(message) = detail.as_string() {
                        $crate::leptos_components::small_components::popup::show_popup(message);
                    }
                }
            },
        );
        leptos::prelude::on_cleanup(move || {
            handle.remove();
        });
    };
}