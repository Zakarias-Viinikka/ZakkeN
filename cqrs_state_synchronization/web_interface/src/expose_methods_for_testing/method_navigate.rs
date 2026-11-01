//! Listens for `navigate` on `window`. When fired, the event's `detail`
//! is treated as a path and the router navigates to it.
//!
//! From the tester:
//!
//!     window.dispatchEvent(new CustomEvent('navigate', { detail: '/' }))

#[macro_export]
macro_rules! method_navigate {
    () => {
        let navigate = leptos_router::hooks::use_navigate();
        let handle = leptos::prelude::window_event_listener_untyped(
            "navigate",
            move |ev| {
                if let Ok(custom_ev) =
                    wasm_bindgen::JsCast::dyn_into::<web_sys::CustomEvent>(ev)
                {
                    let detail = custom_ev.detail();
                    if let Some(path) = detail.as_string() {
                        navigate(&path, Default::default());
                    }
                }
            },
        );
        leptos::prelude::on_cleanup(move || {
            handle.remove();
        });
    };
}
