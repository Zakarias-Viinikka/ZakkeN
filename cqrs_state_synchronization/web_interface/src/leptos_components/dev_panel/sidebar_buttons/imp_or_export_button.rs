use leptos::prelude::*;
use leptos_router::hooks::use_navigate;

#[component]
pub fn ImpOrExportButton() -> impl IntoView {
    let navigate = use_navigate();
    view! {
        <button on:click=move |_| {
            navigate("/imp_or_export", Default::default());
        }>"imp/export"</button>
    }
}
