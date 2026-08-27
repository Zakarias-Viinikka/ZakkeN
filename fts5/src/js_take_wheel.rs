//this is just because im messing around with dependencies too much. so i can't include one of the projects without getting an error because that one needs this
// and it doesn't exist if i don't add it manually. but this project doesn't actaully need it.
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
