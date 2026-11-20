use gloo_timers::future::sleep;
use leptos::prelude::*;
use leptos::task::spawn_local_scoped;
use leptos_meta::*;
use std::time::Duration;
use wasm_bindgen::JsValue;
use web_sys::window;

#[component]
pub fn DbGuiAssets(loaded: WriteSignal<bool>) -> impl IntoView {
    Effect::new(move |_| {
        spawn_local_scoped(wait_for_stylesheet(loaded));
    });

    view! {
        <Stylesheet href="/css/dbgui.css" />
    }
}

async fn wait_for_stylesheet(set_loaded: WriteSignal<bool>) {
    loop {
        let found = window()
            .and_then(|w| w.document())
            .and_then(|d| {
                d.query_selector("link[href='/css/dbgui.css']")
                    .ok()
                    .flatten()
            })
            .map(|el| {
                js_sys::Reflect::get(&el, &JsValue::from_str("sheet"))
                    .map(|v| !v.is_null() && !v.is_undefined())
                    .unwrap_or(false)
            })
            .unwrap_or(false);

        if found {
            set_loaded.set(true);
            break;
        }

        sleep(Duration::from_millis(50)).await;
    }
}
