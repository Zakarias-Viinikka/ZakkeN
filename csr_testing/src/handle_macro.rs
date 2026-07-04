#[macro_export]
macro_rules! javascript_take_the_wheel {
    ($name:expr, |$payload:ident| $callback:expr) => {
        use leptos::prelude::{on_cleanup, window_event_listener_untyped};
        use wasm_bindgen::{JsCast, JsValue};

        let handle = window_event_listener_untyped($name, move |ev| {
            if let Ok(custom_ev) = ev.dyn_into::<web_sys::CustomEvent>() {
                let $payload: JsValue = custom_ev.detail();
                $callback
            }
        });

        on_cleanup(move || {
            handle.remove();
        });
    };
}
