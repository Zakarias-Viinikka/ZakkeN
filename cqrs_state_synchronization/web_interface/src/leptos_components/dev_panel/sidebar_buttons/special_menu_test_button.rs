use leptos::prelude::*;
use leptos_router::hooks::use_navigate;

#[component]
pub fn SpecialMenuTestButton() -> impl IntoView {
    let navigate = use_navigate();
    view!(
        <button on:click=move |_| {
            navigate("/special_menu_test", Default::default());
        }>"Special menu test"</button>
    )
}
