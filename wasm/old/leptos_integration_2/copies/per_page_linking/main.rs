use leptos::prelude::*;
use leptos_meta::*;

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(App);
}
#[component]
fn App() -> impl IntoView {
    provide_meta_context();
    view! {
        /* copy-dir copies the folder itself into dist, not just its contents. see index.html*/
        <Stylesheet href="/public/per_page_linking/some.css"/>
        <Script src="/public/per_page_linking/write_to_console.js"/>

        <div class="container">
        <button id="write-to-console"
            on:click=move |_| {
            leptos::logging::log!("hello from button");
        }>
        </button>
        </div>
    }
}
