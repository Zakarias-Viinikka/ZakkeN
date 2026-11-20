use leptos::prelude::*;
use wasm_bindgen::prelude::*; //for being able to link to the js file
/*
wasm-bindgen-futures = "0.4.76" #so i can put async on the code under wasm_bindgen
 */

use js_sys::{Array, Function, Promise};
use wasm_bindgen_futures::JsFuture; // converts a JavaScript Promise into a Rust Future so we can `.await` it // Array: to build a list of arguments for the JS function
// Function: to cast a JS value into a callable function
// Promise: to assert the return type when calling the JS function
use web_sys::window; // gives access to the browser's `window` object so we can reach our global function

use leptos::task::spawn_local;

fn main() {
    console_error_panic_hook::set_once();
    //  trunk serve --open
    mount_to_body(App);
}

#[component]
fn App() -> impl IntoView {
    Effect::new(move || {
        spawn_local(async {
            match beg_js_to_work_the_worker(vec!["list_tables".to_string()]).await {
                Ok(val) => {
                    let text = js_sys::JSON::stringify(&val)
                        .unwrap_or_else(|_| JsValue::from("(unstringifiable)").into());
                    leptos::logging::log!(
                        "Worker response: {}",
                        text.as_string().unwrap_or_default()
                    );
                }
                Err(e) => {
                    leptos::logging::log!("Worker error: {:?}", e);
                }
            }
        });
    });
    view! {
        ""
    }
}

#[wasm_bindgen(js_name = javascript_im_begging_you)]
extern "C" {
    fn javascript_im_begging_you(args: &JsValue) -> js_sys::Promise;
}

async fn beg_js_to_work_the_worker(args: Vec<String>) -> Result<JsValue, JsValue> {
    let arr = js_sys::Array::new();
    for arg in &args {
        arr.push(&JsValue::from_str(arg));
    }
    let promise = javascript_im_begging_you(&arr);
    let result = wasm_bindgen_futures::JsFuture::from(promise).await?;
    Ok(result)
}
