use gloo_timers::future::sleep;
use leptos::prelude::*;
use leptos::task::spawn_local_scoped;
use leptos_meta::*;
use std::time::Duration;
use wasm_bindgen::JsCast;
use web_sys::{HtmlScriptElement, window};

#[component]
pub fn CssAndJs() -> impl IntoView {
    let (loaded, set_loaded) = signal(false);

    Effect::new(move |_| {
        if !loaded.get_untracked() {
            spawn_local_scoped(wait_for_sortable(set_loaded));
        }
    });

    view! {
        <Stylesheet href="/public/text_blocks_page/drag_reorder.css"/>
        <Script src="/public/text_blocks_page/Sortable.js"/>

        {move || {
            if loaded.get() {
                view! { <Script src="/public/text_blocks_page/drag_reorder.js"/> }.into_any()
            } else {
                view! { "" }.into_any()
            }
        }}
    }
}

async fn wait_for_sortable(set_loaded: WriteSignal<bool>) {
    loop {
        let loaded = window().and_then(|w| w.get("Sortable")).is_some();

        if loaded {
            set_loaded.set(true);
            break;
        }

        sleep(Duration::from_millis(50)).await;
    }
}
