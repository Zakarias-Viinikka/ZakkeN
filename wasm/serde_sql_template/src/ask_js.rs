use js_sys::{Array, Uint8Array};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = javascript_im_begging_you, catch)]
    fn javascript_im_begging_you(args: &JsValue) -> Result<js_sys::Promise, JsValue>;
}

pub async fn ask(command: &str, payload: Option<Vec<u8>>) -> Result<Vec<u8>, JsValue> {
    let args = Array::new();
    args.push(&JsValue::from_str(command));
    if let Some(bytes) = payload {
        let u8array = Uint8Array::from(bytes.as_slice());
        args.push(&u8array);
    }

    let promise = javascript_im_begging_you(&args)?;
    let result = JsFuture::from(promise).await?;

    if !Array::is_array(&result) {
        return Err(JsValue::from_str("worker returned malformed response"));
    }
    let result_array = Array::from(&result);

    let first = result_array.get(0).as_string().unwrap_or_default();
    if first == "error" {
        let message = result_array
            .get(1)
            .as_string()
            .unwrap_or_else(|| "unknown worker error".to_string());
        return Err(JsValue::from_str(&message));
    }

    let raw = result_array.get(1);
    if raw.is_undefined() {
        return Ok(Vec::new());
    }
    let u8array = raw.dyn_into::<Uint8Array>()?;
    let mut out = vec![0u8; u8array.length() as usize];
    u8array.copy_to(&mut out);
    Ok(out)
}

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
