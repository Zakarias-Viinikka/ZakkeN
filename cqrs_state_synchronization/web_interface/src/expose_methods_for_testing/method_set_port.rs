//! Listens for `set_port` on `window`. When fired, the event's `detail`
//! (a number) is written to the server port and the connect gate flips.
//!
//! From the tester:
//!
//!     window.dispatchEvent(new CustomEvent('set_port', { detail: 41495 }))

#[macro_export]
macro_rules! method_set_port {
    () => {
        let handle = leptos::prelude::window_event_listener_untyped(
            "set_port",
            move |ev| {
                if let Ok(custom_ev) =
                    wasm_bindgen::JsCast::dyn_into::<web_sys::CustomEvent>(ev)
                {
                    let detail = custom_ev.detail();
                    if let Some(n) = detail.as_f64() {
                        $crate::owner_of_interacting_with_server::server_port::set_port(n as u16);
                    }
                }
            },
        );
        leptos::prelude::on_cleanup(move || {
            handle.remove();
        });
    };
}
