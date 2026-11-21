//! Listens for `set_popup_clickable` on `window`. When fired, sets whether
//! popups accept clicks/hover, using the event's boolean `detail`.
//!
//! From the tester:
//!
//!     window.dispatchEvent(new CustomEvent('set_popup_clickable', { detail: false }))

#[macro_export]
macro_rules! method_set_popup_clickable {
    () => {
        let handle = leptos::prelude::window_event_listener_untyped(
            "set_popup_clickable",
            move |ev| {
                if let Ok(custom_ev) =
                    wasm_bindgen::JsCast::dyn_into::<web_sys::CustomEvent>(ev)
                {
                    if let Some(clickable) = custom_ev.detail().as_bool() {
                        $crate::complex_state::clickable_popups::plexset_popup_clickability(
                            clickable,
                        );
                    }
                }
            },
        );
        leptos::prelude::on_cleanup(move || {
            handle.remove();
        });
    };
}
