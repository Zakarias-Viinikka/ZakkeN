use leptos::prelude::*;

#[component]
pub fn ReconcileButton() -> impl IntoView {
    view! {
        <button on:click=move |_| {
            if let Some(window) = web_sys::window() {
                let _ = window.location().set_href("/reconcile");
            }
        }>"reconcile"</button>
    }
}
