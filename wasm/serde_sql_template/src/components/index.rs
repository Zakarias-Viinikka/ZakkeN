use leptos::prelude::*;
use leptos_router::components::A;

#[component]
pub fn IndexPage() -> impl IntoView {
    view! {
        <br/> <br/> <br/>
        <h2>"Navigation"</h2>
        <A href="/dbgui">"DB Admin GUI | Todo"</A>
        <br/>
        <A href="/">"Tests | Todo"</A>
        <br/>
    }
}
